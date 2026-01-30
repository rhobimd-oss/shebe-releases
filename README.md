# Shebe Releases

Distribution hub for [Shebe](https://github.com/rhobimd-oss/shebe) -
an MCP-based RAG service providing BM25 full-text search for code
repositories.

This repository manages release packaging and publication across
multiple distribution channels. The source code lives in the main
[shebe](https://github.com/rhobimd-oss/shebe) repository.

---

## Distribution Channels

| Channel | Status | Install Command |
|---------|--------|-----------------|
| Homebrew | Planned | `brew tap rhobimd-oss/shebe-releases && brew install shebe` |
| Zed Extension | Available | Install from Zed Extensions marketplace |
| VS Code Extension | Planned | Install from VS Code marketplace |

### Homebrew

Install both `shebe` (CLI) and `shebe-mcp` (MCP server) binaries:

```bash
brew tap rhobimd-oss/shebe-releases
brew install shebe
```

### Zed Extension

Search for "Shebe" in Zed's extension panel, or add to
`.zed/settings.json`:

```json
{
  "context_servers": {
    "shebe-mcp": {
      "command": {
        "path": "shebe-mcp"
      }
    }
  }
}
```

### VS Code Extension

Search for "Shebe" in the VS Code extensions marketplace.

---

## Repository Structure

```
shebe-releases/
├── Formula/
│   └── shebe.rb                  # Homebrew formula
├── extensions/
│   ├── zed/                      # Zed extension package
│   │   ├── extension.toml
│   │   └── ...
│   └── vscode/                   # VS Code extension package
│       ├── package.json
│       └── ...
├── .github/
│   └── workflows/
│       ├── update-formula.yml    # Update Homebrew on new release
│       ├── publish-zed.yml       # Publish Zed extension
│       └── publish-vscode.yml    # Publish VS Code extension
├── .gitlab-ci.yml                # GitLab CI (triggers GitHub Actions)
├── ARCHITECTURE.md               # Design and release flow
└── README.md                     # This file
```

---

## Release Flow

1. A new tag is pushed on the main
   [shebe](https://github.com/rhobimd-oss/shebe) repo
2. GitLab CI builds cross-platform binaries (Linux x86_64, macOS
   Intel and macOS ARM)
3. GitLab CI triggers this repo's GitHub Actions workflows
4. GitHub Actions updates the Homebrew formula and publishes
   editor extensions

No manual steps required after tagging.

---

## Supported Platforms

| Platform | Architecture | Homebrew | Zed | VS Code |
|----------|-------------|----------|-----|---------|
| macOS | ARM (Apple Silicon) | Yes | Yes | Yes |
| macOS | x86_64 (Intel) | Yes | Yes | Yes |
| Linux | x86_64 | Yes | Yes | Yes |

---

## Related Repositories

- [shebe](https://github.com/rhobimd-oss/shebe) - Source code
- [shebe-releases](https://github.com/rhobimd-oss/shebe-releases) -
  This repository (distribution hub)

---

## License

See the main [shebe](https://github.com/rhobimd-oss/shebe) repository
for license information.
