use crate::mcp_client::McpClient;
use crate::models::*;
use serde_json::Value;

pub async fn execute_mcp(
    mcp: &Mcp,
    env_vars: &[EnvVar],
    tool_name: &str,
    args: &Value,
) -> Result<Value, String> {
    match &mcp.mcp_type {
        McpType::Docker | McpType::Binary => execute_stdio_mcp(mcp, env_vars, tool_name, args).await,
        McpType::Http => execute_http_mcp(mcp, env_vars, args).await,
    }
}

/// Execute MCP via stdio (Docker or Binary)
async fn execute_stdio_mcp(
    mcp: &Mcp,
    env_vars: &[EnvVar],
    tool_name: &str,
    args: &Value,
) -> Result<Value, String> {
    eprintln!("[Executor] Creating MCP client for: {}", mcp.name);

    // Create MCP client
    let client = McpClient::new(mcp, env_vars)?;

    // Initialize the connection
    eprintln!("[Executor] Initializing MCP: {}", mcp.name);
    client.initialize()?;

    eprintln!(
        "[Executor] Calling tool '{}' on: {}",
        tool_name, mcp.name
    );

    // Call the tool
    let result = client.call_tool(tool_name, args)?;

    eprintln!("[Executor] Tool call successful for: {}", mcp.name);
    Ok(result)
}

/// Execute HTTP MCP (unchanged from original)
async fn execute_http_mcp(
    mcp: &Mcp,
    env_vars: &[EnvVar],
    args: &Value,
) -> Result<Value, String> {
    let http_url = mcp
        .config
        .http_url
        .as_ref()
        .ok_or("No HTTP URL specified")?;

    let client = reqwest::Client::new();
    let mut req = client.post(http_url).json(args);

    for env_var in env_vars {
        if env_var.key.to_lowercase().starts_with("header_") {
            let header_name = env_var.key[7..].to_string();
            req = req.header(header_name, &env_var.value);
        }
    }

    let response = req.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    response.json().await.map_err(|e| e.to_string())
}
