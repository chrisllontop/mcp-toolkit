use crate::mcp_protocol::*;
use crate::router::McpRouterState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

pub async fn handle_mcp_jsonrpc(
    State(state): State<McpRouterState>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if req.jsonrpc != "2.0" {
        let response = JsonRpcResponse::error(
            req.id,
            -32600,
            "Invalid Request: jsonrpc must be '2.0'".to_string(),
        );
        return (StatusCode::OK, Json(response));
    }

    let result = match req.method.as_str() {
        "initialize" => handle_initialize(&req).await,
        "tools/list" => handle_tools_list(&state).await,
        "tools/call" => handle_tools_call(&state, &req).await,
        _ => JsonRpcResponse::error(
            req.id.clone(),
            -32601,
            format!("Method not found: {}", req.method),
        ),
    };

    (StatusCode::OK, Json(result))
}

async fn handle_initialize(req: &JsonRpcRequest) -> JsonRpcResponse {
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
            experimental: None,
        },
        server_info: ServerInfo {
            name: "mcp-manager".to_string(),
            version: "0.1.0".to_string(),
        },
    };

    JsonRpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap())
}

async fn handle_tools_list(state: &McpRouterState) -> JsonRpcResponse {
    let project_id = state.current_project_id.read().await;
    if project_id.is_none() {
        return JsonRpcResponse::error(
            None,
            -32000,
            "No project set. Please set active project first.".to_string(),
        );
    }

    let project_id = project_id.as_ref().unwrap().clone();

    let bindings = match state.storage.get_bindings_by_project(&project_id) {
        Ok(b) => b,
        Err(e) => {
            return JsonRpcResponse::error(None, -32000, format!("Failed to get bindings: {}", e))
        }
    };

    let mcps = match state.storage.get_mcps() {
        Ok(m) => m,
        Err(e) => {
            return JsonRpcResponse::error(None, -32000, format!("Failed to get MCPs: {}", e))
        }
    };

    let mut tools = vec![];
    for binding in bindings.iter().filter(|b| b.enabled) {
        if let Some(mcp) = mcps.iter().find(|m| m.id == binding.mcp_id) {
            tools.push(McpTool {
                name: format!("{}__execute", mcp.name),
                description: format!("Execute tool from MCP: {}", mcp.name),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "args": {
                            "type": "object",
                            "description": "Arguments to pass to the MCP tool"
                        }
                    }
                }),
            });
        }
    }

    let result = ListToolsResult { tools };
    JsonRpcResponse::success(None, serde_json::to_value(result).unwrap())
}

async fn handle_tools_call(state: &McpRouterState, req: &JsonRpcRequest) -> JsonRpcResponse {
    let params: CallToolRequest = match req.params.as_ref() {
        Some(p) => match serde_json::from_value(p.clone()) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id.clone(),
                    -32602,
                    format!("Invalid params: {}", e),
                )
            }
        },
        None => {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32602,
                "Missing params for tools/call".to_string(),
            )
        }
    };

    let project_id = state.current_project_id.read().await;
    if project_id.is_none() {
        return JsonRpcResponse::error(
            req.id.clone(),
            -32000,
            "No project set".to_string(),
        );
    }

    let project_id = project_id.as_ref().unwrap().clone();

    let mcp_name = params.name.split("__").next().unwrap_or("");
    let mcps = match state.storage.get_mcps() {
        Ok(m) => m,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32000,
                format!("Failed to get MCPs: {}", e),
            )
        }
    };

    let mcp = match mcps.iter().find(|m| m.name == mcp_name || m.id == mcp_name) {
        Some(m) => m,
        None => {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32000,
                format!("MCP not found: {}", mcp_name),
            )
        }
    };

    let bindings = match state.storage.get_bindings_by_project(&project_id) {
        Ok(b) => b,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32000,
                format!("Failed to get bindings: {}", e),
            )
        }
    };

    let binding = match bindings.iter().find(|b| b.mcp_id == mcp.id) {
        Some(b) => b,
        None => {
            return JsonRpcResponse::error(
                req.id.clone(),
                -32000,
                "MCP not activated for this project".to_string(),
            )
        }
    };

    let mut env_vars = mcp.config.env_vars.clone();
    for override_var in &binding.overrides {
        if let Some(existing) = env_vars.iter_mut().find(|v| v.key == override_var.key) {
            existing.value = override_var.value.clone();
        } else {
            env_vars.push(override_var.clone());
        }
    }

    for env_var in env_vars.iter_mut() {
        if env_var.is_secret {
            if let Ok(Some(encrypted)) = state.storage.get_encrypted_secret(&env_var.value) {
                if let Ok(decrypted) = state.secret_manager.decrypt(&encrypted) {
                    env_var.value = decrypted;
                }
            }
        }
    }

    let args = params.arguments.unwrap_or(json!({}));
    let execution_result = crate::router::execute_mcp_public(mcp, &env_vars, &args).await;

    match execution_result {
        Ok(res) => {
            let result = CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: serde_json::to_string_pretty(&res).unwrap_or_else(|_| res.to_string()),
                }],
                is_error: Some(false),
            };
            JsonRpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap())
        }
        Err(e) => {
            let result = CallToolResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: format!("Error executing MCP: {}", e),
                }],
                is_error: Some(true),
            };
            JsonRpcResponse::success(req.id.clone(), serde_json::to_value(result).unwrap())
        }
    }
}
