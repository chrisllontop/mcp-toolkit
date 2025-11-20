import { Routes, Route, Link } from "react-router-dom";
import { Toaster } from "react-hot-toast";
import Home from "./pages/home";
import ProjectDetail from "./pages/project-detail";
import McpCatalog from "./pages/mcp-catalog";
import Secrets from "./pages/secrets";
import "./app.css";

function App() {
  return (
    <div className="app">
      <Toaster position="top-right" />
      <nav className="sidebar">
        <h1>MCP Manager</h1>
        <ul>
          <li>
            <Link to="/">Projects</Link>
          </li>
          <li>
            <Link to="/catalog">MCP Catalog</Link>
          </li>
          <li>
            <Link to="/secrets">Secrets</Link>
          </li>
        </ul>
      </nav>
      <main className="content">
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/project/:id" element={<ProjectDetail />} />
          <Route path="/catalog" element={<McpCatalog />} />
          <Route path="/secrets" element={<Secrets />} />
        </Routes>
      </main>
    </div>
  );
}

export default App;
