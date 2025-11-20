use std::path::PathBuf;

/// Detecta el sistema operativo actual y retorna la ruta del binario mcp-toolkit
pub fn get_mcp_stdio_path() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let path = "/Applications/MCP Toolkit.app/Contents/MacOS/mcp-toolkit";
        Ok(path.to_string())
    }

    #[cfg(target_os = "windows")]
    {
        // En Windows, necesitamos expandir %LOCALAPPDATA% o usar una ruta por defecto
        if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
            let mut path = PathBuf::from(local_appdata);
            path.push("MCP Toolkit");
            path.push("mcp-toolkit.exe");
            Ok(path.to_string_lossy().to_string())
        } else {
            // Fallback a ruta por defecto
            Ok("C:\\Program Files\\MCP Toolkit\\mcp-toolkit.exe".to_string())
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Intentar detectar la instalación en ubicaciones comunes
        let possible_paths = vec![
            "/usr/local/bin/mcp-toolkit",
            "~/.local/share/MCP Toolkit/mcp-toolkit",
            "/opt/MCP Toolkit/mcp-toolkit",
        ];

        // Si tenemos HOME, expandir la ruta con ~
        if let Some(home) = std::env::var_os("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local/share/MCP Toolkit/mcp-toolkit");
            Ok(path.to_string_lossy().to_string())
        } else {
            Ok(possible_paths[0].to_string())
        }
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Unsupported operating system".to_string())
    }
}

/// Genera la configuración JSON para MCP Toolkit
pub fn generate_mcp_config() -> Result<String, String> {
    let command_path = get_mcp_stdio_path()?;

    let config = serde_json::json!({
        "mcpServers": {
            "mcp-toolkit": {
                "command": command_path,
                "args": []
            }
        }
    });

    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mcp_stdio_path() {
        let path = get_mcp_stdio_path();
        assert!(path.is_ok());
        assert!(!path.unwrap().is_empty());
    }

    #[test]
    fn test_generate_mcp_config() {
        let config = generate_mcp_config();
        assert!(config.is_ok());
        let config_str = config.unwrap();
        assert!(config_str.contains("mcpServers"));
        assert!(config_str.contains("mcp-toolkit"));
        assert!(config_str.contains("command"));
    }
}
