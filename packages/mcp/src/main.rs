mod executor;
mod mcp_client;
mod mcp_protocol;
mod models;
mod secrets;
mod storage;

use mcp_protocol::*;
use models::*;
use secrets::{get_or_create_key, SecretManager};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write as IoWrite};
use storage::Storage;

fn main() {
    // Initialize secret manager
    let key = match get_or_create_key() {
        Ok(k) => k,
        Err(e) => {
            eprintln!("Failed to initialize encryption key from OS keychain: {}", e);
            eprintln!("Please ensure keychain access is available.");
            std::process::exit(1);
        }
    };
    let secret_manager = SecretManager::new(&key);

    // Initialize storage
    let storage = match Storage::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to initialize storage: {}", e);
            std::process::exit(1);
        }
    };

    // Create Tokio runtime for async operations
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create Tokio runtime: {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("MCP Toolkit server starting...");

    // Process stdin/stdout
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                eprintln!("Received input: {}", input);
                let response = runtime.block_on(async {
                    handle_request(&input, &storage, &secret_manager).await
                });

                match response {
                    Ok(resp) => {
                        if !resp.is_empty() {
                            eprintln!("Sending response: {}", resp);
                            if let Err(e) = writeln!(stdout, "{}", resp) {
                                let _ = writeln!(stderr, "Error writing response: {}", e);
                            }
                            let _ = stdout.flush();
                        } else {
                            eprintln!("Empty response (notification acknowledged)");
                        }
                    }
                    Err(e) => {
                        eprintln!("Error handling request: {}", e);
                        let error_response = json!({
                            "jsonrpc": "2.0",
                            "id": null,
                            "error": {
                                "code": -32603,
                                "message": format!("Internal error: {}", e)
                            }
                        });
                        let _ = writeln!(stdout, "{}", error_response);
                        let _ = stdout.flush();
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    eprintln!("Server exiting...");
}

async fn handle_request(
    input: &str,
    storage: &Storage,
    secret_manager: &SecretManager,
) -> Result<String, String> {
    let request: JsonRpcRequest = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse JSON-RPC request: {}", e))?;

    let id = request.id.clone();

    // Handle notifications (no response needed)
    if request.method == "notifications/initialized" {
        return Ok("".to_string());
    }

    let response = match request.method.as_str() {
        "initialize" => handle_initialize(id, &request),
        "tools/list" => handle_tools_list(id, storage),
        "tools/call" => {
            handle_tools_call(id, &request, storage, secret_manager).await
        }
        _ => JsonRpcResponse::error(
            id,
            -32601,
            format!("Method not found: {}", request.method),
        ),
    };

    serde_json::to_string(&response).map_err(|e| format!("Failed to serialize response: {}", e))
}

fn handle_initialize(id: Option<Value>, request: &JsonRpcRequest) -> JsonRpcResponse {
    // Extract protocol version from client's request
    let client_protocol_version = request
        .params
        .as_ref()
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");

    eprintln!("Client requested protocol version: {}", client_protocol_version);

    // Use the client's protocol version in response
    let result = InitializeResult {
        protocol_version: client_protocol_version.to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: None, // Empty tools capability
            }),
            experimental: None,
        },
        server_info: ServerInfo {
            name: "mcp-toolkit".to_string(),
            version: "0.1.0".to_string(),
        },
    };

    JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
}

