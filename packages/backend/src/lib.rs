mod bindings;
mod commands;
mod commands_import;
mod import;
mod mcp_protocol;
mod mcps;
mod models;
mod projects;
mod router;
mod router_mcp;
mod secrets;
mod storage;

use router::start_router;
use secrets::{get_or_create_key, SecretManager};
use storage::Storage;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir).unwrap();
            let db_path = app_dir.join("mcp_manager.db");

            let storage = Arc::new(Storage::new(db_path.to_str().unwrap()).unwrap());
            let key = get_or_create_key();
            let secret_manager = Arc::new(SecretManager::new(&key));

            app.manage(storage.clone());
            app.manage(secret_manager.clone());

            let storage_clone = storage.clone();
            let secret_manager_clone = secret_manager.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = start_router(storage_clone, secret_manager_clone).await {
                    eprintln!("Router error: {}", e);
                }
            });

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
            commands::get_recent_logs,
            commands::set_active_project,
            commands::generate_mcp_config,
            commands_import::parse_mcp_json_command,
            commands_import::import_mcps_from_json,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
