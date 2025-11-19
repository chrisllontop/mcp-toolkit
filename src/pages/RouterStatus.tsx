import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { api, RouterLog } from "../api";

function RouterStatus() {
  const { projectId } = useParams<{ projectId?: string }>();
  const [logs, setLogs] = useState<RouterLog[]>([]);
  const [routerStatus, setRouterStatus] = useState<"ok" | "error">("ok");
  const [mcpConfig, setMcpConfig] = useState<string>("");
  const [showConfig, setShowConfig] = useState(false);

  useEffect(() => {
    loadLogs();
    checkRouter();
    const interval = setInterval(() => {
      loadLogs();
      checkRouter();
    }, 5000);
    return () => clearInterval(interval);
  }, []);

  const loadLogs = async () => {
    try {
      const data = await api.getRecentLogs(50);
      setLogs(data);
    } catch (error) {
      console.error(error);
    }
  };

  const checkRouter = async () => {
    try {
      const response = await fetch("http://127.0.0.1:9876/health");
      if (response.ok) {
        setRouterStatus("ok");
      } else {
        setRouterStatus("error");
      }
    } catch (error) {
      setRouterStatus("error");
    }
  };

  const handleGenerateConfig = async () => {
    if (!projectId) {
      alert("No project selected. Please select a project first.");
      return;
    }
    try {
      const config = await api.generateMcpConfig(projectId);
      setMcpConfig(config);
      setShowConfig(true);
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const handleCopyConfig = () => {
    navigator.clipboard.writeText(mcpConfig);
    alert("Configuration copied to clipboard!");
  };

  return (
    <div>
      <h1>Router Status</h1>

      <div className="card" style={{ marginBottom: 20 }}>
        <h3>MCP Router</h3>
        <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 10 }}>
          <span>Status:</span>
          <span className={`badge ${routerStatus === "ok" ? "success" : "error"}`}>
            {routerStatus === "ok" ? "Running" : "Error"}
          </span>
        </div>
        <p style={{ color: "#888", fontSize: 14, marginBottom: 15 }}>
          MCP Endpoint: http://127.0.0.1:9876/mcp
        </p>
        <button onClick={handleGenerateConfig}>Generate Claude Code Config</button>
      </div>

      {showConfig && (
        <div className="card" style={{ marginBottom: 20 }}>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 10 }}>
            <h3>Configuration for Claude Code</h3>
            <button className="secondary" onClick={handleCopyConfig}>Copy to Clipboard</button>
          </div>
          <p style={{ color: "#888", fontSize: 13, marginBottom: 10 }}>
            Create <code>.mcp.json</code> in your project root with this content:
          </p>
          <pre style={{
            backgroundColor: "#1a1a1a",
            padding: 15,
            borderRadius: 5,
            overflow: "auto",
            fontSize: 12,
            fontFamily: "monospace"
          }}>
            {mcpConfig}
          </pre>
          <p style={{ color: "#888", fontSize: 13, marginTop: 10 }}>
            Or use CLI: <code style={{ backgroundColor: "#1a1a1a", padding: "2px 6px", borderRadius: 3 }}>
              claude mcp add --transport http mcp-manager http://127.0.0.1:9876/mcp --scope project
            </code>
          </p>
        </div>
      )}

      <h2>Recent Logs</h2>
      <table>
        <thead>
          <tr>
            <th>Time</th>
            <th>Tool</th>
            <th>Status</th>
            <th>Duration (ms)</th>
            <th>Error</th>
          </tr>
        </thead>
        <tbody>
          {logs.map((log) => (
            <tr key={log.id}>
              <td>{new Date(log.timestamp).toLocaleTimeString()}</td>
              <td>{log.tool_name}</td>
              <td>
                <span className={`badge ${log.status === "success" ? "success" : "error"}`}>
                  {log.status}
                </span>
              </td>
              <td>{log.duration_ms}</td>
              <td style={{ color: log.error ? "#e63946" : "#888" }}>{log.error || "-"}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export default RouterStatus;
