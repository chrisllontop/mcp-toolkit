import { useEffect, useState } from "react";
import { api, Secret } from "../api";

function Secrets() {
  const [secrets, setSecrets] = useState<Secret[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [key, setKey] = useState("");
  const [value, setValue] = useState("");

  useEffect(() => {
    loadSecrets();
  }, []);

  const loadSecrets = async () => {
    const data = await api.listSecrets();
    setSecrets(data);
  };

  const handleSave = async () => {
    try {
      await api.saveSecret(key, value);
      setShowModal(false);
      setKey("");
      setValue("");
      loadSecrets();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <h1>Secrets</h1>
        <button onClick={() => setShowModal(true)}>Add Secret</button>
      </div>

      <table>
        <thead>
          <tr>
            <th>Key</th>
            <th>Created At</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {secrets.map((secret) => (
            <tr key={secret.id}>
              <td>{secret.key}</td>
              <td>{new Date(secret.created_at).toLocaleString()}</td>
              <td>
                <span className="badge success">Configured</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {showModal && (
        <div className="modal">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Add Secret</h2>
              <button className="close-btn" onClick={() => setShowModal(false)}>×</button>
            </div>
            <div className="form-group">
              <label>Key</label>
              <input value={key} onChange={(e) => setKey(e.target.value)} placeholder="API_KEY" />
            </div>
            <div className="form-group">
              <label>Value</label>
              <input
                type="password"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                placeholder="secret value"
              />
            </div>
            <div className="button-group">
              <button onClick={handleSave}>Save</button>
              <button className="secondary" onClick={() => setShowModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Secrets;
