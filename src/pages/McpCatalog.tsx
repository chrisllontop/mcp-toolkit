import { useEffect, useState } from "react";
import { api, Mcp, ImportPreview } from "../api";

function McpCatalog() {
  const [mcps, setMcps] = useState<Mcp[]>([]);
  const [showAddModal, setShowAddModal] = useState(false);
  const [jsonInput, setJsonInput] = useState("");
  const [importPreviews, setImportPreviews] = useState<ImportPreview[]>([]);

  useEffect(() => {
    loadMcps();
  }, []);

  const loadMcps = async () => {
    const data = await api.listMcps();
    setMcps(data);
  };

  const handleDelete = async (e: React.MouseEvent, id: string) => {
    e.preventDefault();
    e.stopPropagation();

    if (!window.confirm("Delete this MCP?")) return;
    try {
      await api.deleteMcp(id);
      loadMcps();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const handleParseJson = async () => {
    try {
      const previews = await api.parseMcpJson(jsonInput);
      setImportPreviews(previews);
    } catch (error) {
      alert(`Error parsing JSON: ${error}`);
    }
  };

  const handleImportJson = async () => {
    try {
      await api.importMcpsFromJson(jsonInput);
      setShowAddModal(false);
      setJsonInput("");
      setImportPreviews([]);
      loadMcps();
    } catch (error) {
      alert(`Error importing MCPs: ${error}`);
    }
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <h1>MCP Catalog</h1>
        <button onClick={() => setShowAddModal(true)}>Add MCP</button>
      </div>

      <div>
        {mcps.map((mcp) => (
          <div key={mcp.id} className="card">
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div>
                <h3>{mcp.name}</h3>
                <span className="badge success">{mcp.mcp_type}</span>
              </div>
              <button type="button" className="danger" onClick={(e) => handleDelete(e, mcp.id)}>Delete</button>
            </div>
          </div>
        ))}
      </div>

      {showAddModal && (
        <div className="modal">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Add MCP</h2>
              <button className="close-btn" onClick={() => setShowAddModal(false)}>×</button>
            </div>
            <div className="form-group">
              <label>Paste MCP JSON Configuration</label>
              <textarea
                value={jsonInput}
                onChange={(e) => setJsonInput(e.target.value)}
                placeholder={`{\n  "mcpServers": {\n    "server-name": {\n      "command": "executable",\n      "args": ["arg1"],\n      "env": { "KEY": "value" }\n    }\n  }\n}`}
                rows={12}
                style={{ fontFamily: "monospace", fontSize: 12 }}
              />
            </div>
            {importPreviews.length > 0 && (
              <div style={{ marginTop: 15, padding: 15, backgroundColor: "#1a1a1a", borderRadius: 5 }}>
                <h3 style={{ fontSize: 14, marginBottom: 10 }}>Preview ({importPreviews.length} MCPs)</h3>
                {importPreviews.map((preview, idx) => (
                  <div key={idx} style={{ marginBottom: 8, fontSize: 13 }}>
                    <strong>{preview.name}</strong>
                    <span className="badge success" style={{ marginLeft: 8 }}>{preview.mcp_type}</span>
                    <div style={{ color: "#888", fontSize: 11, marginTop: 3 }}>
                      {preview.command} {preview.args.join(" ")}
                    </div>
                  </div>
                ))}
              </div>
            )}
            <div className="button-group">
              {importPreviews.length === 0 ? (
                <button onClick={handleParseJson}>Parse JSON</button>
              ) : (
                <button onClick={handleImportJson}>Add {importPreviews.length} MCP(s)</button>
              )}
              <button className="secondary" onClick={() => setShowAddModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default McpCatalog;
