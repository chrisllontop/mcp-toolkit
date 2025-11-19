use crate::models::*;
use crate::storage::Storage;
use uuid::Uuid;

pub struct BindingManager<'a> {
    storage: &'a Storage,
}

impl<'a> BindingManager<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        BindingManager { storage }
    }

    pub fn activate_mcp(&self, project_id: String, mcp_id: String, overrides: Vec<EnvVar>) -> Result<ProjectMcpBinding, String> {
        let binding = ProjectMcpBinding {
            id: Uuid::new_v4().to_string(),
            project_id,
            mcp_id,
            enabled: true,
            overrides,
        };

        self.storage
            .insert_binding(&binding)
            .map_err(|e| e.to_string())?;

        Ok(binding)
    }

    pub fn list_bindings(&self, project_id: String) -> Result<Vec<ProjectMcpBinding>, String> {
        self.storage
            .get_bindings_by_project(&project_id)
            .map_err(|e| e.to_string())
    }

    pub fn update_binding(&self, binding: ProjectMcpBinding) -> Result<(), String> {
        self.storage
            .update_binding(&binding)
            .map_err(|e| e.to_string())
    }
}
