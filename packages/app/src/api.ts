import { invoke } from "@tauri-apps/api/core";

export interface Project {
  id: string;
  name: string;
  path: string;
  created_at: string;
}

export interface Mcp {
  id: string;
  name: string;
  mcp_type: "Docker" | "Binary" | "Http";
  config: McpConfig;
  created_at: string;
}

export interface McpConfig {
  docker_image?: string;
  binary_path?: string;
  http_url?: string;
  command?: string;
  args: string[];
  env_vars: EnvVar[];
}

export interface EnvVar {
  key: string;
  value: string;
  is_secret: boolean;
}

export interface ProjectMcpBinding {
  id: string;
  project_id: string;
  mcp_id: string;
  enabled: boolean;
  overrides: EnvVar[];
}

export interface Secret {
  id: string;
  key: string;
  created_at: string;
}

export const api = {
  async createProject(name: string, path: string): Promise<Project> {
    return await invoke("create_project", { name, path });
  },

  async listProjects(): Promise<Project[]> {
    return await invoke("list_projects");
  },

  async deleteProject(id: string): Promise<void> {
    return await invoke("delete_project", { id });
  },

  async detectAiConfig(projectPath: string): Promise<string | null> {
    return await invoke("detect_ai_config", { projectPath });
  },

  async createMcp(
    name: string,
    mcpType: "Docker" | "Binary" | "Http",
    config: McpConfig
  ): Promise<Mcp> {
    return await invoke("create_mcp", { name, mcpType, config });
  },

  async listMcps(): Promise<Mcp[]> {
    return await invoke("list_mcps");
  },

  async updateMcp(mcp: Mcp): Promise<void> {
    return await invoke("update_mcp", { mcp });
  },

  async deleteMcp(id: string): Promise<void> {
    return await invoke("delete_mcp", { id });
  },

  async activateMcp(
    projectId: string,
    mcpId: string,
    overrides: EnvVar[]
  ): Promise<ProjectMcpBinding> {
    return await invoke("activate_mcp", { projectId, mcpId, overrides });
  },

  async listBindings(projectId: string): Promise<ProjectMcpBinding[]> {
    return await invoke("list_bindings", { projectId });
  },

  async updateBinding(binding: ProjectMcpBinding): Promise<void> {
    return await invoke("update_binding", { binding });
  },

  async saveSecret(key: string, value: string): Promise<Secret> {
    return await invoke("save_secret", { key, value });
  },

  async listSecrets(): Promise<Secret[]> {
    return await invoke("list_secrets");
  },

  async parseMcpJson(jsonStr: string): Promise<ImportPreview[]> {
    return await invoke("parse_mcp_json_command", { jsonStr });
  },

  async importMcpsFromJson(jsonStr: string): Promise<Mcp[]> {
    return await invoke("import_mcps_from_json", { jsonStr });
  },

  async generateMcpConfig(projectId: string): Promise<string> {
    return await invoke("generate_mcp_config", { projectId });
  },
};

export interface ImportPreview {
  name: string;
  mcp_type: "Docker" | "Binary" | "Http";
  command?: string;
  args: string[];
}
