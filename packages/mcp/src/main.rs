use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct Mcp {
    id: String,
    name: String,
    mcp_type: String,
    config: String,
}

fn get_db_path() -> PathBuf {
    // Match Tauri's app_data_dir path
    let mut path = dirs::data_local_dir().expect("Could not find data directory");
    path.push("com.mcp.manager");
    path.push("mcp_manager.db");
    path
}

fn get_enabled_mcps() -> Result<Vec<Mcp>, String> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    let mut stmt = conn
        .prepare("SELECT m.id, m.name, m.mcp_type, m.config FROM mcps m INNER JOIN project_mcp_bindings b ON m.id = b.mcp_id WHERE b.enabled = 1")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let mcps = stmt
        .query_map([], |row| {
            Ok(Mcp {
                id: row.get(0)?,
                name: row.get(1)?,
                mcp_type: row.get(2)?,
                config: row.get(3)?,
            })
        })
        .map_err(|e| format!("Failed to query mcps: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Failed to collect mcps: {}", e))?;

    Ok(mcps)
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    for line in stdin.lock().lines() {
        match line {
            Ok(input) => {
                match handle_request(&input) {
                    Ok(response) => {
                        // Only write response if not empty (notifications return empty string)
                        if !response.is_empty() {
                            if let Err(e) = writeln!(stdout, "{}", response) {
                                let _ = writeln!(stderr, "Error writing response: {}", e);
                            }
                            let _ = stdout.flush();
                        }
                    }
                    Err(e) => {
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
                let _ = writeln!(stderr, "Error reading input: {}", e);
                break;
            }
        }
    }
}

fn handle_request(input: &str) -> Result<String, String> {
    let request: Value = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let method = request["method"]
        .as_str()
        .ok_or("Missing method field")?;

    let id = request.get("id").cloned();

    // Handle notifications (no response needed)
    if method == "notifications/initialized" {
        // Notifications don't get a response
        return Ok("".to_string());
    }

    let response = match method {
        "initialize" => handle_initialize(id),
        "tools/list" => handle_tools_list(id),
        "tools/call" => handle_tools_call(id, &request),
        _ => error_response(
            id,
            -32601,
            format!("Method not found: {}", method),
        ),
    };

    serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))
}

fn handle_initialize(id: Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "mcp-manager",
                "version": "0.1.0"
            }
        }
    })
}

fn handle_tools_list(id: Option<Value>) -> Value {
    match get_enabled_mcps() {
        Ok(mcps) => {
            let tools: Vec<Value> = mcps
                .iter()
                .map(|mcp| {
                    json!({
                        "name": format!("{}_{}", mcp.name.replace(" ", "_").to_lowercase(), mcp.id.chars().take(6).collect::<String>()),
                        "description": format!("Execute MCP: {}", mcp.name),
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "params": {
                                    "type": "object",
                                    "description": "Parameters to pass to the MCP"
                                }
                            }
                        }
                    })
                })
                .collect();

            json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": tools
                }
            })
        }
        Err(e) => error_response(id, -32000, format!("Failed to get MCPs: {}", e)),
    }
}

fn handle_tools_call(id: Option<Value>, _request: &Value) -> Value {
    // TODO: Implement tool execution
    error_response(
        id,
        -32000,
        "Tool execution not implemented yet".to_string(),
    )
}

fn error_response(id: Option<Value>, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}
