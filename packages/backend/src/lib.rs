mod bindings;
mod commands;
mod commands_import;
mod import;
mod mcps;
mod models;
mod projects;
mod secrets;
mod storage;
mod utils;

use secrets::{get_or_create_key, SecretManager};
use storage::Storage;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            // Use custom app directory: ~/Library/Application Support/MCP Toolkit
            let mut app_dir = dirs::data_local_dir().expect("Could not find data directory");
            app_dir.push("MCP Toolkit");
            std::fs::create_dir_all(&app_dir).unwrap();
            let db_path = app_dir.join("mcp_manager.db");

            let storage = Arc::new(Storage::new(db_path.to_str().unwrap()).unwrap());
            let key = get_or_create_key()
                .expect("Failed to initialize encryption key from OS keychain. Please ensure keychain access is available.");
            let secret_manager = Arc::new(SecretManager::new(&key));

            app.manage(storage.clone());
            app.manage(secret_manager.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::create_project,
            commands::list_projects,
            commands::detect_ai_config,
            commands::delete_project,
            commands::create_mcp,
            commands::list_mcps,
            commands::update_mcp,
            commands::delete_mcp,
            commands::activate_mcp,
            commands::list_bindings,
            commands::update_binding,
            commands::save_secret,
            commands::list_secrets,
            commands::generate_mcp_config,
            commands::copy_mcp_config,
            commands_import::parse_mcp_json_command,
            commands_import::import_mcps_from_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
