# MCP Toolkit

**Manage your AI tools the way you work.**

MCP Toolkit helps you organize Model Context Protocol servers at the project level. Use different API keys for different clients, customize tool access per project, and keep everything organized in one clean desktop app.

---

## Features

🎨 **Visual Interface** - Manage your MCP servers through a clean desktop interface instead of editing JSON files.

🔗 **Project-Based Organization** - Connect MCPs to specific projects with independent configurations and environment variables.

🔐 **Secure Secrets** - Keep your API keys and credentials encrypted with AES-256-GCM encryption. Encryption keys are securely stored in your OS keychain (macOS Keychain, Windows Credential Manager, or Linux Secret Service).

🌐 **Universal Support** - Works with Claude Desktop, Cursor, VS Code, Codex, and any MCP-compatible tool.

---

## How It Works

**1. Create Projects** - Organize your codebases into projects with isolated configurations.

**2. Add MCP Servers** - Import your existing servers or add new ones (supports Docker, Binary, and HTTP).

**3. Configure Bindings** - Connect servers to projects and customize environment variables as needed.

**4. Integrate** - Point your AI tool to MCP Toolkit and you're ready to go.

---

## Getting Started

**Installation**

Download the latest release for your platform:

- **Windows**: Download the `.msi` installer and run it
- **macOS**: Download the `.dmg` file, open it, and drag the app to Applications
- **Linux**: Download the `.AppImage` file, make it executable (`chmod +x`), and run it

**Configuration**

Add MCP Toolkit to your AI tool's configuration. The command path varies by platform:

**macOS:**
```json
{
  "mcpServers": {
    "mcp-toolkit": {
      "command": "/Applications/MCP Toolkit.app/Contents/MacOS/mcp-toolkit",
      "args": []
    }
  }
}
```

**Windows:**
```json
{
  "mcpServers": {
    "mcp-toolkit": {
      "command": "C:\\Users\\YourUsername\\AppData\\Local\\MCP Toolkit\\mcp-toolkit.exe",
      "args": []
    }
  }
}
```

**Linux:**
```json
{
  "mcpServers": {
    "mcp-toolkit": {
      "command": "/home/yourusername/.local/share/MCP Toolkit/mcp-toolkit",
      "args": []
    }
  }
}
```

> **Tip**: The exact path depends on your installation. Use the "Copy Config" button in MCP Toolkit to get the correct path for your system.

Once configured, open MCP Toolkit and start managing your servers.

## Security

MCP Toolkit uses **OS-native keychain** storage for encryption keys:

- **macOS**: Stored in macOS Keychain
- **Windows**: Stored in Windows Credential Manager
- **Linux**: Stored in Secret Service (gnome-keyring/KWallet)

This provides better security than file-based storage, as your encryption keys are protected by your OS's security mechanisms and can optionally require biometric authentication.

**Note**: MCP Toolkit requires keychain access to function. On first launch, you may be prompted to grant access to the keychain.

---

## Development

If you'd like to contribute or run the project locally:

```bash
pnpm install        # Install dependencies
pnpm tauri dev      # Start development server
pnpm tauri build    # Build for production
```

---

MIT © 2025 | Christian Llontop
