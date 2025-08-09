// File: src-tauri/src/wallet_rpc.rs
// Description: Handles general wallet-related RPC calls including currency conversion estimates.
// Changes:
// - Moved connect_and_get_block_height and get_private_balance functions from verus_rpc.rs.
// - Added necessary use statements for rpc_client and serde_json.
// - Added UtxoInfo struct and get_utxo_info function for Fast Messages feature
// - Implemented z_listunspent RPC call with UTXO filtering and processing
// - Added EstimateConversionRequest/Response structures and estimate_conversion function for USD pricing
// - Added get_wallet_info function and command to get wallet balances and reserve balances
// - Added currency conversion commands: get_wallet_addresses, get_address_currency_balances, send_currency_conversion

use serde_json::{json, Value};
use super::rpc_client::{make_rpc_call, VerusRpcError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

// UTXO information structure for Fast Messages feature
#[derive(Debug, Serialize, Deserialize)]
pub struct UtxoInfo {
    pub total_utxos: u32,           // Total count including dust
    pub usable_utxos: u32,          // Count with amount >= 0.0001 (Fast Messages count)
    pub total_spendable_value: f64, // Sum of usable UTXOs only
    pub largest_utxo: f64,          // Largest single UTXO amount
    pub smallest_utxo: f64,         // Smallest usable UTXO amount (>= 0.0001)
}

// Request structure for estimateconversion
#[derive(Debug, Serialize, Deserialize)]
pub struct EstimateConversionRequest {
    pub currency: String,
    pub convertto: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    pub amount: f64,
}

// Response structure for estimateconversion (simplified to get the key field)
#[derive(Debug, Serialize, Deserialize)]
pub struct EstimateConversionResponse {
    #[serde(rename = "estimatedcurrencyout")]
    pub estimated_currency_out: f64,
    // We can add other fields later if needed
}

// Wallet info structure for payment details step
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub balance: f64,
    pub unconfirmed_balance: f64,
    pub reserve_balance: std::collections::HashMap<String, f64>,
    pub paytxfee: f64,
}

// Function to connect and get block height
// Exposed as a Tauri command
pub async fn connect_and_get_block_height(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
) -> Result<u64, VerusRpcError> {
    log::info!("Attempting to connect to Verus daemon...");
    make_rpc_call(&rpc_user, &rpc_pass, rpc_port, "getblockcount", vec![]).await
}

// Function to get balance for a z-address
pub async fn get_private_balance(rpc_user: String, rpc_pass: String, rpc_port: u16, address: String) -> Result<f64, VerusRpcError> {
    log::info!("Fetching private balance for address: {}", address);
    make_rpc_call(&rpc_user, &rpc_pass, rpc_port, "z_getbalance", vec![json!(address)]).await
}

// Function to get pending balance for a z-address (0 confirmations)
pub async fn get_pending_balance(rpc_user: String, rpc_pass: String, rpc_port: u16, address: String) -> Result<f64, VerusRpcError> {
    log::info!("Fetching pending balance for address: {}", address);
    make_rpc_call(&rpc_user, &rpc_pass, rpc_port, "z_getbalance", vec![json!(address), json!(0)]).await
}

