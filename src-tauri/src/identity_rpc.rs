// File: src-tauri/src/identity_rpc.rs
// Description: Handles VerusID identity-related RPC calls.
// Changes:
// - Separated identity fetching from balance fetching for progressive loading
// - Added get_login_identities_fast for immediate name loading
// - Updated get_login_identities to maintain compatibility
// - Added get_identity_balance for individual balance fetching
// - NEW: Added VerusID registration helpers and Tauri commands:
//   - get_new_address (getnewaddress)
//   - get_new_private_address (z_getnewaddress)
//   - register_name_commitment (registernamecommitment)
//   - register_identity (registeridentity)
//   - get_transaction_confirmations (gettransaction)
//   - wait_for_confirmations (poll confirmations)
//   - get_identity (getidentity raw)
//   - dump_privkey (dumpprivkey)
//   - export_z_key (z_exportkey)

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::rpc_client::{make_rpc_call, VerusRpcError};
use super::wallet_rpc::get_private_balance;
use tokio::time::{sleep, Duration};

// Updated struct to include balance for dropdown display
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormattedIdentity {
    pub formatted_name: String,        // Transformed from fullyqualifiedname
    pub i_address: String,            // identityaddress
    pub private_address: String,      // privateaddress (required, not optional)
    pub balance: Option<f64>,         // Private balance (None while loading)
}

// NEW: Fast function to get identities without balances for progressive loading
pub async fn get_login_identities_fast(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
) -> Result<Vec<FormattedIdentity>, VerusRpcError> {
    log::info!("Fetching identities (fast mode - no balances)...");

    let identities_raw: Vec<Value> = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "listidentities",
        vec![json!(true), json!(true), json!(true)],
    )
    .await?;

    log::info!("Received {} raw identity entries from listidentities.", identities_raw.len());

    let mut qualifying_identities = Vec::new();

    // Step 1: Filter identities based on new criteria
    for identity_obj in identities_raw {
        if let Some(identity_details) = identity_obj.get("identity") {
            // Check all required fields and conditions
            let private_address = identity_details.get("privateaddress")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            
            let can_spend_for = identity_obj.get("canspendfor")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let can_sign_for = identity_obj.get("cansignfor")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let identity_address = identity_details.get("identityaddress")
                .and_then(|v| v.as_str());

            // Apply enhanced filtering criteria
            if let (Some(private_addr), Some(id_addr)) = (private_address, identity_address) {
                if can_spend_for && can_sign_for {
                    log::debug!("Identity {} qualifies: has private address, canspendfor=true, cansignfor=true", id_addr);
                    qualifying_identities.push((id_addr.to_string(), private_addr.to_string()));
                } else {
                    log::debug!("Identity {} skipped: canspendfor={}, cansignfor={}", id_addr, can_spend_for, can_sign_for);
                }
            } else {
                log::debug!("Identity skipped: missing private address or identity address");
            }
        } else {
            log::warn!("Skipping raw identity entry because 'identity' sub-object is missing.");
        }
    }

    if qualifying_identities.is_empty() {
        log::error!("No qualifying VerusIDs found (must have private address, canspendfor=true, cansignfor=true).");
        return Err(VerusRpcError::Rpc {
            code: -1,
            message: "No eligible VerusIDs found. Identities must have private addresses and spending/signing permissions.".to_string(),
        });
    }

    log::info!("Found {} qualifying identities, fetching names...", qualifying_identities.len());

    // Step 2: Get formatted names using getidentity + fullyqualifiedname (NO BALANCE FETCHING)
    let mut formatted_identities = Vec::new();

    for (identity_address, private_address) in qualifying_identities {
        log::debug!("Fetching name for identity: {}", identity_address);
        
        match make_rpc_call::<Value>(&rpc_user, &rpc_pass, rpc_port, "getidentity", vec![json!(identity_address)]).await {
            Ok(identity_result) => {
                if let Some(fully_qualified_name) = identity_result.get("fullyqualifiedname").and_then(|v| v.as_str()) {
                    // Transform fullyqualifiedname by removing everything after the last dot before @
                    let formatted_name = transform_fully_qualified_name(fully_qualified_name);
                    
                    log::debug!("Transformed '{}' -> '{}'", fully_qualified_name, formatted_name);
                    
                    formatted_identities.push(FormattedIdentity {
                        formatted_name,
                        i_address: identity_address.clone(),
                        private_address: private_address.clone(),
                        balance: None, // No balance fetching in fast mode
                    });
                } else {
                    log::warn!("No fullyqualifiedname found for identity {}, skipping", identity_address);
                }
            }
            Err(e) => {
                log::error!("Failed to get identity details for {}: {:?}, skipping", identity_address, e);
            }
        }
    }

    if formatted_identities.is_empty() {
        log::error!("No identities could be processed for name formatting.");
        return Err(VerusRpcError::Rpc {
            code: -1,
            message: "Failed to process identity names.".to_string(),
        });
    }

    log::info!("Successfully processed {} identities (fast mode)", formatted_identities.len());

    Ok(formatted_identities)
}

