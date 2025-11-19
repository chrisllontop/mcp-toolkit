use crate::models::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct McpServersWrapper {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

pub fn parse_mcp_json(json_str: &str) -> Result<Vec<(String, McpConfig)>, String> {
    let wrapper: McpServersWrapper = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut results = Vec::new();

    for (name, config) in wrapper.mcp_servers {
        let mcp_type = detect_mcp_type(&config);
        let mcp_config = convert_to_mcp_config(config, &mcp_type);
        results.push((name, mcp_config));
    }

    Ok(results)
}

fn detect_mcp_type(config: &McpServerConfig) -> McpType {
    if config.command.starts_with("docker") || config.command.contains("docker") {
        McpType::Docker
    } else if config.command.starts_with("http") || config.command.starts_with("https") {
        McpType::Http
    } else {
        McpType::Binary
    }
}

fn convert_to_mcp_config(config: McpServerConfig, mcp_type: &McpType) -> McpConfig {
    let env_vars: Vec<EnvVar> = config
        .env
        .into_iter()
        .map(|(key, value)| {
            let is_secret = is_likely_secret(&key);
            EnvVar {
                key,
                value,
                is_secret,
            }
        })
        .collect();

    let (docker_image, binary_path, http_url) = match mcp_type {
        McpType::Docker => {
            let image = config.args.first().cloned();
            (image, None, None)
        }
        McpType::Http => (None, None, Some(config.command.clone())),
        McpType::Binary => (None, Some(config.command.clone()), None),
    };

    McpConfig {
        docker_image,
        binary_path,
        http_url,
        command: Some(config.command),
        args: config.args,
        env_vars,
    }
}

fn is_likely_secret(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    key_lower.contains("key")
        || key_lower.contains("secret")
        || key_lower.contains("token")
        || key_lower.contains("password")
        || key_lower.contains("auth")
}