// NEW function to get UTXO information for Fast Messages
pub async fn get_utxo_info(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    address: String,
) -> Result<UtxoInfo, VerusRpcError> {
    log::info!("Fetching UTXO info for address: {}", address);
    
    // Call z_listunspent with specific parameters:
    // minconf=1: Only confirmed UTXOs
    // maxconf=9999999: All confirmed UTXOs  
    // watchonly=false: Only spendable UTXOs
    // addresses=[address]: Only for this specific address
    let utxo_list: Value = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "z_listunspent",
        vec![json!(1), json!(9999999), json!(false), json!([address])],
    ).await?;

    log::debug!("Raw UTXO response: {:?}", utxo_list);

    // Process the UTXO list
    let utxos = utxo_list.as_array().ok_or(VerusRpcError::ParseError(
        "Expected array of UTXOs".to_string(),
    ))?;

    let mut total_utxos = 0u32;
    let mut usable_utxos = 0u32;
    let mut total_spendable_value = 0.0f64;
    let mut largest_utxo = 0.0f64;
    let mut smallest_utxo = f64::MAX;

    for utxo in utxos {
        let amount = utxo["amount"].as_f64().unwrap_or(0.0);
        total_utxos += 1;

        // Track largest UTXO regardless of usability
        if amount > largest_utxo {
            largest_utxo = amount;
        }

        // Filter for usable UTXOs (amount >= 0.0001)
        if amount >= 0.0001 {
            usable_utxos += 1;
            total_spendable_value += amount;
            
            // Track smallest usable UTXO
            if amount < smallest_utxo {
                smallest_utxo = amount;
            }
        }
    }

    // If no usable UTXOs, set smallest to 0
    if usable_utxos == 0 {
        smallest_utxo = 0.0;
    }

    let utxo_info = UtxoInfo {
        total_utxos,
        usable_utxos,
        total_spendable_value,
        largest_utxo,
        smallest_utxo,
    };

    log::info!(
        "UTXO analysis complete: {} total UTXOs, {} usable UTXOs, {:.4} total spendable, largest: {:.4}, smallest: {:.4}",
        utxo_info.total_utxos,
        utxo_info.usable_utxos,
        utxo_info.total_spendable_value,
        utxo_info.largest_utxo,
        utxo_info.smallest_utxo
    );

    Ok(utxo_info)
}

// NEW function to estimate currency conversion
pub async fn estimate_conversion(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    request: EstimateConversionRequest,
) -> Result<f64, VerusRpcError> {
    log::info!(
        "Estimating conversion: {} {} to {} {}",
        request.amount,
        request.currency,
        request.convertto,
        request.via.as_ref().map_or("".to_string(), |v| format!("via {}", v))
    );

    // Build the JSON request for the RPC call
    let mut conversion_params = json!({
        "currency": request.currency,
        "convertto": request.convertto,
        "amount": request.amount
    });

    // Add 'via' parameter if provided
    if let Some(via) = request.via {
        conversion_params["via"] = json!(via);
    }

    // Make the RPC call
    let response: Value = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "estimateconversion",
        vec![conversion_params],
    ).await?;

    log::debug!("Raw estimateconversion response: {:?}", response);

    // Extract the estimated currency out value
    let estimated_out = response["estimatedcurrencyout"]
        .as_f64()
        .ok_or(VerusRpcError::ParseError(
            "Missing or invalid 'estimatedcurrencyout' in response".to_string(),
        ))?;

    log::info!(
        "Conversion estimate: {} {} = {} {}",
        request.amount,
        request.currency,
        estimated_out,
        request.convertto
    );

    Ok(estimated_out)
}

// NEW function to get wallet info including balances and reserves
pub async fn fetch_wallet_info(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
) -> Result<WalletInfo, VerusRpcError> {
    log::info!("Fetching wallet info including balances and reserves...");
    
    // Call getwalletinfo RPC method
    let response: Value = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "getwalletinfo",
        vec![],
    ).await?;

    log::debug!("Raw getwalletinfo response: {:?}", response);

    // Extract required fields from the response
    let balance = response["balance"]
        .as_f64()
        .unwrap_or(0.0);
    
    let unconfirmed_balance = response["unconfirmed_balance"]
        .as_f64()
        .unwrap_or(0.0);
    
    let paytxfee = response["paytxfee"]
        .as_f64()
        .unwrap_or(0.0001); // Default fee if not specified
    
    // Extract reserve balances
    let mut reserve_balance = std::collections::HashMap::new();
    if let Some(reserves) = response["reserve_balance"].as_object() {
        for (currency, amount) in reserves {
            if let Some(amount_f64) = amount.as_f64() {
                reserve_balance.insert(currency.clone(), amount_f64);
            }
        }
    }

    let wallet_info = WalletInfo {
        balance,
        unconfirmed_balance,
        reserve_balance,
        paytxfee,
    };

    log::info!(
        "Wallet info fetched: balance={:.8}, reserves={}, paytxfee={:.8}",
        wallet_info.balance,
        wallet_info.reserve_balance.len(),
        wallet_info.paytxfee
    );

    Ok(wallet_info)
}

