
// Mock Tauri IPC
(function() {
  // State storage
  const state = {
    projects: [],
    mcps: [],
    bindings: [],
    secrets: []
  };

  // Helper to generate IDs
  const generateId = () => Math.random().toString(36).substring(2, 9);

  // Mock implementation
  const mockInvoke = async (cmd, args) => {
    console.error(`[MockInvoke] ${cmd}`, JSON.stringify(args));

    switch (cmd) {
      // --- Projects ---
      case 'create_project': {
        const { name, path } = args;
        if (!name || !path) {
          throw new Error("Project name and path are required");
        }
        const project = {
          id: generateId(),
          name,
          path,
          created_at: new Date().toISOString()
        };
        state.projects.push(project);
        return project;
      }
      case 'list_projects':
        return state.projects;
      case 'delete_project': {
        const { id } = args;
        state.projects = state.projects.filter(p => p.id !== id);
        return null;
      }
      case 'detect_ai_config':
        return null; // Simulate no config detected by default

      // --- MCPs ---
      case 'create_mcp': {
        const { name, mcpType, config } = args;
        const mcp = {
          id: generateId(),
          name,
          mcp_type: mcpType,
          config,
          created_at: new Date().toISOString()
        };
        state.mcps.push(mcp);
        return mcp;
      }
      case 'list_mcps':
        return state.mcps;
      case 'update_mcp': {
        const { mcp } = args;
        const index = state.mcps.findIndex(m => m.id === mcp.id);
        if (index !== -1) {
          state.mcps[index] = mcp;
        }
        return null;
      }
      case 'delete_mcp': {
        const { id } = args;
        state.mcps = state.mcps.filter(m => m.id !== id);
        return null;
      }

      // --- Bindings ---
      case 'activate_mcp': {
        const { projectId, mcpId, overrides } = args;
        const binding = {
          id: generateId(),
          project_id: projectId,
          mcp_id: mcpId,
          enabled: true,
          overrides: overrides || []
        };
        state.bindings.push(binding);
        return binding;
      }
      case 'list_bindings': {
        const { projectId } = args;
        return state.bindings.filter(b => b.project_id === projectId);
      }
      case 'update_binding': {
        const { binding } = args;
        const index = state.bindings.findIndex(b => b.id === binding.id);
        if (index !== -1) {
          state.bindings[index] = binding;
        }
        return null;
      }

      // --- Secrets ---
      case 'save_secret': {
        const { key, value } = args;
        const existingIndex = state.secrets.findIndex(s => s.key === key);
        const secret = {
          id: existingIndex !== -1 ? state.secrets[existingIndex].id : generateId(),
          key,
          created_at: new Date().toISOString()
        };
        
        if (existingIndex !== -1) {
          state.secrets[existingIndex] = secret;
        } else {
          state.secrets.push(secret);
        }
        return secret;
      }
      case 'list_secrets':
        return state.secrets;

      // --- Import/Export ---
      case 'parse_mcp_json_command': {
        const { jsonStr } = args;
        console.error(`[MockInvoke] parse_mcp_json_command input:`, jsonStr);
        try {
          const parsed = JSON.parse(jsonStr);
          // Handle both simple object and mcpServers wrapper
          const servers = parsed.mcpServers || parsed;
          
          const result = Object.entries(servers).map(([name, config]) => {
            console.error(`[MockInvoke] Parsing config for ${name}:`, JSON.stringify(config));
            let mcp_type;
            if (config.command) mcp_type = 'Binary';
            else if (config.url || config.http_url) mcp_type = 'Http';
            else if (config.docker_image || config.docker) mcp_type = 'Docker';
            else throw new Error(`Invalid config for ${name}`);

            return {
              name,
              mcp_type,
              command: config.command || "",
              args: Array.isArray(config.args) ? config.args : [],
              ...config
            };
          });
          console.error('[MockInvoke] parse_mcp_json_command result:', JSON.stringify(result, null, 2));
          return result;
        } catch (e) {
          console.error("[MockInvoke] Parse error:", e);
          throw new Error("Failed to parse JSON: " + e.message);
        }
      }
      case 'import_mcps_from_json': {
        const { jsonStr } = args;
        try {
          const parsed = JSON.parse(jsonStr);
          const servers = parsed.mcpServers || parsed;
          const imported = [];
          
          for (const [name, config] of Object.entries(servers)) {
            const mcp = {
              id: generateId(),
              name,
              mcp_type: config.command ? 'Binary' : 'Http', // Simplified
              config: {
                command: config.command,
                args: config.args || [],
                env_vars: []
              },
              created_at: new Date().toISOString()
            };
            state.mcps.push(mcp);
            imported.push(mcp);
          }
          return imported;
        } catch (e) {
          throw new Error("Failed to import");
        }
      }
      case 'generate_mcp_config':
        return JSON.stringify({ mcpServers: {} }, null, 2);
      case 'copy_mcp_config':
        return JSON.stringify({ mcpServers: {} }, null, 2);

      default:
        console.warn(`[MockInvoke] Unknown command: ${cmd}`);
        return null;
    }
  };

  // Inject into window
  window.__TAURI_INTERNALS__ = { invoke: mockInvoke };
  window.__TAURI__ = { invoke: mockInvoke };
})();
