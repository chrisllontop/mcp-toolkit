use crate::models::*;
use crate::router_mcp::handle_mcp_jsonrpc;
use crate::secrets::SecretManager;
use crate::storage::Storage;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::process::Command;
use uuid::Uuid;

#[derive(Clone)]
pub struct McpRouterState {
    pub storage: Arc<Storage>,
    pub secret_manager: Arc<SecretManager>,
    pub current_project_id: Arc<tokio::sync::RwLock<Option<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolRequest {
    tool: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolResponse {
    result: serde_json::Value,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListToolsResponse {
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolInfo {
    name: String,
    description: String,
    mcp_id: String,
    mcp_name: String,
}

pub async fn start_router(storage: Arc<Storage>, secret_manager: Arc<SecretManager>) -> Result<(), String> {
    let state = McpRouterState {
        storage,
        secret_manager,
        current_project_id: Arc::new(tokio::sync::RwLock::new(None)),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/tools/list", get(list_tools))
        .route("/tools/call", post(call_tool))
        .route("/project/set", post(set_current_project))
        .route("/mcp", post(handle_mcp_jsonrpc))
        // API endpoints for CLI
        .route("/api/projects", get(api_list_projects).post(api_create_project))
        .route("/api/projects/:id", delete(api_delete_project))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9876")
        .await
        .map_err(|e| e.to_string())?;

    println!("MCP Router listening on http://127.0.0.1:9876");

    axum::serve(listener, app)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn list_tools(State(state): State<McpRouterState>) -> impl IntoResponse {
    let project_id = state.current_project_id.read().await;
    if project_id.is_none() {
        return (StatusCode::BAD_REQUEST, Json(ListToolsResponse { tools: vec![] }));
    }

    let project_id = project_id.as_ref().unwrap().clone();

    let bindings = match state.storage.get_bindings_by_project(&project_id) {
        Ok(b) => b,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ListToolsResponse { tools: vec![] })),
    };

    let mcps = match state.storage.get_mcps() {
        Ok(m) => m,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(ListToolsResponse { tools: vec![] })),
    };

    let mut tools = vec![];
    for binding in bindings.iter().filter(|b| b.enabled) {
        if let Some(mcp) = mcps.iter().find(|m| m.id == binding.mcp_id) {
            tools.push(ToolInfo {
                name: format!("{}__execute", mcp.name),
                description: format!("Execute tool from MCP: {}", mcp.name),
                mcp_id: mcp.id.clone(),
                mcp_name: mcp.name.clone(),
            });
        }
    }

    (StatusCode::OK, Json(ListToolsResponse { tools }))
}

async fn call_tool(
    State(state): State<McpRouterState>,
    Json(req): Json<ToolRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let project_id = state.current_project_id.read().await;
    if project_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ToolResponse {
                result: serde_json::json!({}),
                error: Some("No project set".to_string()),
            }),
        );
    }

    let project_id = project_id.as_ref().unwrap().clone();

    let mcp_id = req.tool.split("__").next().unwrap_or("");
    let mcps = match state.storage.get_mcps() {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ToolResponse {
                    result: serde_json::json!({}),
                    error: Some(e.to_string()),
                }),
            )
        }
    };

    let mcp = match mcps.iter().find(|m| m.name == mcp_id || m.id == mcp_id) {
        Some(m) => m,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ToolResponse {
                    result: serde_json::json!({}),
                    error: Some("MCP not found".to_string()),
                }),
            )
        }
    };

    let bindings = match state.storage.get_bindings_by_project(&project_id) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ToolResponse {
                    result: serde_json::json!({}),
                    error: Some(e.to_string()),
                }),
            )
        }
    };

    let binding = match bindings.iter().find(|b| b.mcp_id == mcp.id) {
        Some(b) => b,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ToolResponse {
                    result: serde_json::json!({}),
                    error: Some("Binding not found".to_string()),
                }),
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

    let result = execute_mcp(mcp, &env_vars, &req.args).await;

    let duration = start.elapsed().as_millis() as i64;
    let log = RouterLog {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339(),
        project_id: project_id.clone(),
        mcp_id: mcp.id.clone(),
        tool_name: req.tool.clone(),
        status: if result.is_ok() { "success" } else { "error" }.to_string(),
        duration_ms: duration,
        error: result.as_ref().err().map(|e| e.to_string()),
    };

    let _ = state.storage.insert_log(&log);

    match result {
        Ok(res) => (
            StatusCode::OK,
            Json(ToolResponse {
                result: res,
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ToolResponse {
                result: serde_json::json!({}),
                error: Some(e),
            }),
        ),
    }
}

