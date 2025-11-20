use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mcp {
    pub id: String,
    pub name: String,
    pub mcp_type: McpType,
    pub config: McpConfig,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum McpType {
    Docker,
    Binary,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    pub docker_image: Option<String>,
    pub binary_path: Option<String>,
    pub http_url: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub env_vars: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMcpBinding {
    pub id: String,
    pub project_id: String,
    pub mcp_id: String,
    pub enabled: bool,
    pub overrides: Vec<EnvVar>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub id: String,
    pub key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterLog {
    pub id: String,
    pub timestamp: String,
    pub project_id: String,
    pub mcp_id: String,
    pub tool_name: String,
    pub status: String,
    pub duration_ms: i64,
    pub error: Option<String>,
}