// NEW: Function to get balance for a specific identity
pub async fn get_identity_balance(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    private_address: String,
) -> Result<f64, VerusRpcError> {
    log::debug!("Fetching balance for private address: {}", private_address);
    get_private_balance(rpc_user, rpc_pass, rpc_port, private_address).await
}

// Updated function with new filtering logic and balance integration (MAINTAINED FOR COMPATIBILITY)
pub async fn get_login_identities(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
) -> Result<Vec<FormattedIdentity>, VerusRpcError> {
    log::info!("Fetching identities for login selection with enhanced filtering...");

    // First get identities without balances
    let mut identities = get_login_identities_fast(rpc_user.clone(), rpc_pass.clone(), rpc_port).await?;

    // Then fetch balances for all identities
    for identity in &mut identities {
        log::debug!("Fetching balance for {}", identity.private_address);
        
        match get_private_balance(rpc_user.clone(), rpc_pass.clone(), rpc_port, identity.private_address.clone()).await {
            Ok(balance) => {
                identity.balance = Some(balance);
                log::debug!("Balance for {}: {:.5}", identity.formatted_name, balance);
            }
            Err(e) => {
                log::warn!("Failed to fetch balance for {}: {:?}, will show '-'", identity.formatted_name, e);
                identity.balance = None; // Will be displayed as "-" in UI
            }
        }
    }

    // Sort by balance (highest first), treating None as 0
    identities.sort_by(|a, b| {
        let balance_a = a.balance.unwrap_or(0.0);
        let balance_b = b.balance.unwrap_or(0.0);
        balance_b.partial_cmp(&balance_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    log::info!("Successfully processed {} identities with balances", identities.len());

    Ok(identities)
}

// Helper function to transform fullyqualifiedname
fn transform_fully_qualified_name(fully_qualified_name: &str) -> String {
    // Remove everything after the last dot before @
    // Example: "JohnGomez.parent.VRSCTEST@" -> "JohnGomez.parent@"
    // Example: "JohnGomez.VRSCTEST@" -> "JohnGomez@"
    
    if let Some(at_pos) = fully_qualified_name.rfind('@') {
        let before_at = &fully_qualified_name[..at_pos];
        if let Some(last_dot_pos) = before_at.rfind('.') {
            format!("{}@", &before_at[..last_dot_pos])
        } else {
            // No dot found, return as-is (shouldn't happen based on requirements)
            fully_qualified_name.to_string()
        }
    } else {
        // No @ found, return as-is (malformed name)
        fully_qualified_name.to_string()
    }
}

// NEW function for New Chat: Check identity eligibility
pub async fn check_identity_eligibility(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    target_identity_name: String,
) -> Result<FormattedIdentity, VerusRpcError> {
    log::info!("Checking eligibility for identity: {}", target_identity_name);

    // Basic format check
    if !target_identity_name.ends_with('@') || target_identity_name.len() <= 1 {
        log::warn!("Invalid identity format: {}", target_identity_name);
        return Err(VerusRpcError::InvalidFormat);
    }

    match make_rpc_call::<Value>(&rpc_user, &rpc_pass, rpc_port, "getidentity", vec![json!(target_identity_name)]).await {
        Ok(identity_result) => {
            log::debug!("getidentity result for {}: {:?}", target_identity_name, identity_result);
            if let Some(identity_details) = identity_result.get("identity") {
                let private_address_opt = identity_details.get("privateaddress")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from);

                if private_address_opt.is_some() {
                    if let (Some(name), Some(i_address), Some(parent_id), Some(system_id)) = (
                        identity_details.get("name").and_then(|v| v.as_str()),
                        identity_details.get("identityaddress").and_then(|v| v.as_str()),
                        identity_details.get("parent").and_then(|v| v.as_str()),
                        identity_details.get("systemid").and_then(|v| v.as_str()),
                    ) {
                        // Start with default format
                        let mut formatted_name = format!("{}@", name);
                        
                        // Check if it's a sub-ID (parent is not the system ID)
                        if parent_id != system_id {
                            log::debug!("Identity '{}' is a sub-ID. Fetching parent '{}'...", name, parent_id);
                            // Get parent identity to format the name properly (name.parentname@)
                            match make_rpc_call::<Value>(&rpc_user, &rpc_pass, rpc_port, "getidentity", vec![json!(parent_id)]).await {
                                Ok(parent_identity_result) => {
                                    // Extract parent name from the parent identity details
                                    if let Some(parent_name) = parent_identity_result
                                        .get("identity")
                                        .and_then(|id_details| id_details.get("name"))
                                        .and_then(|n| n.as_str()) 
                                    {
                                        log::debug!("Parent name found: {}", parent_name);
                                        formatted_name = format!("{}.{}@", name, parent_name);
                                    } else {
                                        log::error!("Failed to extract parent name for sub-ID. Using default format.");
                                        // Keep default format as fallback
                                    }
                                },
                                Err(e) => {
                                    log::error!("Error fetching parent identity: {:?}. Using default format.", e);
                                    // Keep default format as fallback
                                }
                            }
                        }
                        
                        log::info!("Identity {} is eligible. Formatted as: {}", target_identity_name, formatted_name);
                        Ok(FormattedIdentity {
                            formatted_name,
                            i_address: i_address.to_string(),
                            private_address: private_address_opt.unwrap(),
                            balance: None,
                        })
                    } else {
                        log::warn!("Identity {} found but missing required fields.", target_identity_name);
                        Err(VerusRpcError::NotFoundOrIneligible)
                    }
                } else {
                    log::warn!("Identity {} found but has no private address.", target_identity_name);
                    Err(VerusRpcError::NotFoundOrIneligible)
                }
            } else {
                 log::warn!("'identity' object not found in getidentity result for {}.", target_identity_name);
                 Err(VerusRpcError::NotFoundOrIneligible)
            }
        }
        Err(e) => {
            // Handle specific error types that indicate "Not Found" for getidentity
            match e {
                VerusRpcError::Rpc { code, ref message } if code == -5 || code == -8 => {
                    // Code -5: Invalid address or key (Identity not found)
                    // Code -8: Invalid parameter (Could also indicate identity not found)
                    log::warn!("getidentity RPC error indicates not found for {}: code={}, message={}", target_identity_name, code, message);
                    Err(VerusRpcError::NotFoundOrIneligible)
                },
                VerusRpcError::ParseError(ref msg) if msg.contains("500 Internal Server Error") => {
                     // Treat 500 error specifically for getidentity as likely not found
                    log::warn!("getidentity received 500 error, treating as not found for {}: {}", target_identity_name, msg);
                    Err(VerusRpcError::NotFoundOrIneligible)
                }
                _ => {
                    // Propagate other errors (network, timeout, different RPC errors, etc.)
                    log::error!("RPC call failed for getidentity({}): {:?}", target_identity_name, e);
                    Err(e)
                }
            }
        }
    }
}

// NEW: Check if a VerusID exists (for name availability and referral validation)
pub async fn check_identity_exists(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    identity_name: String,
) -> Result<bool, VerusRpcError> {
    log::info!("Checking existence of identity: {}", identity_name);

    // Basic format check to avoid unnecessary RPC calls
    if !identity_name.ends_with('@') || identity_name.len() <= 1 {
        log::warn!("Invalid identity format for existence check: {}", identity_name);
        // We can treat this as "not existing" for validation purposes
        return Ok(false);
    }

    match make_rpc_call::<Value>(&rpc_user, &rpc_pass, rpc_port, "getidentity", vec![json!(identity_name)]).await {
        Ok(_) => {
            // If we get any successful result, the identity exists.
            log::info!("Identity '{}' exists.", identity_name);
            Ok(true)
        }
        Err(e) => {
            // Handle the specific "Not Found" error, which is a success case for name availability checks.
            match e {
                VerusRpcError::Rpc { code, ref message } if code == -5 || code == -8 => {
                    // Code -5: Invalid address or key (Identity not found)
                    // Code -8: Invalid parameter (Could also indicate identity not found)
                    log::info!("Identity '{}' does not exist (RPC code {}): {}", identity_name, code, message);
                    Ok(false)
                },
                VerusRpcError::ParseError(ref msg) if msg.contains("500 Internal Server Error") => {
                     // Treat 500 error specifically for getidentity as likely not found
                    log::warn!("getidentity received 500 error for '{}', treating as non-existent: {}", identity_name, msg);
                    Ok(false)
                }
                _ => {
                    // Propagate other errors (network, timeout, different RPC errors, etc.)
                    log::error!("RPC call failed while checking existence for '{}': {:?}", identity_name, e);
                    Err(e)
                }
            }
        }
    }
} 

// --- Registration helpers & commands ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NameCommitmentResponse {
    pub txid: String,
    pub namereservation: Value,
}

/// Get a new transparent control address (R-addr)
#[tauri::command]
pub async fn get_new_address(app: tauri::AppHandle) -> Result<String, String> {
    log::info!("get_new_address called");
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    let addr = make_rpc_call::<String>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "getnewaddress", vec![])
        .await
        .map_err(|e| format!("getnewaddress failed: {}", e))?;
    log::info!("get_new_address result: {}", addr);
    Ok(addr)
}

