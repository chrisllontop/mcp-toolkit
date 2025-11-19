use crate::models::*;
use rusqlite::{params, Connection, Result};
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>,
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Storage {
            conn: Mutex::new(conn),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mcps (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                mcp_type TEXT NOT NULL,
                config TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS project_mcp_bindings (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                mcp_id TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                overrides TEXT NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id),
                FOREIGN KEY(mcp_id) REFERENCES mcps(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS secrets (
                id TEXT PRIMARY KEY,
                key TEXT NOT NULL UNIQUE,
                encrypted_value TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS router_logs (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                project_id TEXT NOT NULL,
                mcp_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                status TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                error TEXT
            )",
            [],
        )?;

        Ok(())
    }

    pub fn insert_project(&self, project: &Project) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO projects (id, name, path, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![project.id, project.name, project.path, project.created_at],
        )?;
        Ok(())
    }

    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, path, created_at FROM projects")?;
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        projects.collect()
    }

    pub fn delete_project(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_mcp(&self, mcp: &Mcp) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let config_json = serde_json::to_string(&mcp.config).unwrap();
        let mcp_type_str = match mcp.mcp_type {
            McpType::Docker => "docker",
            McpType::Binary => "binary",
            McpType::Http => "http",
        };
        conn.execute(
            "INSERT INTO mcps (id, name, mcp_type, config, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![mcp.id, mcp.name, mcp_type_str, config_json, mcp.created_at],
        )?;
        Ok(())
    }

    pub fn get_mcps(&self) -> Result<Vec<Mcp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name, mcp_type, config, created_at FROM mcps")?;
        let mcps = stmt.query_map([], |row| {
            let mcp_type_str: String = row.get(2)?;
            let mcp_type = match mcp_type_str.as_str() {
                "docker" => McpType::Docker,
                "binary" => McpType::Binary,
                "http" => McpType::Http,
                _ => McpType::Binary,
            };
            let config_json: String = row.get(3)?;
            let config: McpConfig = serde_json::from_str(&config_json).unwrap();
            Ok(Mcp {
                id: row.get(0)?,
                name: row.get(1)?,
                mcp_type,
                config,
                created_at: row.get(4)?,
            })
        })?;
        mcps.collect()
    }

    pub fn update_mcp(&self, mcp: &Mcp) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let config_json = serde_json::to_string(&mcp.config).unwrap();
        let mcp_type_str = match mcp.mcp_type {
            McpType::Docker => "docker",
            McpType::Binary => "binary",
            McpType::Http => "http",
        };
        conn.execute(
            "UPDATE mcps SET name = ?1, mcp_type = ?2, config = ?3 WHERE id = ?4",
            params![mcp.name, mcp_type_str, config_json, mcp.id],
        )?;
        Ok(())
    }

    pub fn delete_mcp(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM mcps WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_binding(&self, binding: &ProjectMcpBinding) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let overrides_json = serde_json::to_string(&binding.overrides).unwrap();
        conn.execute(
            "INSERT INTO project_mcp_bindings (id, project_id, mcp_id, enabled, overrides) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![binding.id, binding.project_id, binding.mcp_id, binding.enabled as i32, overrides_json],
        )?;
        Ok(())
    }

    pub fn get_bindings_by_project(&self, project_id: &str) -> Result<Vec<ProjectMcpBinding>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, mcp_id, enabled, overrides FROM project_mcp_bindings WHERE project_id = ?1")?;
        let bindings = stmt.query_map(params![project_id], |row| {
            let overrides_json: String = row.get(4)?;
            let overrides: Vec<EnvVar> = serde_json::from_str(&overrides_json).unwrap();
            Ok(ProjectMcpBinding {
                id: row.get(0)?,
                project_id: row.get(1)?,
                mcp_id: row.get(2)?,
                enabled: row.get::<_, i32>(3)? != 0,
                overrides,
            })
        })?;
        bindings.collect()
    }

    pub fn update_binding(&self, binding: &ProjectMcpBinding) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let overrides_json = serde_json::to_string(&binding.overrides).unwrap();
        conn.execute(
            "UPDATE project_mcp_bindings SET enabled = ?1, overrides = ?2 WHERE id = ?3",
            params![binding.enabled as i32, overrides_json, binding.id],
        )?;
        Ok(())
    }

    pub fn insert_secret(&self, id: &str, key: &str, encrypted_value: &str, created_at: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO secrets (id, key, encrypted_value, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, key, encrypted_value, created_at],
        )?;
        Ok(())
    }

    pub fn get_secrets(&self) -> Result<Vec<Secret>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, key, created_at FROM secrets")?;
        let secrets = stmt.query_map([], |row| {
            Ok(Secret {
                id: row.get(0)?,
                key: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        secrets.collect()
    }

    pub fn get_encrypted_secret(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT encrypted_value FROM secrets WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn insert_log(&self, log: &RouterLog) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO router_logs (id, timestamp, project_id, mcp_id, tool_name, status, duration_ms, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![log.id, log.timestamp, log.project_id, log.mcp_id, log.tool_name, log.status, log.duration_ms, log.error],
        )?;
        Ok(())
    }

    pub fn get_recent_logs(&self, limit: usize) -> Result<Vec<RouterLog>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, timestamp, project_id, mcp_id, tool_name, status, duration_ms, error FROM router_logs ORDER BY timestamp DESC LIMIT ?1")?;
        let logs = stmt.query_map(params![limit], |row| {
            Ok(RouterLog {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                project_id: row.get(2)?,
                mcp_id: row.get(3)?,
                tool_name: row.get(4)?,
                status: row.get(5)?,
                duration_ms: row.get(6)?,
                error: row.get(7)?,
            })
        })?;
        logs.collect()
    }
}
