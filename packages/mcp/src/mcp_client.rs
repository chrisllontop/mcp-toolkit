use crate::models::*;
use crate::mcp_protocol::*;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write as IoWrite};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct McpClient {
    process: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    next_id: Arc<AtomicU64>,
    pub mcp_name: String,
}

impl McpClient {
    /// Creates a new MCP client and starts the process
    pub fn new(mcp: &Mcp, env_vars: &[EnvVar]) -> Result<Self, String> {
        eprintln!("[McpClient] Starting MCP: {}", mcp.name);

        let mut cmd = match &mcp.mcp_type {
            McpType::Docker => {
                let image = mcp
                    .config
                    .docker_image
                    .as_ref()
                    .ok_or("No docker image specified")?;

                let mut cmd = Command::new("docker");
                cmd.arg("run")
                    .arg("--rm")
                    .arg("-i") // Interactive mode for stdin
                    .arg("--init"); // Use init process

                // Add environment variables
                for env_var in env_vars {
                    cmd.arg("-e").arg(format!("{}={}", env_var.key, env_var.value));
                }

                cmd.arg(image);
                cmd
            }
            McpType::Binary => {
                let binary_path = mcp
                    .config
                    .binary_path
                    .as_ref()
                    .ok_or("No binary path specified")?;

                let mut cmd = Command::new(binary_path);

                // Add environment variables
                for env_var in env_vars {
                    cmd.env(&env_var.key, &env_var.value);
                }

                // Add arguments if specified
                if !mcp.config.args.is_empty() {
                    cmd.args(&mcp.config.args);
                }

                cmd
            }
            McpType::Http => {
                return Err("HTTP MCPs not supported via stdio client".to_string());
            }
        };

        // Configure stdio
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Start the process
        let mut process = cmd.spawn().map_err(|e| {
            format!("Failed to start MCP process '{}': {}", mcp.name, e)
        })?;

        // Take ownership of stdin/stdout/stderr
        let stdin = process
            .stdin
            .take()
            .ok_or("Failed to open stdin for MCP process")?;
        let stdout = process
            .stdout
            .take()
            .ok_or("Failed to open stdout for MCP process")?;
        let stderr = process
            .stderr
            .take()
            .ok_or("Failed to open stderr for MCP process")?;

        // Spawn thread to read stderr
        let mcp_name_clone = mcp.name.clone();
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[McpClient stderr:{}] {}", mcp_name_clone, line);
                }
            }
        });

        let client = McpClient {
            process: Arc::new(Mutex::new(process)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            next_id: Arc::new(AtomicU64::new(1)),
            mcp_name: mcp.name.clone(),
        };

        eprintln!("[McpClient] Process started for: {}", mcp.name);
        Ok(client)
    }

    /// Initialize the MCP connection
    pub fn initialize(&self) -> Result<Value, String> {
        eprintln!("[McpClient] Initializing: {}", self.mcp_name);

        let init_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(self.next_id.fetch_add(1, Ordering::SeqCst))),
            method: "initialize".to_string(),
            params: Some(json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {
                    "name": "mcp-toolkit",
                    "version": "0.1.0"
                }
            })),
        };

        let response = self.send_request(&init_request)?;
        eprintln!("[McpClient] Initialize response: {:?}", response);

        // Send initialized notification
        let init_notification = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "notifications/initialized".to_string(),
            params: None,
        };

        self.send_notification(&init_notification)?;
        eprintln!("[McpClient] Sent initialized notification for: {}", self.mcp_name);

        Ok(response)
    }

    /// List available tools from the MCP server
    pub fn list_tools(&self) -> Result<Vec<Value>, String> {
        eprintln!("[McpClient] Listing tools for: {}", self.mcp_name);

        let list_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(self.next_id.fetch_add(1, Ordering::SeqCst))),
            method: "tools/list".to_string(),
            params: Some(json!({})),
        };

        let response = self.send_request(&list_request)?;

        // Extract tools array from response
        let tools = response
            .get("tools")
            .and_then(|t| t.as_array())
            .ok_or("Invalid tools/list response: missing 'tools' array")?
            .clone();

        eprintln!("[McpClient] Found {} tools for: {}", tools.len(), self.mcp_name);
        Ok(tools)
    }

    /// Call a tool on the MCP server
    pub fn call_tool(&self, tool_name: &str, arguments: &Value) -> Result<Value, String> {
        eprintln!(
            "[McpClient] Calling tool '{}' on: {}",
            tool_name, self.mcp_name
        );

        let call_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(self.next_id.fetch_add(1, Ordering::SeqCst))),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": tool_name,
                "arguments": arguments
            })),
        };

        let response = self.send_request(&call_request)?;
        eprintln!("[McpClient] Tool call response: {:?}", response);

        Ok(response)
    }

    /// Send a JSON-RPC request and wait for response
    fn send_request(&self, request: &JsonRpcRequest) -> Result<Value, String> {
        // Serialize request
        let request_str =
            serde_json::to_string(request).map_err(|e| format!("Failed to serialize request: {}", e))?;

        eprintln!("[McpClient] >>> {}", request_str);

        // Send to stdin
        {
            let mut stdin = self.stdin.lock().unwrap();
            writeln!(stdin, "{}", request_str).map_err(|e| format!("Failed to write to stdin: {}", e))?;
            stdin.flush().map_err(|e| format!("Failed to flush stdin: {}", e))?;
        }

        // Read response from stdout - keep reading until we get valid JSON
        let response_str = {
            let mut stdout = self.stdout.lock().unwrap();
            let mut attempts = 0;
            const MAX_ATTEMPTS: i32 = 10;

            loop {
                let mut line = String::new();
                let bytes_read = stdout
                    .read_line(&mut line)
                    .map_err(|e| format!("Failed to read from stdout: {}", e))?;

                if bytes_read == 0 {
                    return Err("EOF: Process closed stdout".to_string());
                }

                let trimmed = line.trim();

                // Skip empty lines or lines that don't look like JSON
                if trimmed.is_empty() || !trimmed.starts_with('{') {
                    eprintln!("[McpClient] Skipping non-JSON line: {}", trimmed);
                    attempts += 1;
                    if attempts >= MAX_ATTEMPTS {
                        return Err("Too many non-JSON lines, giving up".to_string());
                    }
                    continue;
                }

                break line;
            }
        };

        eprintln!("[McpClient] <<< {}", response_str.trim());

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        // Check for error
        if let Some(error) = response.error {
            return Err(format!(
                "MCP error {}: {}",
                error.code, error.message
            ));
        }

        // Return result
        response
            .result
            .ok_or_else(|| "Response missing result field".to_string())
    }

    /// Send a notification (no response expected)
    fn send_notification(&self, request: &JsonRpcRequest) -> Result<(), String> {
        let request_str =
            serde_json::to_string(request).map_err(|e| format!("Failed to serialize notification: {}", e))?;

        eprintln!("[McpClient] >>> (notification) {}", request_str);

        let mut stdin = self.stdin.lock().unwrap();
        writeln!(stdin, "{}", request_str).map_err(|e| format!("Failed to write notification: {}", e))?;
        stdin.flush().map_err(|e| format!("Failed to flush stdin: {}", e))?;

        Ok(())
    }

    /// Check if the process is still running
    pub fn is_alive(&self) -> bool {
        let mut process = self.process.lock().unwrap();
        match process.try_wait() {
            Ok(Some(_)) => false, // Process has exited
            Ok(None) => true,     // Process is still running
            Err(_) => false,      // Error checking status
        }
    }

    /// Shutdown the MCP client
    pub fn shutdown(&self) -> Result<(), String> {
        eprintln!("[McpClient] Shutting down: {}", self.mcp_name);

        let mut process = self.process.lock().unwrap();
        process
            .kill()
            .map_err(|e| format!("Failed to kill process: {}", e))?;
        process
            .wait()
            .map_err(|e| format!("Failed to wait for process: {}", e))?;

        eprintln!("[McpClient] Shutdown complete: {}", self.mcp_name);
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        eprintln!("[McpClient] Dropping client for: {}", self.mcp_name);
        let _ = self.shutdown();
    }
}
