// File: src-tauri/src/namespace_rpc.rs
// Description: Handles RPC calls for fetching and filtering available namespaces for VerusID registration
// Changes:
// - Updated filtering to require ALL reserves > 0 for valid namespaces
// - Added fee currency resolution based on idimportfees logic
// - Implemented parallel getcurrency calls to resolve currency names
// - Enhanced NamespaceOption to include actual fee currency name
// - Added get_root_currency function to fetch blockchain's native currency data
// - Added blockchain ID to currency name mapping for getcurrency calls

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::rpc_client::{make_rpc_call, VerusRpcError};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyDefinition {
    pub version: u32,
    pub options: u32,
    pub name: String,
    pub currencyid: String,
    pub parent: Option<String>,
    pub systemid: String,
    pub notarizationprotocol: Option<u32>,
    pub proofprotocol: u32,
    pub launchsystemid: Option<String>,
    pub startblock: u64,
    pub endblock: u64,
    pub currencies: Option<Vec<String>>,
    pub weights: Option<Vec<f64>>,
    pub conversions: Option<Vec<f64>>,
    pub initialsupply: Option<f64>,
    pub prelaunchcarveout: Option<f64>,
    pub preallocations: Option<Vec<Value>>,
    pub initialcontributions: Option<Vec<f64>>,
    pub idregistrationfees: f64,
    pub idreferrallevels: u32,
    pub idimportfees: f64,
    pub currencyidhex: String,
    pub fullyqualifiedname: String,
    pub definitiontxid: String,
    pub definitiontxout: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReserveCurrency {
    pub currencyid: String,
    pub weight: f64,
    pub reserves: f64,
    pub priceinreserve: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BestCurrencyState {
    pub flags: u32,
    pub version: u32,
    pub currencyid: String,
    pub reservecurrencies: Option<Vec<ReserveCurrency>>, // Made optional - not all currencies have reserves
    pub initialsupply: Option<f64>,
    pub emitted: Option<f64>,
    pub supply: Option<f64>,
    pub currencies: Option<serde_json::Value>,
    pub primarycurrencyfees: Option<f64>,
    pub primarycurrencyconversionfees: Option<f64>,
    pub primarycurrencyout: Option<f64>,
    pub preconvertedout: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrencyInfo {
    pub currencydefinition: CurrencyDefinition,
    pub bestheight: u64,
    pub besttxid: String,
    pub besttxout: u32,
    pub bestcurrencystate: BestCurrencyState,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetCurrencyResponse {
    pub name: String,
    pub currencyid: String,
    pub idregistrationfees: f64,
    pub idimportfees: f64,
    pub currencynames: Option<HashMap<String, String>>,
    pub bestcurrencystate: Option<BestCurrencyState>,
    // Make other fields optional to handle different response structures
    pub version: Option<u32>,
    pub options: Option<u32>,
    pub parent: Option<String>,
    pub systemid: Option<String>,
    pub notarizationprotocol: Option<u32>,
    pub proofprotocol: Option<u32>,
    pub launchsystemid: Option<String>,
    pub startblock: Option<u64>,
    pub endblock: Option<u64>,
    pub currencies: Option<Vec<String>>,
    pub weights: Option<Vec<f64>>,
    pub conversions: Option<Vec<f64>>,
    pub initialsupply: Option<f64>,
    pub prelaunchcarveout: Option<f64>,
    pub initialcontributions: Option<Vec<f64>>,
    pub idreferrallevels: Option<u32>,
    pub currencyidhex: Option<String>,
    pub fullyqualifiedname: Option<String>,
    pub magicnumber: Option<i64>, // Changed from u64 to i64 - magic numbers can be negative
    pub definitiontxid: Option<String>,
    pub definitiontxout: Option<u32>,
    pub bestheight: Option<u64>,
    pub lastconfirmedheight: Option<u64>,
    pub lastconfirmedcurrencystate: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NamespaceOption {
    pub name: String,
    pub currency_id: String,
    pub registration_fee: f64,
    pub fully_qualified_name: String,
    pub fee_currency_name: String,
    pub options: u32,
    pub id_referral_levels: u32,
}

// Response structure for getcurrency (for root currency)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RootCurrencyResponse {
    pub name: String,
    pub currencyid: String,
    pub idregistrationfees: f64,
    pub idreferrallevels: u32,
    // Only include the fields we need for root currency
}

#[tauri::command]
pub async fn get_available_namespaces(
    app: tauri::AppHandle,
) -> Result<Vec<NamespaceOption>, String> {
    println!("Starting namespace fetch...");
    
    // Load credentials first
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    
    println!("Credentials loaded, calling listcurrencies...");
    
    // Call listcurrencies RPC method
    let response: Value = make_rpc_call(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "listcurrencies",
        vec![],
    ).await
        .map_err(|e| format!("Failed to call listcurrencies: {}", e))?;
    
    println!("Got response, parsing currencies...");
    println!("Response type: {:?}", response.as_array().map(|arr| arr.len()).unwrap_or(0));
    
    // Parse the response as an array of currency info
    let currencies_array: Vec<CurrencyInfo> = match serde_json::from_value::<Vec<CurrencyInfo>>(response.clone()) {
        Ok(currencies) => {
            println!("Successfully parsed {} currencies", currencies.len());
            currencies
        }
        Err(e) => {
            println!("Failed to parse currencies response: {}", e);
            println!("Response sample: {}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Unable to serialize".to_string()));
            return Err(format!("Failed to parse currencies response: {}", e));
        }
    };
    
    println!("Parsed {} currencies, filtering...", currencies_array.len());
    
    // First pass: filter by existing criteria + ALL reserves > 0
    let mut valid_currency_infos = Vec::new();
    
    for (index, currency_info) in currencies_array.iter().enumerate() {
        let def = &currency_info.currencydefinition;
        
        println!(
            "Processing currency {}/{}: {} - options: {}, proofprotocol: {}",
            index + 1, currencies_array.len(), def.name, def.options, def.proofprotocol
        );
        
        // Filter criteria: options must be 33 or 41, and proofprotocol must be 1
        if (def.options == 33 || def.options == 41) && def.proofprotocol == 1 {
            println!("  ✓ Currency {} passed options/proofprotocol check", def.name);
            
            // NEW: Check that currency has reserves and ALL reserves > 0
            if let Some(reserves) = &currency_info.bestcurrencystate.reservecurrencies {
                if reserves.is_empty() {
                    println!("  ✗ Currency {} has empty reserves array", def.name);
                } else {
                    let reserve_count = reserves.len();
                    println!("  Checking {} reserves for currency {}", reserve_count, def.name);
                    
                    let mut all_reserves_positive = true;
                    for (i, reserve) in reserves.iter().enumerate() {
                        println!("    Reserve {}: {} (reserves: {})", i, reserve.currencyid, reserve.reserves);
                        if reserve.reserves <= 0.0 {
                            all_reserves_positive = false;
                            println!("    ✗ Reserve {} has zero reserves: {}", i, reserve.reserves);
                        }
                    }
                    
                    if all_reserves_positive {
                        println!("  ✓ Currency {} passed all filters (all {} reserves > 0)", def.name, reserve_count);
                        valid_currency_infos.push(currency_info.clone());
                    } else {
                        println!("  ✗ Currency {} failed reserve check (some reserves are 0)", def.name);
                    }
                }
            } else {
                println!("  ✗ Currency {} has no reserves field", def.name);
            }
        } else {
            println!("  ✗ Currency {} failed options/proofprotocol check (options: {}, proofprotocol: {})", 
                def.name, def.options, def.proofprotocol);
        }
    }
    
    println!("Found {} currencies passing initial filters", valid_currency_infos.len());
    
    // Second pass: make batched getcurrency calls to resolve fee currencies (5 at a time)
    println!("Processing {} namespaces in batches of 5...", valid_currency_infos.len());
    
    if valid_currency_infos.is_empty() {
        println!("No namespaces to process - returning empty list");
        return Ok(Vec::new());
    }
    
    let mut valid_namespaces = Vec::new();
    let batch_size = 5;
    let total_batches = (valid_currency_infos.len() + batch_size - 1) / batch_size;
    
    // Process in batches
    for (batch_index, batch) in valid_currency_infos.chunks(batch_size).enumerate() {
        println!("Processing batch {}/{} ({} items)...", batch_index + 1, total_batches, batch.len());
        
        // Create futures for this batch
        let mut batch_futures = Vec::new();
        
        for currency_info in batch {
            let currency_id = currency_info.currencydefinition.currencyid.clone();
            let rpc_user = creds.rpc_user.clone();
            let rpc_pass = creds.rpc_pass.clone();
            let rpc_port = creds.rpc_port;
            let currency_info_clone = currency_info.clone();
            
            let future = async move {
                resolve_namespace_fee_currency(
                    currency_info_clone,
                    &rpc_user,
                    &rpc_pass,
                    rpc_port,
                ).await
            };
            
            batch_futures.push(future);
        }
        
        // Execute this batch in parallel
        let batch_results = futures::future::join_all(batch_futures).await;
        
        // Process batch results
        for (local_index, result) in batch_results.into_iter().enumerate() {
            let global_index = batch_index * batch_size + local_index + 1;
            match result {
                Ok(namespace) => {
                    println!("✓ Result {}: Successfully resolved namespace: {} (fee: {} {})", 
                        global_index, namespace.name, namespace.registration_fee, namespace.fee_currency_name);
                    valid_namespaces.push(namespace);
                }
                Err(e) => {
                    println!("✗ Result {}: Failed to resolve namespace: {}", global_index, e);
                    // Skip this namespace as requested
                }
            }
        }
        
        // Small delay between batches to be nice to the RPC server
        if batch_index < total_batches - 1 {
            println!("Waiting 100ms before next batch...");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
    
    println!("Final result: {} valid namespaces", valid_namespaces.len());
    
    // Sort by name for better UX
    valid_namespaces.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(valid_namespaces)
}

// Map blockchain ID to currency name for getcurrency calls
fn get_currency_name_for_blockchain(blockchain_id: &str) -> Option<String> {
    match blockchain_id {
        "verus-testnet" => Some("vrsctest".to_string()),
        "verus" => Some("vrsc".to_string()),
        "chips" => Some("chips".to_string()),
        "vdex" => Some("vdex".to_string()),
        "varrr" => Some("varrr".to_string()),
        _ => None,
    }
}

// Get root currency information for a blockchain
#[tauri::command]
pub async fn get_root_currency(
    app: tauri::AppHandle,
    blockchain_id: String,
) -> Result<NamespaceOption, String> {
    println!("Getting root currency for blockchain: {}", blockchain_id);
    
    // Load credentials
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    
    // Get the currency name for this blockchain
    let currency_name = get_currency_name_for_blockchain(&blockchain_id)
        .ok_or_else(|| format!("Unsupported blockchain: {}", blockchain_id))?;
    
    println!("Calling getcurrency for: {}", currency_name);
    
    // Call getcurrency RPC method
    let response: Value = make_rpc_call(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "getcurrency",
        vec![json!(currency_name)],
    ).await
        .map_err(|e| format!("Failed to call getcurrency: {}", e))?;
    
    println!("Got getcurrency response for {}", currency_name);
    
    // Parse the response
    let root_currency: RootCurrencyResponse = serde_json::from_value::<RootCurrencyResponse>(response.clone())
        .map_err(|e| {
            println!("Failed to parse getcurrency response: {}", e);
            println!("Response: {}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Unable to serialize".to_string()));
            format!("Failed to parse getcurrency response: {}", e)
        })?;
    
    // Convert to NamespaceOption format
    let namespace_option = NamespaceOption {
        name: root_currency.name.clone(),
        currency_id: root_currency.currencyid,
        registration_fee: root_currency.idregistrationfees,
        fully_qualified_name: root_currency.name.clone(),
        fee_currency_name: root_currency.name.clone(), // Root currency fees are paid in itself
        options: 41, // Assume root currencies support referral system
        id_referral_levels: root_currency.idreferrallevels,
    };
    
    println!("Root currency created: {} (fee: {} {})", 
        namespace_option.name, 
        namespace_option.registration_fee, 
        namespace_option.fee_currency_name);
    
    Ok(namespace_option)
}

async fn resolve_namespace_fee_currency(
    currency_info: CurrencyInfo,
    rpc_user: &str,
    rpc_pass: &str,
    rpc_port: u16,
) -> Result<NamespaceOption, String> {
    let def = &currency_info.currencydefinition;
    
    println!("  Resolving fee currency for namespace: {}", def.name);
    
    // Call getcurrency to get currency names mapping
    let response: Value = make_rpc_call(
        rpc_user,
        rpc_pass,
        rpc_port,
        "getcurrency",
        vec![json!(def.currencyid)],
    ).await
        .map_err(|e| {
            println!("  ✗ RPC call failed for {}: {}", def.name, e);
            format!("Failed to call getcurrency for {}: {}", def.name, e)
        })?;
    
    println!("  ✓ Got getcurrency response for {}", def.name);
    
    let currency_details: GetCurrencyResponse = serde_json::from_value::<GetCurrencyResponse>(response.clone())
        .map_err(|e| {
            println!("  ✗ Failed to parse getcurrency response for {}: {}", def.name, e);
            println!("  Response sample: {}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Unable to serialize".to_string()));
            format!("Failed to parse getcurrency response for {}: {}", def.name, e)
        })?;
    
    // Determine fee currency based on idimportfees
    println!("  Determining fee currency for {} (idimportfees: {})", def.name, def.idimportfees);
    
    // Check if idimportfees is one of the special reserve index values (0.00000000 - 0.00000009)
    let reserve_index_opt = match def.idimportfees {
        x if (x * 100000000.0).round() as i32 >= 0 && (x * 100000000.0).round() as i32 <= 9 => {
            let index = (x * 100000000.0).round() as usize;
            if index <= 9 { Some(index) } else { None }
        }
        _ => None
    };
    
    let fee_currency_name = if let Some(reserve_index) = reserve_index_opt {
        // Special case: fee is in one of the reserve currencies (index 0-9)
        println!("  Reserve fee case: idimportfees {} -> reserve index {}", def.idimportfees, reserve_index);
        
        if let Some(currency_names) = &currency_details.currencynames {
            println!("  Found currency names mapping with {} entries", currency_names.len());
            
            if let Some(reserve_currencies) = &currency_details.bestcurrencystate {
                if let Some(reserves) = &reserve_currencies.reservecurrencies {
                    let reserve_count = reserves.len();
                    println!("  Found {} reserve currencies", reserve_count);
                    
                    if reserve_index < reserve_count {
                        let reserve_currency_id = &reserves[reserve_index].currencyid;
                        println!("  Looking up reserve currency ID: {}", reserve_currency_id);
                        
                        let currency_name = currency_names.get(reserve_currency_id)
                            .unwrap_or(&format!("Unknown_{}", reserve_index))
                            .clone();
                        
                        println!("  ✓ Resolved to currency: {}", currency_name);
                        currency_name
                    } else {
                        println!("  ✗ Invalid reserve index {} (only {} reserves available)", reserve_index, reserve_count);
                        format!("InvalidIndex_{}", reserve_index)
                    }
                } else {
                    println!("  ✗ No reservecurrencies found in bestcurrencystate");
                    "NoReserves".to_string()
                }
            } else {
                println!("  ✗ No bestcurrencystate found in getcurrency response");
                "UnknownReserve".to_string()
            }
        } else {
            println!("  ✗ No currencynames found in getcurrency response");
            "UnknownCurrency".to_string()
        }
    } else {
        // Default case: fee is in the namespace's own currency
        println!("  ✓ Default fee case: using namespace currency '{}' (idimportfees: {})", def.name, def.idimportfees);
        def.name.clone()
    };
    
    println!("  ✓ Final fee currency name: {}", fee_currency_name);
    
    Ok(NamespaceOption {
        name: def.name.clone(),
        currency_id: def.currencyid.clone(),
        registration_fee: def.idregistrationfees,
        fully_qualified_name: def.fullyqualifiedname.clone(),
        fee_currency_name,
        options: def.options,
        id_referral_levels: def.idreferrallevels,
    })
} 

// Tauri command to get currency details including reserves
#[tauri::command]
pub async fn get_currency(
    app: tauri::AppHandle,
    currencyname: String,
) -> Result<GetCurrencyResponse, String> {
    println!("Getting currency details for: {}", currencyname);
    
    // Load credentials
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;
    
    // Call getcurrency RPC method
    let response: Value = make_rpc_call(
        &creds.rpc_user,
        &creds.rpc_pass,
        creds.rpc_port,
        "getcurrency",
        vec![json!(currencyname)],
    ).await
        .map_err(|e| format!("Failed to call getcurrency: {}", e))?;
    
    println!("Got getcurrency response for {}", currencyname);
    
    // Parse the response
    let currency_details: GetCurrencyResponse = serde_json::from_value::<GetCurrencyResponse>(response.clone())
        .map_err(|e| {
            println!("Failed to parse getcurrency response: {}", e);
            println!("Response: {}", serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Unable to serialize".to_string()));
            format!("Failed to parse getcurrency response: {}", e)
        })?;
    
    println!("Successfully parsed currency details for {}", currencyname);
    Ok(currency_details)
} 