async fn execute_mcp(mcp: &Mcp, env_vars: &[EnvVar], args: &serde_json::Value) -> Result<serde_json::Value, String> {
    match &mcp.mcp_type {
        McpType::Docker => execute_docker_mcp(mcp, env_vars, args).await,
        McpType::Binary => execute_binary_mcp(mcp, env_vars, args).await,
        McpType::Http => execute_http_mcp(mcp, env_vars, args).await,
    }
}

pub async fn execute_mcp_public(mcp: &Mcp, env_vars: &[EnvVar], args: &serde_json::Value) -> Result<serde_json::Value, String> {
    execute_mcp(mcp, env_vars, args).await
}

async fn execute_docker_mcp(mcp: &Mcp, env_vars: &[EnvVar], args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let image = mcp.config.docker_image.as_ref().ok_or("No docker image specified")?;

    let mut cmd = Command::new("docker");
    cmd.arg("run").arg("--rm");

    for env_var in env_vars {
        cmd.arg("-e").arg(format!("{}={}", env_var.key, env_var.value));
    }

    cmd.arg(image);
    cmd.arg(serde_json::to_string(args).unwrap());

    let output = cmd.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or(serde_json::json!({"output": String::from_utf8_lossy(&output.stdout)}));

    Ok(result)
}

async fn execute_binary_mcp(mcp: &Mcp, env_vars: &[EnvVar], args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let binary_path = mcp.config.binary_path.as_ref().ok_or("No binary path specified")?;

    let mut cmd = Command::new(binary_path);

    for env_var in env_vars {
        cmd.env(&env_var.key, &env_var.value);
    }

    cmd.arg(serde_json::to_string(args).unwrap());

    let output = cmd.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let result: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or(serde_json::json!({"output": String::from_utf8_lossy(&output.stdout)}));

    Ok(result)
}

async fn execute_http_mcp(mcp: &Mcp, env_vars: &[EnvVar], args: &serde_json::Value) -> Result<serde_json::Value, String> {
    let http_url = mcp.config.http_url.as_ref().ok_or("No HTTP URL specified")?;

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

#[derive(Debug, Deserialize)]
struct SetProjectRequest {
    project_id: String,
}

async fn set_current_project(
    State(state): State<McpRouterState>,
    Json(req): Json<SetProjectRequest>,
) -> impl IntoResponse {
    *state.current_project_id.write().await = Some(req.project_id);
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

// API handlers for CLI

#[derive(Deserialize)]
struct CreateProjectRequest {
    name: String,
    path: String,
}

async fn api_list_projects(State(state): State<McpRouterState>) -> impl IntoResponse {
    match state.storage.get_projects() {
        Ok(projects) => (StatusCode::OK, Json(projects)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn api_create_project(
    State(state): State<McpRouterState>,
    Json(req): Json<CreateProjectRequest>,
) -> impl IntoResponse {
    // Validate path exists
    if !std::path::Path::new(&req.path).exists() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Path does not exist"})),
        )
            .into_response();
    }

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name: req.name,
        path: req.path,
        created_at: Utc::now().to_rfc3339(),
    };

    match state.storage.insert_project(&project) {
        Ok(_) => (StatusCode::CREATED, Json(project)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

async fn api_delete_project(
    State(state): State<McpRouterState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.storage.delete_project(&id) {
        Ok(_) => (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
