# Shebe Code Search - Zed Extension

BM25 full-text code search for AI agents via MCP. This extension
downloads and registers the
[shebe-mcp](https://gitlab.com/rhobimd-oss/shebe) binary as a
context server in Zed's Agent Panel.

---

## Installation

### From Marketplace

1. Open Zed
2. Go to **Extensions** (`Cmd+Shift+X` / `Ctrl+Shift+X`)
3. Search for "Shebe Code Search"
4. Click **Install**

### Dev Extension (Manual)

1. Clone the repository:
   ```bash
   git clone https://github.com/rhobimd-oss/shebe-releases.git
   ```
2. In Zed: **Extensions > Install Dev Extension**
3. Select the `extensions/zed` directory

---

## Usage

1. Open the **Agent Panel** (`Cmd+Shift+A` / `Ctrl+Shift+A`)
2. Enable the "Shebe Code Search" context server
3. The extension downloads the shebe-mcp binary automatically
   on first use
4. Use any of the 14 MCP tools in your agent conversations

### Indexing a Repository

Ask the agent to index your project:

```
Index this repository for code search
```

Shebe indexes at ~2,000+ files/sec with 2ms search latency.

### Searching Code

Once indexed, search across your codebase:

```
Search for authentication middleware in the codebase
```

---

## MCP Tools

The extension provides 14 tools through shebe-mcp:

| Tool | Description |
|------|-------------|
| `search_code` | BM25 full-text search across indexed files |
| `index_repository` | Index a directory for search |
| `reindex_session` | Re-index an existing session |
| `list_sessions` | List all active sessions |
| `get_session_info` | Get details about a session |
| `delete_session` | Remove a session and its index |
| `upgrade_session` | Upgrade session to current version |
| `read_file` | Read file contents from an indexed session |
| `find_file` | Find files by name pattern |
| `find_references` | Find symbol references across files |
| `list_dir` | List directory contents |
| `preview_chunk` | Preview indexed chunks for a file |
| `get_server_info` | Server version and status |
| `show_shebe_config` | Show current configuration |

---

## Supported Platforms

| Platform | Architecture | Supported |
|----------|-------------|-----------|
| macOS | ARM (Apple Silicon) | Yes |
| macOS | x86_64 (Intel) | Yes |
| Linux | x86_64 | Yes |

---

## Troubleshooting

### Extension not loading

Check the Zed log for errors:

```bash
# Linux
cat ~/.local/share/zed/logs/Zed.log | grep -i shebe

# macOS
cat ~/Library/Logs/Zed/Zed.log | grep -i shebe
```

### Binary download fails

The extension downloads from GitHub releases. Verify network
access to `github.com` and check DNS resolution:

```bash
dig github.com
curl -sI https://github.com/rhobimd-oss/shebe/releases
```

### Context server not appearing

1. Verify the extension is installed: **Extensions** panel
   should show "Shebe Code Search"
2. Open the Agent Panel and check the context server list
3. Restart Zed if the extension was recently installed

---

## Related

- [Shebe](https://gitlab.com/rhobimd-oss/shebe) - Source code
- [shebe-releases](https://github.com/rhobimd-oss/shebe-releases) -
  Distribution hub