// NEW function to get all wallet addresses
pub async fn fetch_wallet_addresses(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
) -> Result<Vec<String>, VerusRpcError> {
    log::info!("Fetching wallet addresses...");
    
    // Call getaddressesbyaccount with empty string (default account)
    let addresses: Vec<String> = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "getaddressesbyaccount",
        vec![json!("")],
    ).await?;

    log::info!("Found {} wallet addresses", addresses.len());
    log::debug!("Wallet addresses: {:?}", addresses);

    Ok(addresses)
}

// NEW function to get currency balances for a specific address
pub async fn fetch_address_currency_balances(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    address: String,
) -> Result<HashMap<String, f64>, VerusRpcError> {
    log::info!("Fetching currency balances for address: {}", address);
    
    // Call getcurrencybalance for the address
    let balances: Value = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "getcurrencybalance",
        vec![json!(address)],
    ).await?;

    log::debug!("Raw currency balance response: {:?}", balances);

    // Parse the response into a HashMap
    let mut balance_map = HashMap::new();
    if let Some(balances_obj) = balances.as_object() {
        for (currency, amount) in balances_obj {
            if let Some(amount_f64) = amount.as_f64() {
                balance_map.insert(currency.clone(), amount_f64);
            }
        }
    }

    log::info!("Found {} currencies for address {}: {:?}", 
               balance_map.len(), address, balance_map);

    Ok(balance_map)
}

// NEW function to initiate currency conversion
pub async fn initiate_currency_conversion(
    rpc_user: String,
    rpc_pass: String,
    rpc_port: u16,
    from_address: String,
    to_address: String,
    from_currency: String,
    to_currency: String,
    amount: f64,
) -> Result<String, VerusRpcError> {
    log::info!(
        "Initiating currency conversion: {} {} from {} to {} at {}",
        amount, from_currency, from_address, to_address, to_currency
    );

    // Round amount to 8 decimal places to avoid RPC errors with high precision floats
    let rounded_amount = (amount * 100_000_000.0).round() / 100_000_000.0;

    // Build the sendcurrency parameters as a direct JSON object.
    let amounts_param = json!([{
        "address": to_address,
        "currency": from_currency,
        "amount": rounded_amount,
        "convertto": to_currency
    }]);

    let params = vec![
        json!(from_address), // Can be "*" for wildcard
        amounts_param,       // Pass the JSON array directly
    ];

    log::debug!("sendcurrency params: {:?}", params);

    // Make the RPC call
    let txid: String = make_rpc_call(
        &rpc_user,
        &rpc_pass,
        rpc_port,
        "sendcurrency",
        params,
    ).await?;

    log::info!("Currency conversion initiated successfully, txid: {}", txid);

    Ok(txid)
}

// Tauri command wrapper for estimate_conversion
#[tauri::command]
pub async fn estimate_currency_conversion(
    app: tauri::AppHandle,
    currency: String,
    convert_to: String,
    via: Option<String>,
    amount: f64,
) -> Result<f64, String> {
    // Load credentials
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    let request = EstimateConversionRequest {
        currency,
        convertto: convert_to,
        via,
        amount,
    };

    estimate_conversion(creds.rpc_user, creds.rpc_pass, creds.rpc_port, request)
        .await
        .map_err(|e| format!("Conversion estimate failed: {}", e))
} 

// Tauri command wrapper for get_wallet_info
#[tauri::command]
pub async fn get_wallet_info(
    app: tauri::AppHandle,
) -> Result<WalletInfo, String> {
    // Load credentials
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    fetch_wallet_info(creds.rpc_user, creds.rpc_pass, creds.rpc_port)
        .await
        .map_err(|e| format!("Failed to get wallet info: {}", e))
} 

