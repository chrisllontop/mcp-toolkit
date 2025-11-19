import { useEffect, useState } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { api, ProjectMcpBinding, Mcp, EnvVar } from "../api";

function ProjectDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [bindings, setBindings] = useState<ProjectMcpBinding[]>([]);
  const [allMcps, setAllMcps] = useState<Mcp[]>([]);
  const [showAddModal, setShowAddModal] = useState(false);
  const [selectedMcpId, setSelectedMcpId] = useState("");
  const [editingBinding, setEditingBinding] = useState<ProjectMcpBinding | null>(null);
  const [overrides, setOverrides] = useState<EnvVar[]>([]);

  useEffect(() => {
    if (id) {
      loadBindings();
      loadMcps();
    }
  }, [id]);

  const loadBindings = async () => {
    if (!id) return;
    const data = await api.listBindings(id);
    setBindings(data);
  };

  const loadMcps = async () => {
    const data = await api.listMcps();
    setAllMcps(data);
  };

  const handleActivate = async () => {
    if (!id || !selectedMcpId) return;
    try {
      await api.activateMcp(id, selectedMcpId, overrides);
      setShowAddModal(false);
      setSelectedMcpId("");
      setOverrides([]);
      loadBindings();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const handleToggle = async (binding: ProjectMcpBinding) => {
    try {
      await api.updateBinding({ ...binding, enabled: !binding.enabled });
      loadBindings();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const handleEditOverrides = (binding: ProjectMcpBinding) => {
    setEditingBinding(binding);
    setOverrides([...binding.overrides]);
  };

  const handleSaveOverrides = async () => {
    if (!editingBinding) return;
    try {
      await api.updateBinding({ ...editingBinding, overrides });
      setEditingBinding(null);
      setOverrides([]);
      loadBindings();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const addOverride = () => {
    setOverrides([...overrides, { key: "", value: "", is_secret: false }]);
  };

  const updateOverride = (index: number, field: keyof EnvVar, value: string | boolean) => {
    const updated = [...overrides];
    updated[index] = { ...updated[index], [field]: value };
    setOverrides(updated);
  };

  const removeOverride = (index: number) => {
    setOverrides(overrides.filter((_, i) => i !== index));
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <h1>Project Configuration</h1>
        <div style={{ display: "flex", gap: 10 }}>
          <button className="secondary" onClick={() => navigate(`/router/${id}`)}>
            Generate Config
          </button>
          <button onClick={() => setShowAddModal(true)}>Activate MCP</button>
        </div>
      </div>

      <div>
        {bindings.map((binding) => {
          const mcp = allMcps.find((m) => m.id === binding.mcp_id);
          return (
            <div key={binding.id} className="card">
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                <div>
                  <h3>{mcp?.name || "Unknown MCP"}</h3>
                  <span className={`badge ${binding.enabled ? "success" : "warning"}`}>
                    {binding.enabled ? "Enabled" : "Disabled"}
                  </span>
                </div>
                <div style={{ display: "flex", gap: 10 }}>
                  <button className="secondary" onClick={() => handleEditOverrides(binding)}>
                    Config
                  </button>
                  <button onClick={() => handleToggle(binding)}>
                    {binding.enabled ? "Disable" : "Enable"}
                  </button>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {showAddModal && (
        <div className="modal">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Activate MCP</h2>
              <button className="close-btn" onClick={() => setShowAddModal(false)}>×</button>
            </div>
            <div className="form-group">
              <label>Select MCP</label>
              <select value={selectedMcpId} onChange={(e) => setSelectedMcpId(e.target.value)}>
                <option value="">-- Select --</option>
                {allMcps.map((mcp) => (
                  <option key={mcp.id} value={mcp.id}>
                    {mcp.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="button-group">
              <button onClick={handleActivate}>Activate</button>
              <button className="secondary" onClick={() => setShowAddModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}

      {editingBinding && (
        <div className="modal">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Edit Overrides</h2>
              <button className="close-btn" onClick={() => setEditingBinding(null)}>×</button>
            </div>
            <div style={{ marginBottom: 20 }}>
              {overrides.map((override, index) => (
                <div key={index} style={{ marginBottom: 10, padding: 10, backgroundColor: "#1a1a1a", borderRadius: 5 }}>
                  <input
                    value={override.key}
                    onChange={(e) => updateOverride(index, "key", e.target.value)}
                    placeholder="Key"
                    style={{ marginBottom: 5 }}
                  />
                  <input
                    value={override.value}
                    onChange={(e) => updateOverride(index, "value", e.target.value)}
                    placeholder="Value"
                    style={{ marginBottom: 5 }}
                  />
                  <label style={{ display: "flex", alignItems: "center", gap: 5 }}>
                    <input
                      type="checkbox"
                      checked={override.is_secret}
                      onChange={(e) => updateOverride(index, "is_secret", e.target.checked)}
                    />
                    Is Secret
                  </label>
                  <button className="danger" onClick={() => removeOverride(index)} style={{ marginTop: 5 }}>
                    Remove
                  </button>
                </div>
              ))}
              <button onClick={addOverride} style={{ marginTop: 10 }}>Add Override</button>
            </div>
            <div className="button-group">
              <button onClick={handleSaveOverrides}>Save</button>
              <button className="secondary" onClick={() => setEditingBinding(null)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default ProjectDetail;
