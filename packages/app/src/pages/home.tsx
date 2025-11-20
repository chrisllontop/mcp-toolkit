import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import toast from "react-hot-toast";
import { api, Project } from "../api";

function Home() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const navigate = useNavigate();

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    const data = await api.listProjects();
    setProjects(data);
  };

  const handleCreate = async () => {
    try {
      await api.createProject(name, path);
      setShowModal(false);
      setName("");
      setPath("");
      loadProjects();
    } catch (error) {
      alert(`Error: ${error}`);
    }
  };

  const handleProjectClick = (project: Project) => {
    navigate(`/project/${project.id}`);
  };

  const handleDelete = async (e: React.MouseEvent, projectId: string) => {
    e.preventDefault();
    e.stopPropagation();
    console.log("Delete clicked for project:", projectId);

    if (window.confirm("Are you sure you want to delete this project?")) {
      console.log("User confirmed deletion");
      try {
        await api.deleteProject(projectId);
        console.log("Project deleted successfully");
        loadProjects();
      } catch (error) {
        console.error("Error deleting project:", error);
        alert(`Error: ${error}`);
      }
    } else {
      console.log("User cancelled deletion");
    }
  };

  const handleCopyConfig = async () => {
    try {
      const message = await api.copyMcpConfig();
      toast.success(message);
    } catch (error) {
      toast.error(`Error: ${error}`);
    }
  };

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 20 }}>
        <h1>Projects</h1>
        <div style={{ display: "flex", gap: 10 }}>
          <button onClick={handleCopyConfig}>Copy Config</button>
          <button onClick={() => setShowModal(true)}>Add Project</button>
        </div>
      </div>

      <div>
        {projects.map((project) => (
          <div key={project.id} className="card" onClick={() => handleProjectClick(project)} style={{ cursor: "pointer" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div>
                <h3>{project.name}</h3>
                <p style={{ color: "#888", fontSize: 14 }}>{project.path}</p>
              </div>
              <button
                type="button"
                className="danger"
                onClick={(e) => handleDelete(e, project.id)}
                style={{ marginLeft: 10 }}
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>

      {showModal && (
        <div className="modal">
          <div className="modal-content">
            <div className="modal-header">
              <h2>Add Project</h2>
              <button className="close-btn" onClick={() => setShowModal(false)}>×</button>
            </div>
            <div className="form-group">
              <label>Name</label>
              <input value={name} onChange={(e) => setName(e.target.value)} placeholder="My Project" />
            </div>
            <div className="form-group">
              <label>Path</label>
              <input value={path} onChange={(e) => setPath(e.target.value)} placeholder="/path/to/project" />
            </div>
            <div className="button-group">
              <button onClick={handleCreate}>Create</button>
              <button className="secondary" onClick={() => setShowModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Home;