// NEW Tauri command to get wallet addresses
#[tauri::command]
pub async fn get_wallet_addresses(
    app: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    fetch_wallet_addresses(creds.rpc_user, creds.rpc_pass, creds.rpc_port)
        .await
        .map_err(|e| format!("Failed to get wallet addresses: {}", e))
}

// NEW Tauri command to get address currency balances
#[tauri::command]
pub async fn get_address_currency_balances(
    app: tauri::AppHandle,
    address: String,
) -> Result<HashMap<String, f64>, String> {
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    fetch_address_currency_balances(creds.rpc_user, creds.rpc_pass, creds.rpc_port, address)
        .await
        .map_err(|e| format!("Failed to get address currency balances: {}", e))
}

// NEW Tauri command to get balance of a specific currency for a specific address
#[tauri::command]
pub async fn get_address_currency_balance(
    app: tauri::AppHandle,
    address: String,
    currency: String,
) -> Result<f64, String> {
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    let balances = fetch_address_currency_balances(creds.rpc_user, creds.rpc_pass, creds.rpc_port, address)
        .await
        .map_err(|e| format!("Failed to get address currency balances: {}", e))?;

    Ok(balances.get(&currency).copied().unwrap_or(0.0))
}

// NEW Tauri command to send currency conversion
#[tauri::command]
pub async fn send_currency_conversion(
    app: tauri::AppHandle,
    from_address: String,
    to_address: String,
    from_currency: String,
    to_currency: String,
    amount: f64,
) -> Result<String, String> {
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    initiate_currency_conversion(
        creds.rpc_user,
        creds.rpc_pass,
        creds.rpc_port,
        from_address,
        to_address,
        from_currency,
        to_currency,
        amount
    )
    .await
    .map_err(|e| format!("Failed to send currency conversion: {}", e))
}

// NEW Tauri command to get current block height  
#[tauri::command]
pub async fn get_current_block_height(
    app: tauri::AppHandle,
) -> Result<u64, String> {
    let creds = crate::credentials::load_credentials(app).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    connect_and_get_block_height(creds.rpc_user, creds.rpc_pass, creds.rpc_port)
        .await
        .map_err(|e| format!("Failed to get current block height: {}", e))
} 

// NEW: Wait for block height to increase by N blocks (polling)
#[tauri::command]
pub async fn wait_for_block_increase(
    app: tauri::AppHandle,
    blocks: u64,
    interval_secs: u64,
    timeout_secs: u64,
) -> Result<bool, String> {
    let creds = crate::credentials::load_credentials(app.clone()).await
        .map_err(|e| format!("Failed to load credentials: {}", e))?;

    log::info!(
        "wait_for_block_increase: blocks={}, interval={}s, timeout={}s",
        blocks, interval_secs, timeout_secs
    );

    let start_height: u64 = connect_and_get_block_height(
        creds.rpc_user.clone(),
        creds.rpc_pass.clone(),
        creds.rpc_port,
    )
    .await
    .map_err(|e| format!("Failed to get start height: {}", e))?;

    log::info!("wait_for_block_increase: start_height={}", start_height);

    let target_height = start_height.saturating_add(blocks);
    let start_time = std::time::Instant::now();

    loop {
        let height = connect_and_get_block_height(
            creds.rpc_user.clone(),
            creds.rpc_pass.clone(),
            creds.rpc_port,
        )
        .await
        .map_err(|e| format!("Failed to poll height: {}", e))?;

        log::debug!("wait_for_block_increase: current_height={}, target_height={}", height, target_height);

        if height >= target_height {
            log::info!("wait_for_block_increase: reached target height {}", height);
            return Ok(true);
        }

        if start_time.elapsed() >= Duration::from_secs(timeout_secs) {
            log::warn!(
                "wait_for_block_increase: timeout after {}s, current_height={}, target_height={}",
                timeout_secs,
                height,
                target_height
            );
            return Ok(false);
        }

        sleep(Duration::from_secs(interval_secs)).await;
    }
}