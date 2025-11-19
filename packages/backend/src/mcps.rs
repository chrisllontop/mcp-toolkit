use crate::models::*;
use crate::storage::Storage;
use chrono::Utc;
use uuid::Uuid;

pub struct McpManager<'a> {
    storage: &'a Storage,
}

impl<'a> McpManager<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        McpManager { storage }
    }

    pub fn create_mcp(&self, name: String, mcp_type: McpType, config: McpConfig) -> Result<Mcp, String> {
        let mcp = Mcp {
            id: Uuid::new_v4().to_string(),
            name,
            mcp_type,
            config,
            created_at: Utc::now().to_rfc3339(),
        };

        self.storage.insert_mcp(&mcp).map_err(|e| e.to_string())?;

        Ok(mcp)
    }

    pub fn list_mcps(&self) -> Result<Vec<Mcp>, String> {
        self.storage.get_mcps().map_err(|e| e.to_string())
    }

    pub fn update_mcp(&self, mcp: Mcp) -> Result<(), String> {
        self.storage.update_mcp(&mcp).map_err(|e| e.to_string())
    }

    pub fn delete_mcp(&self, id: String) -> Result<(), String> {
        self.storage.delete_mcp(&id).map_err(|e| e.to_string())
    }
}
