// File: src-tauri/src/lib.rs
// Description: Main library file for the Tauri application backend.
// Changes:
// - Added credentials module and commands.
// - Added verus_rpc module.
// - Added thiserror dependency for error handling.
// - Added connect_verus_daemon command to interact with Verus RPC.
// - Registered tauri-plugin-secure-store.
// - Updated invoke_handler to include all commands.
// - Added send_private_message command.
// - Added settings module and persistence commands.
// - Added `pub mod rpc_client;` to declare the new module for RPC client logic.
// - Added `pub mod identity_rpc;` to declare the new module for identity RPC logic.
// - Added `pub mod message_rpc;` to declare the new module for message RPC logic.
// - Added `pub mod wallet_rpc;` to declare the new module for wallet RPC logic.
// - Added `pub mod namespace_rpc;` to declare the new module for namespace/currency RPC logic.
// - Corrected use statements and function call paths to reflect new module structure.
// - SECURITY: Integrated cryptographic message signing and verification system
// - Zero-trust messaging: Only verified messages are displayed, unverified messages are silently filtered
// - MAJOR: Added parallel blockchain detection commands for automatic onboarding
// - Added macOS window customization with almost black background for native titlebar appearance
// - Added get_utxo_info command for Fast Messages feature
// - Added progressive loading commands: get_login_identities_fast, get_identity_balance

mod credentials; // Added credentials module
mod settings; // Added settings module
pub mod rpc_client;
pub mod identity_rpc;
pub mod message_rpc;
pub mod wallet_rpc;
pub mod namespace_rpc;

use crate::rpc_client::VerusRpcError; // Corrected
use crate::credentials::CredentialError; // Import credential error
use crate::settings::SettingsError; // Import settings error
use crate::identity_rpc::FormattedIdentity; // Corrected
use crate::message_rpc::ChatMessage; // Corrected
use crate::wallet_rpc::UtxoInfo; // Import UtxoInfo struct

// Custom error type serializable for Tauri
#[derive(Debug, serde::Serialize, thiserror::Error)]
enum CommandError {
    #[error("Verus RPC Error: {0}")]
    Rpc(String),
    #[error("Credential Error: {0}")] // Added Credential error variant
    Credentials(String),
    #[error("Settings Error: {0}")] // Added Settings error variant
    Settings(String),
    #[error("Verus RPC Error: {0}")] // Use the same variant, but handle specific RPC errors
    RpcSpecific(crate::rpc_client::VerusRpcError), // Corrected
}

// Convert VerusRpcError to CommandError
impl From<crate::rpc_client::VerusRpcError> for CommandError { // Corrected
    fn from(error: crate::rpc_client::VerusRpcError) -> Self { // Corrected
        log::error!("RPC call failed: {:?}", error);
        // Return the specific error type for frontend handling
        CommandError::RpcSpecific(error)
    }
}

// Convert CredentialError to CommandError
impl From<CredentialError> for CommandError {
    fn from(error: CredentialError) -> Self {
        log::error!("Credential operation failed: {:?}", error);
        // Avoid leaking potentially sensitive details from StoreError
        match error {
            CredentialError::Store(_) => CommandError::Credentials("Failed to access store.".to_string()),
            _ => CommandError::Credentials(error.to_string()),
        }
    }
}

// Convert SettingsError to CommandError
impl From<SettingsError> for CommandError {
    fn from(error: SettingsError) -> Self {
        log::error!("Settings operation failed: {:?}", error);
        // Avoid leaking potentially sensitive details from StoreError
        match error {
            SettingsError::Store(_) => CommandError::Settings("Failed to access settings store.".to_string()),
            _ => CommandError::Settings(error.to_string()),
        }
    }
}

