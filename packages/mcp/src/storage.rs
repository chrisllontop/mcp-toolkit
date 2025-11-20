use crate::models::*;
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        let conn = Connection::open(&db_path)?;
        Ok(Storage { conn })
    }

    fn get_db_path() -> PathBuf {
        // Match Tauri's app_data_dir path: ~/Library/Application Support/com.mcp.manager/mcp_manager.db
        let mut path = dirs::data_local_dir().expect("Could not find data directory");
        path.push("com.mcp.manager");
        path.push("mcp_manager.db");
        path
    }

    /// Get all enabled MCPs with their bindings
    pub fn get_enabled_mcps_with_bindings(&self) -> Result<Vec<(Mcp, ProjectMcpBinding)>, String> {
        let mut stmt = self.conn
            .prepare(
                "SELECT
                    m.id, m.name, m.mcp_type, m.config, m.created_at,
                    b.id, b.project_id, b.mcp_id, b.enabled, b.overrides
                FROM mcps m
                INNER JOIN project_mcp_bindings b ON m.id = b.mcp_id
                WHERE b.enabled = 1"
            )
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let results = stmt
            .query_map([], |row| {
                // Parse MCP
                let mcp_type_str: String = row.get(2)?;
                let mcp_type = match mcp_type_str.as_str() {
                    "docker" => McpType::Docker,
                    "binary" => McpType::Binary,
                    "http" => McpType::Http,
                    _ => McpType::Binary,
                };
                let config_json: String = row.get(3)?;
                let config: McpConfig = serde_json::from_str(&config_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let mcp = Mcp {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    mcp_type,
                    config,
                    created_at: row.get(4)?,
                };

                // Parse Binding
                let overrides_json: String = row.get(9)?;
                let overrides: Vec<EnvVar> = serde_json::from_str(&overrides_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                let binding = ProjectMcpBinding {
                    id: row.get(5)?,
                    project_id: row.get(6)?,
                    mcp_id: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? != 0,
                    overrides,
                };

                Ok((mcp, binding))
            })
            .map_err(|e| format!("Failed to query mcps: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to collect mcps: {}", e))?;

        Ok(results)
    }

    /// Get encrypted secret by key
    pub fn get_encrypted_secret(&self, key: &str) -> Result<Option<String>, String> {
        let mut stmt = self.conn
            .prepare("SELECT encrypted_value FROM secrets WHERE key = ?1")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let mut rows = stmt
            .query(params![key])
            .map_err(|e| format!("Failed to query secrets: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| format!("Failed to get row: {}", e))? {
            let value: String = row.get(0).map_err(|e| format!("Failed to get value: {}", e))?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
