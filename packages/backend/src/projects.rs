use crate::models::*;
use crate::storage::Storage;
use chrono::Utc;
use std::path::Path;
use uuid::Uuid;

pub struct ProjectManager<'a> {
    storage: &'a Storage,
}

impl<'a> ProjectManager<'a> {
    pub fn new(storage: &'a Storage) -> Self {
        ProjectManager { storage }
    }

    pub fn create_project(&self, name: String, path: String) -> Result<Project, String> {
        if !Path::new(&path).exists() {
            return Err("Path does not exist".to_string());
        }

        let project = Project {
            id: Uuid::new_v4().to_string(),
            name,
            path,
            created_at: Utc::now().to_rfc3339(),
        };

        self.storage
            .insert_project(&project)
            .map_err(|e| e.to_string())?;

        Ok(project)
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, String> {
        self.storage.get_projects().map_err(|e| e.to_string())
    }

    pub fn delete_project(&self, id: String) -> Result<(), String> {
        self.storage.delete_project(&id).map_err(|e| e.to_string())
    }

    pub fn detect_ai_config(&self, project_path: &str) -> Option<String> {
        let config_files = vec![
            ".claude.json",
            ".cursor/config.json",
            ".ai/config.json",
            "mcp.json",
        ];

        for file in config_files {
            let config_path = Path::new(project_path).join(file);
            if config_path.exists() {
                return Some(file.to_string());
            }
        }

        None
    }
}
