use crate::import::parse_mcp_json;
use crate::mcps::McpManager;
use crate::models::*;
use crate::storage::Storage;
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct ImportPreview {
    pub name: String,
    pub mcp_type: McpType,
    pub command: Option<String>,
    pub args: Vec<String>,
}

#[tauri::command]
pub async fn parse_mcp_json_command(json_str: String) -> Result<Vec<ImportPreview>, String> {
    let parsed = parse_mcp_json(&json_str)?;

    let previews: Vec<ImportPreview> = parsed
        .into_iter()
        .map(|(name, config)| {
            let mcp_type = if config.docker_image.is_some() {
                McpType::Docker
            } else if config.http_url.is_some() {
                McpType::Http
            } else {
                McpType::Binary
            };

            ImportPreview {
                name,
                mcp_type,
                command: config.command.clone(),
                args: config.args.clone(),
            }
        })
        .collect();

    Ok(previews)
}

#[tauri::command]
pub async fn import_mcps_from_json(
    json_str: String,
    storage: tauri::State<'_, Arc<Storage>>,
) -> Result<Vec<Mcp>, String> {
    let parsed = parse_mcp_json(&json_str)?;
    let manager = McpManager::new(&storage);

    let mut imported = Vec::new();

    for (name, config) in parsed {
        let mcp_type = if config.docker_image.is_some() {
            McpType::Docker
        } else if config.http_url.is_some() {
            McpType::Http
        } else {
            McpType::Binary
        };

        let mcp = manager.create_mcp(name, mcp_type, config)?;
        imported.push(mcp);
    }

    Ok(imported)
}