/// Get a new shielded private address (zs-addr)
#[tauri::command]
pub async fn get_new_private_address(app: tauri::AppHandle) -> Result<String, String> {
    log::info!("get_new_private_address called");
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    let zaddr = make_rpc_call::<String>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "z_getnewaddress", vec![])
        .await
        .map_err(|e| format!("z_getnewaddress failed: {}", e))?;
    log::info!("get_new_private_address result: {}", zaddr);
    Ok(zaddr)
}

/// Call registernamecommitment
#[tauri::command]
pub async fn register_name_commitment(
    app: tauri::AppHandle,
    name: String,
    control_address: String,
    referral_identity: Option<String>,
    parent_namespace: Option<String>,
) -> Result<NameCommitmentResponse, String> {
    log::info!(
        "register_name_commitment: name={}, control={}, referral='{}', parent='{}'",
        name,
        control_address,
        referral_identity.clone().unwrap_or_else(|| "".into()),
        parent_namespace.clone().unwrap_or_else(|| "".into())
    );
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    // Per spec, explicitly pass empty string when not provided
    let referral = referral_identity.unwrap_or_else(|| "".to_string());
    let parent = parent_namespace.unwrap_or_else(|| "".to_string());

    let result: Value = make_rpc_call(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "registernamecommitment",
        vec![json!(name), json!(control_address), json!(referral), json!(parent)],
    )
    .await
    .map_err(|e| format!("registernamecommitment failed: {}", e))?;

    // Expect { txid, namereservation: {...} }
    let txid = result
        .get("txid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing txid in registernamecommitment response".to_string())?
        .to_string();

    let namereservation = result
        .get("namereservation")
        .cloned()
        .ok_or_else(|| "Missing namereservation in response".to_string())?;

    log::info!("register_name_commitment txid: {}", txid);
    Ok(NameCommitmentResponse { txid, namereservation })
}

