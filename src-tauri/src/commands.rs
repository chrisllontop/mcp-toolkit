use crate::bindings::BindingManager;
use crate::mcps::McpManager;
use crate::models::*;
use crate::projects::ProjectManager;
use crate::secrets::SecretManager;
use crate::storage::Storage;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tauri::command]
pub async fn create_project(
    name: String,
    path: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Project, String> {
    let manager = ProjectManager::new(&storage);
    manager.create_project(name, path)
}

#[tauri::command]
pub async fn list_projects(storage: tauri::State<'_, Arc<Storage>>) -> Result<Vec<Project>, String> {
    let manager = ProjectManager::new(&storage);
    manager.list_projects()
}

#[tauri::command]
pub async fn detect_ai_config(
    project_path: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Option<String>, String> {
    let manager = ProjectManager::new(&storage);
    Ok(manager.detect_ai_config(&project_path))
}

#[tauri::command]
pub async fn delete_project(
    id: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<(), String> {
    let manager = ProjectManager::new(&storage);
    manager.delete_project(id)
}

#[tauri::command]
pub async fn create_mcp(
    name: String,
    mcp_type: McpType,
    config: McpConfig,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Mcp, String> {
    let manager = McpManager::new(&storage);
    manager.create_mcp(name, mcp_type, config)
}

#[tauri::command]
pub async fn list_mcps(storage: tauri::State<'_, Arc<Storage>>) -> Result<Vec<Mcp>, String> {
    let manager = McpManager::new(&storage);
    manager.list_mcps()
}

#[tauri::command]
pub async fn update_mcp(
    mcp: Mcp,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<(), String> {
    let manager = McpManager::new(&storage);
    manager.update_mcp(mcp)
}

#[tauri::command]
pub async fn delete_mcp(
    id: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<(), String> {
    let manager = McpManager::new(&storage);
    manager.delete_mcp(id)
}

#[tauri::command]
pub async fn activate_mcp(
    project_id: String,
    mcp_id: String,
    overrides: Vec<EnvVar>,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<ProjectMcpBinding, String> {
    let manager = BindingManager::new(&storage);
    manager.activate_mcp(project_id, mcp_id, overrides)
}

#[tauri::command]
pub async fn list_bindings(
    project_id: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Vec<ProjectMcpBinding>, String> {
    let manager = BindingManager::new(&storage);
    manager.list_bindings(project_id)
}

#[tauri::command]
pub async fn update_binding(
    binding: ProjectMcpBinding,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<(), String> {
    let manager = BindingManager::new(&storage);
    manager.update_binding(binding)
}

#[tauri::command]
pub async fn save_secret(
    key: String,
    value: String,
    storage: tauri::State<'_, Arc<Storage>>,
    secret_manager: tauri::State<'_, Arc<SecretManager>>,
) -> Result<Secret, String> {
    let encrypted = secret_manager.encrypt(&value)?;
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();

    storage
        .insert_secret(&id, &key, &encrypted, &created_at)
        .map_err(|e| e.to_string())?;

    Ok(Secret {
        id,
        key,
        created_at,
    })
}

#[tauri::command]
pub async fn list_secrets(storage: tauri::State<'_, Arc<Storage>>) -> Result<Vec<Secret>, String> {
    storage.get_secrets().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_logs(
    limit: usize,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Vec<RouterLog>, String> {
    storage.get_recent_logs(limit).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_active_project(project_id: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    client
        .post("http://127.0.0.1:9876/project/set")
        .json(&serde_json::json!({"project_id": project_id}))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn generate_mcp_config(
    _project_id: String,
    _storage: tauri::State<'_, Arc<Storage>>,
) -> Result<String, String> {
    let config = serde_json::json!({
        "mcpServers": {
            "mcp-manager": {
                "type": "http",
                "url": "http://127.0.0.1:9876/mcp"
            }
        }
    });

    Ok(serde_json::to_string_pretty(&config).unwrap())
}
