# Shebe Code Search - Zed Extension

Fast, local code search powered by [BM25][bm25]. No embeddings, no GPU,
no cloud. This extension downloads and registers the
[shebe-mcp](https://github.com/rhobimd-oss/shebe) binary as a context
server (MCP server) in Zed's Agent Panel.

Research shows [70-85% of developer code search][research] value comes
from keyword-based queries. Developers search with exact terms they know:
function names, API calls and error messages. Shebe delivers 2ms query
latency at ~2,000-12,000 files/sec indexing speed, entirely offline.

**Trade-offs:**
- Repositories must be cloned locally before indexing (no remote URLs)
- No semantic similarity: "login" does not match "authenticate". Use
  boolean queries to cover synonyms (`login OR authenticate OR sign-in`)

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

### Searching Code

Once indexed, search across your codebase with keyword queries:

```
Search for handleLogin in the codebase
```

Boolean operators work for broader searches:

```
Find references to auth AND (session OR token)
```

### Finding References

Before renaming a symbol, discover all usages:

```
Find all references to AuthorizationPolicy
```

Shebe returns results ranked by confidence with pattern
classification (type annotation, function call, instantiation etc).

---

## MCP Tools

### Core

| Tool | Description |
|------|-------------|
| `search_code` | BM25 full-text search (2ms latency, 200-700 tokens) |
| `index_repository` | Index a directory (2k-12k files/sec) |
| `find_references` | Find symbol references with confidence scores |
| `list_sessions` | List all indexed sessions |
| `get_session_info` | Session metadata and statistics |
| `get_server_info` | Server version and capabilities |
| `show_shebe_config` | Display current configuration |

### Ergonomic

| Tool | Description |
|------|-------------|
| `read_file` | Read file contents (auto-truncated to 20KB) |
| `find_file` | Find files by glob or regex pattern |
| `list_dir` | List directory contents (paginated) |
| `preview_chunk` | Show context around an indexed chunk |
| `reindex_session` | Re-index using stored repository path |
| `delete_session` | Remove a session and its index |
| `upgrade_session` | Upgrade session schema to current version |

---

## Configuration

Shebe works out-of-the-box with sensible defaults. For tuning,
set environment variables or create `~/.config/shebe/config.toml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `SHEBE_CHUNK_SIZE` | `512` | Characters per chunk (100-2000) |
| `SHEBE_OVERLAP` | `64` | Overlap between chunks |
| `SHEBE_DEFAULT_K` | `10` | Default search results count |
| `SHEBE_MAX_K` | `100` | Maximum results allowed |
| `SHEBE_DATA_DIR` | `~/.local/share/shebe` | Index storage location |

See [CONFIGURATION.md][config] for the full reference.

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

- [Shebe](https://github.com/rhobimd-oss/shebe) - Source code
  and documentation
- [Benchmarks](https://github.com/rhobimd-oss/shebe/blob/main/WHY_SHEBE.md) -
  Comparisons against grep/ripgrep and Serena MCP
- [Configuration Guide][config] - All configuration options
- [MCP Tools Reference](https://github.com/rhobimd-oss/shebe/blob/main/docs/guides/mcp-tools-reference.md) -
  Detailed API for all 14 tools
- [shebe-releases](https://github.com/rhobimd-oss/shebe-releases) -
  Distribution hub

---

## License

See [LICENSE](https://github.com/rhobimd-oss/shebe/blob/main/LICENSE).

[bm25]: https://en.wikipedia.org/wiki/Okapi_BM25
[research]: https://research.google/pubs/how-developers-search-for-code-a-case-study/
[config]: https://github.com/rhobimd-oss/shebe/blob/main/CONFIGURATION.md