// macOS window customization function
#[cfg(target_os = "macos")]
fn set_macos_window_background(window: &tauri::WebviewWindow) {
    use cocoa::appkit::{NSColor, NSWindow};
    use cocoa::base::{id, nil};

    let ns_window = window.ns_window().unwrap() as id;
    unsafe {
        // Very dark color #0a0a0a (RGB: 10, 10, 10) - normalized to 0.0-1.0
        let bg_color = NSColor::colorWithRed_green_blue_alpha_(
            nil,
            10.0 / 255.0,   // Red: 10 (#0a)
            10.0 / 255.0,   // Green: 10 (#0a)
            10.0 / 255.0,   // Blue: 10 (#0a)
            1.0,            // Alpha: 100%
        );
        ns_window.setBackgroundColor_(bg_color);
        
        log::info!("macOS window background set to #0a0a0a (10, 10, 10)");
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn connect_verus_daemon(rpc_user: String, rpc_pass: String, rpc_port: u16) -> Result<u64, CommandError> {
    // Ensure logging is initialized (can be done once at startup too)
    // TODO: Initialize logger properly in main/run function
    let _ = env_logger::try_init();

    log::info!("connect_verus_daemon command received");
    crate::wallet_rpc::connect_and_get_block_height(rpc_user, rpc_pass, rpc_port) // Corrected path
        .await
        .map_err(CommandError::from)
}

// New command to get formatted identities (fast mode - no balances)
#[tauri::command]
async fn get_login_identities_fast(
    app: tauri::AppHandle, // Need AppHandle to get stored credentials
) -> Result<Vec<FormattedIdentity>, CommandError> {
    log::info!("get_login_identities_fast command received");
    // Load credentials first
    let creds = crate::credentials::load_credentials(app).await?;
    // Then call the RPC function
    crate::identity_rpc::get_login_identities_fast(creds.rpc_user, creds.rpc_pass, creds.rpc_port)
        .await
        .map_err(CommandError::from)
}

// New command to get formatted identities (with balances - full mode)
#[tauri::command]
async fn get_login_identities(
    app: tauri::AppHandle, // Need AppHandle to get stored credentials
) -> Result<Vec<FormattedIdentity>, CommandError> {
    log::info!("get_login_identities command received");
    // Load credentials first
    let creds = crate::credentials::load_credentials(app).await?;
    // Then call the RPC function
    crate::identity_rpc::get_login_identities(creds.rpc_user, creds.rpc_pass, creds.rpc_port) // Corrected path
        .await
        .map_err(CommandError::from)
}

// NEW command to get balance for a specific identity
#[tauri::command]
async fn get_identity_balance(
    app: tauri::AppHandle, // Need AppHandle for credentials
    private_address: String,
) -> Result<f64, CommandError> {
    log::info!("get_identity_balance command received for address: {}", private_address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::identity_rpc::get_identity_balance(creds.rpc_user, creds.rpc_pass, creds.rpc_port, private_address)
        .await
        .map_err(CommandError::from)
}

// NEW command to get private balance
#[tauri::command]
async fn get_private_balance(
    app: tauri::AppHandle, // Need AppHandle for credentials
    address: String,
) -> Result<f64, CommandError> {
    log::info!("get_private_balance command received for address: {}", address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::wallet_rpc::get_private_balance(creds.rpc_user, creds.rpc_pass, creds.rpc_port, address) // Correct path
        .await
        .map_err(CommandError::from)
}

// NEW command to get pending balance (0 confirmations)
#[tauri::command]
async fn get_pending_balance(
    app: tauri::AppHandle,
    address: String,
) -> Result<f64, CommandError> {
    log::info!("get_pending_balance command received for address: {}", address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::wallet_rpc::get_pending_balance(creds.rpc_user, creds.rpc_pass, creds.rpc_port, address)
        .await
        .map_err(CommandError::from)
}

// NEW Command: Check Identity Eligibility
#[tauri::command]
async fn check_identity_eligibility(
    app: tauri::AppHandle,
    target_identity_name: String,
) -> Result<FormattedIdentity, CommandError> {
    log::info!("check_identity_eligibility command received for: {}", target_identity_name);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::identity_rpc::check_identity_eligibility(creds.rpc_user, creds.rpc_pass, creds.rpc_port, target_identity_name) // Corrected path
        .await
        .map_err(CommandError::from) // Uses the updated From implementation
}

// NEW Command: Get Chat History (with automatic signature verification)
#[tauri::command]
async fn get_chat_history(
    app: tauri::AppHandle,
    target_identity_name: String,
    own_private_address: String,
) -> Result<Vec<ChatMessage>, CommandError> {
    log::info!("get_chat_history command received from: {} for owner: {}", target_identity_name, own_private_address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::message_rpc::get_chat_history(creds.rpc_user, creds.rpc_pass, creds.rpc_port, target_identity_name, own_private_address) // Corrected path
        .await
        .map_err(CommandError::from)
}

// NEW Command: Get New Received Messages (Polling) (with automatic signature verification)
#[tauri::command]
async fn get_new_received_messages(
    app: tauri::AppHandle,
    own_private_address: String,
) -> Result<Vec<ChatMessage>, CommandError> {
    log::info!("get_new_received_messages command received for owner: {}", own_private_address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::message_rpc::get_new_received_messages(creds.rpc_user, creds.rpc_pass, creds.rpc_port, own_private_address) // Corrected path
        .await
        .map_err(CommandError::from)
}

// NEW Command: Send Private Message/Gift (with mandatory signature)
#[tauri::command]
async fn send_private_message(
    app: tauri::AppHandle,
    sender_z_address: String,
    recipient_z_address: String,
    memo_text: String,
    sender_identity: String,
    amount: f64,
) -> Result<String, CommandError> { // Returns txid
    log::info!(
        "send_private_message command received: to={}, amount={}, sender_id={}",
        recipient_z_address,
        amount,
        sender_identity
    );
    let creds = crate::credentials::load_credentials(app).await?;
    crate::message_rpc::send_private_message( // Corrected path
        creds.rpc_user,
        creds.rpc_pass,
        creds.rpc_port,
        sender_z_address,
        recipient_z_address,
        memo_text,
        sender_identity,
        amount,
    )
    .await
    .map_err(CommandError::from)
}

// NEW command to get UTXO info for Fast Messages
#[tauri::command]
async fn get_utxo_info(
    app: tauri::AppHandle,
    address: String,
) -> Result<UtxoInfo, CommandError> {
    log::info!("get_utxo_info command received for address: {}", address);
    let creds = crate::credentials::load_credentials(app).await?;
    crate::wallet_rpc::get_utxo_info(creds.rpc_user, creds.rpc_pass, creds.rpc_port, address)
        .await
        .map_err(CommandError::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: Initialize logger here instead of in command
    env_logger::init(); // Basic logger initialization

    let store_plugin = tauri_plugin_store::Builder::default().build(); // Build the store plugin instance

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(store_plugin) // Register the store plugin instance
        .setup(|app| {
            log::info!("Setting up Tauri application");
            
            // Create the main window programmatically for all platforms
            use tauri::{WebviewUrl, WebviewWindowBuilder};
            
            #[cfg(target_os = "macos")]
            {
                use tauri::TitleBarStyle;
                
                let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                    .title("Nymia")
                    .inner_size(900.0, 600.0)
                    .visible(true)
                    .resizable(true)
                    .title_bar_style(TitleBarStyle::Transparent)
                    .hidden_title(true)
                    .accept_first_mouse(true);
                
                let window = win_builder.build()?;
                
                // Set custom almost black background
                set_macos_window_background(&window);
                
                log::info!("macOS window created with transparent titlebar and custom background");
            }
            
            // For other platforms, create a standard window
            #[cfg(not(target_os = "macos"))]
            {
                let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
                    .title("Nymia")
                    .inner_size(900.0, 600.0)
                    .visible(true)
                    .resizable(true);
                
                let _window = win_builder.build()?;
                
                log::info!("Standard window created for non-macOS platform");
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_verus_daemon,
            crate::credentials::save_credentials, // Add credential commands
            crate::credentials::load_credentials,
            crate::credentials::clear_credentials,
            crate::credentials::detect_all_blockchains, // NEW: Parallel detection
            crate::credentials::select_folder_dialog, // NEW: Folder selection
            crate::credentials::detect_blockchain_from_path, // NEW: Custom path detection
            get_login_identities_fast, // NEW: Fast loading without balances
            get_login_identities, // Correct name used here
            get_identity_balance, // NEW: Individual balance fetching
            get_private_balance, // Add the new balance command
            get_pending_balance, // Add the new pending balance command
            check_identity_eligibility,
            get_chat_history,
            get_new_received_messages,
            send_private_message, // Added send message command
            // New Settings Commands
            crate::settings::save_persistence_setting,
            crate::settings::load_persistence_setting,
            crate::settings::save_conversations,
            crate::settings::load_conversations,
            crate::settings::save_messages_for_conversation,
            crate::settings::load_messages_for_conversation,
            crate::settings::delete_chat_data,
            get_utxo_info,
            // Namespace commands
            crate::namespace_rpc::get_available_namespaces,
            crate::namespace_rpc::get_root_currency,
            // Currency conversion commands
            crate::wallet_rpc::estimate_currency_conversion
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