/// Call registeridentity with pass-through bundle, return txid
#[tauri::command]
pub async fn register_identity(app: tauri::AppHandle, identity_bundle: Value) -> Result<String, String> {
    log::info!("register_identity called");
    log::debug!("register_identity payload: {}", identity_bundle);
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    let result: Value = make_rpc_call(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "registeridentity",
        vec![identity_bundle],
    )
    .await
    .map_err(|e| format!("registeridentity failed: {}", e))?;

    // Try common shapes: string txid or object with txid
    if let Some(txid) = result.as_str() {
        log::info!("register_identity txid: {}", txid);
        return Ok(txid.to_string());
    }
    if let Some(txid) = result.get("txid").and_then(|v| v.as_str()) {
        log::info!("register_identity txid: {}", txid);
        return Ok(txid.to_string());
    }
    // Fallback to serialize
    log::warn!("register_identity unexpected response shape: {}", result);
    Ok(result.to_string())
}

/// Get confirmations for a txid using gettransaction
#[tauri::command]
pub async fn get_transaction_confirmations(app: tauri::AppHandle, txid: String) -> Result<u64, String> {
    log::info!("get_transaction_confirmations({}, ..)", txid);
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    // Try gettransaction first
    let primary = make_rpc_call::<Value>(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "gettransaction",
        vec![json!(txid.clone())],
    )
    .await;
    let result: Value = match primary {
        Ok(val) => val,
        Err(e) => {
            log::warn!("gettransaction failed for {}: {:?}. Falling back to getrawtransaction(verbose)", txid, e);
            make_rpc_call::<Value>(
                &creds.rpc_user,
                &creds.rpc_pass,
                creds.rpc_port,
                "getrawtransaction",
                vec![json!(txid.clone()), json!(true)],
            )
            .await
            .map_err(|e2| format!("gettransaction failed and getrawtransaction fallback also failed: {}", e2))?
        }
    };

    let confs = result
        .get("confirmations")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    log::info!("tx {} confirmations: {}", txid, confs);
    Ok(confs)
}