fn handle_tools_list(id: Option<Value>, storage: &Storage) -> JsonRpcResponse {
    match storage.get_enabled_mcps_with_bindings() {
        Ok(mcp_bindings) => {
            let mut all_tools: Vec<McpTool> = Vec::new();

            for (mcp, binding) in &mcp_bindings {
                eprintln!("[handle_tools_list] Listing tools for MCP: {}", mcp.name);

                // Merge env vars with overrides
                let mut env_vars = mcp.config.env_vars.clone();
                for override_var in &binding.overrides {
                    if let Some(existing) = env_vars.iter_mut().find(|v| v.key == override_var.key) {
                        existing.value = override_var.value.clone();
                    } else {
                        env_vars.push(override_var.clone());
                    }
                }

                // Create MCP client to list actual tools
                match mcp_client::McpClient::new(mcp, &env_vars) {
                    Ok(client) => {
                        // Initialize connection
                        if let Err(e) = client.initialize() {
                            eprintln!("[handle_tools_list] Failed to initialize MCP {}: {}", mcp.name, e);
                            continue;
                        }

                        // List tools from this MCP
                        match client.list_tools() {
                            Ok(mcp_tools) => {
                                eprintln!("[handle_tools_list] Found {} tools for MCP: {}", mcp_tools.len(), mcp.name);

                                // Add each tool with server prefix
                                for tool in mcp_tools {
                                    let tool_name = match tool.get("name").and_then(|n| n.as_str()) {
                                        Some(name) => name,
                                        None => {
                                            eprintln!("[handle_tools_list] Tool missing 'name' field, skipping");
                                            continue;
                                        }
                                    };

                                    // Create prefixed tool name: mcp_name__tool_name
                                    // Replace spaces and special chars to match pattern ^[a-zA-Z0-9_-]{1,64}$
                                    let mcp_prefix = mcp.name
                                        .replace(" ", "_")
                                        .replace("-", "_");
                                    let prefixed_name = format!("{}__{}", mcp_prefix, tool_name);

                                    // Extract description and schema
                                    let description = tool
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .unwrap_or("")
                                        .to_string();

                                    let input_schema = tool
                                        .get("inputSchema")
                                        .cloned()
                                        .unwrap_or(json!({}));

                                    all_tools.push(McpTool {
                                        name: prefixed_name,
                                        description,
                                        input_schema,
                                    });
                                }
                            }
                            Err(e) => {
                                eprintln!("[handle_tools_list] Failed to list tools for MCP {}: {}", mcp.name, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[handle_tools_list] Failed to create client for MCP {}: {}", mcp.name, e);
                    }
                }
            }

            eprintln!("[handle_tools_list] Total tools listed: {}", all_tools.len());
            let result = ListToolsResult { tools: all_tools };
            JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(e) => JsonRpcResponse::error(id, -32000, format!("Failed to get MCPs: {}", e)),
    }
}

async fn handle_tools_call(
    id: Option<Value>,
    request: &JsonRpcRequest,
    storage: &Storage,
    secret_manager: &SecretManager,
) -> JsonRpcResponse {
    // Parse the call tool request
    let call_request: CallToolRequest = match request.params.as_ref() {
        Some(params) => match serde_json::from_value(params.clone()) {
            Ok(req) => req,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    -32602,
                    format!("Invalid params: {}", e),
                )
            }
        },
        None => {
            return JsonRpcResponse::error(
                id,
                -32602,
                "Missing params".to_string(),
            )
        }
    };

    // Get all enabled MCPs with bindings
    let mcp_bindings = match storage.get_enabled_mcps_with_bindings() {
        Ok(mcps) => mcps,
        Err(e) => {
            return JsonRpcResponse::error(
                id,
                -32000,
                format!("Failed to get MCPs: {}", e),
            )
        }
    };

    // Parse the prefixed tool name: "mcp_prefix__tool_name"
    let tool_name = &call_request.name;
    let (mcp_prefix, actual_tool_name) = match tool_name.split_once("__") {
        Some((prefix, name)) => (prefix, name),
        None => {
            return JsonRpcResponse::error(
                id,
                -32602,
                format!("Invalid tool name format. Expected 'mcp_prefix__tool_name', got: {}", tool_name),
            )
        }
    };

    // Find the MCP by matching the normalized prefix
    let mut target_mcp: Option<(&Mcp, &ProjectMcpBinding)> = None;
    for (mcp, binding) in &mcp_bindings {
        let normalized_name = mcp.name
            .replace(" ", "_")
            .replace("-", "_");
        if normalized_name == mcp_prefix {
            target_mcp = Some((mcp, binding));
            break;
        }
    }

    let (mcp, binding) = match target_mcp {
        Some(t) => t,
        None => {
            return JsonRpcResponse::error(
                id,
                -32602,
                format!("MCP not found for prefix: {}", mcp_prefix),
            )
        }
    };

    // Merge env vars with overrides
    let mut env_vars = mcp.config.env_vars.clone();
    for override_var in &binding.overrides {
        if let Some(existing) = env_vars.iter_mut().find(|v| v.key == override_var.key) {
            existing.value = override_var.value.clone();
        } else {
            env_vars.push(override_var.clone());
        }
    }

    // Decrypt secrets
    for env_var in env_vars.iter_mut() {
        if env_var.is_secret {
            if let Ok(Some(encrypted)) = storage.get_encrypted_secret(&env_var.value) {
                if let Ok(decrypted) = secret_manager.decrypt(&encrypted) {
                    env_var.value = decrypted;
                }
            }
        }
    }

    // Execute the MCP with the actual tool name (without prefix)
    let args = call_request.arguments.unwrap_or(json!({}));
    let result = executor::execute_mcp(mcp, &env_vars, actual_tool_name, &args).await;

    match result {
        Ok(output) => {
            let call_result = CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string_pretty(&output).unwrap_or_else(|_| output.to_string()),
                }],
                is_error: None,
            };
            JsonRpcResponse::success(id, serde_json::to_value(call_result).unwrap())
        }
        Err(e) => {
            let call_result = CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            };
            JsonRpcResponse::success(id, serde_json::to_value(call_result).unwrap())
        }
    }
}
