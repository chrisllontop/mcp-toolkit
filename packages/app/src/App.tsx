import { Routes, Route, Link } from "react-router-dom";
import Home from "./pages/Home";
import ProjectDetail from "./pages/ProjectDetail";
import McpCatalog from "./pages/McpCatalog";
import Secrets from "./pages/Secrets";
import "./App.css";

function App() {
  return (
    <div className="app">
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