/// Wait until a tx reaches min confirmations, or timeout
#[tauri::command]
pub async fn wait_for_confirmations(
    app: tauri::AppHandle,
    txid: String,
    min_confirmations: u64,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<bool, String> {
    let start = std::time::Instant::now();
    loop {
        let confs = match get_transaction_confirmations(app.clone(), txid.clone()).await {
            Ok(c) => c,
            Err(e) => {
                log::error!("wait_for_confirmations get tx error: {}", e);
                return Err(e);
            }
        };
        if confs >= min_confirmations {
            return Ok(true);
        }
        if start.elapsed() >= Duration::from_secs(timeout_secs) {
            log::warn!(
                "wait_for_confirmations timeout: tx={}, waited_secs={}, required_confs={}",
                txid,
                timeout_secs,
                min_confirmations
            );
            return Ok(false);
        }
        sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Raw getidentity call to retrieve identity object
#[tauri::command]
pub async fn get_identity(app: tauri::AppHandle, identity_name: String) -> Result<Value, String> {
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    make_rpc_call::<Value>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "getidentity", vec![json!(identity_name)])
        .await
        .map_err(|e| format!("getidentity failed: {}", e))
}

/// Check if identity exists (returns true/false instead of erroring on not found)
#[tauri::command]
pub async fn check_identity_ready(app: tauri::AppHandle, identity_name: String) -> Result<bool, String> {
    log::info!("check_identity_ready: checking {}", identity_name);
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    match make_rpc_call::<Value>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "getidentity", vec![json!(identity_name)]).await {
        Ok(_) => {
            log::info!("check_identity_ready: {} exists", identity_name);
            Ok(true)
        }
        Err(e) => {
            // Handle specific "not found" errors
            match e {
                VerusRpcError::Rpc { code, ref message } if code == -5 => {
                    // Code -5: Identity not found (expected during registration process)
                    log::debug!("check_identity_ready: {} not found yet (code -5): {}", identity_name, message);
                    Ok(false)
                }
                VerusRpcError::ParseError(ref msg) if msg.contains("500 Internal Server Error") => {
                    // 500 errors for getidentity usually mean "not found" as well
                    log::debug!("check_identity_ready: {} not found yet (500 error): {}", identity_name, msg);
                    Ok(false)
                }
                _ => {
                    // Propagate other errors (network, auth, etc.)
                    log::error!("check_identity_ready: unexpected error for {}: {:?}", identity_name, e);
                    Err(format!("Error checking identity: {}", e))
                }
            }
        }
    }
}

/// Wait for identity to become available with polling
#[tauri::command]
pub async fn wait_for_identity_ready(
    app: tauri::AppHandle,
    identity_name: String,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<bool, String> {
    log::info!("wait_for_identity_ready: waiting for {} (timeout: {}s)", identity_name, timeout_secs);
    let start = std::time::Instant::now();
    
    loop {
        match check_identity_ready(app.clone(), identity_name.clone()).await {
            Ok(true) => {
                log::info!("wait_for_identity_ready: {} is ready", identity_name);
                return Ok(true);
            }
            Ok(false) => {
                log::debug!("wait_for_identity_ready: {} not ready yet, continuing to poll", identity_name);
                // Continue polling
            }
            Err(e) => {
                log::error!("wait_for_identity_ready: error checking {}: {}", identity_name, e);
                return Err(e);
            }
        }

        if start.elapsed() >= Duration::from_secs(timeout_secs) {
            log::warn!("wait_for_identity_ready: timeout waiting for {}", identity_name);
            return Ok(false);
        }

        sleep(Duration::from_secs(interval_secs)).await;
    }
}

/// Export transparent private key (WIF) for control R-addr
#[tauri::command]
pub async fn dump_privkey(app: tauri::AppHandle, address: String) -> Result<String, String> {
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    make_rpc_call::<String>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "dumpprivkey", vec![json!(address)])
        .await
        .map_err(|e| format!("dumpprivkey failed: {}", e))
}

/// Export shielded private key for zs-addr
#[tauri::command]
pub async fn export_z_key(app: tauri::AppHandle, z_address: String) -> Result<String, String> {
    let creds = crate::credentials::load_credentials(app)
        .await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    make_rpc_call::<String>(&creds.rpc_user, &creds.rpc_pass, creds.rpc_port, "z_exportkey", vec![json!(z_address)])
        .await
        .map_err(|e| format!("z_exportkey failed: {}", e))
}