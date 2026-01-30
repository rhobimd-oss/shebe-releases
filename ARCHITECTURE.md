# Shebe Releases - Architecture

**Purpose:** Distribution hub for Shebe binaries and editor extensions.

**Updated:** 2026-01-30

---

## Overview

This repository contains no source code for Shebe itself. It manages
packaging and publication of pre-built binaries across three channels:
Homebrew, Zed extension and VS Code extension.

```
  github.com/rhobimd-oss/shebe
         │
         │  Tag push (e.g. v0.7.0)
         │
         ▼
  ┌───────────────────────────┐
  │  GitHub Actions           │
  │  (shebe repo)             │
  │  - Build Linux x86_64     │
  │  - Build macOS binaries   │
  │  - Create GitHub release  │
  └────────────┬──────────────┘
               │
               │  Release published
               ▼
  ┌──────────────────────────────────┐
  │  Manual / Automatic Updates      │
  │                                  │
  │  Homebrew: manual formula update │
  │  Zed: auto via registry submodule│
  │  VS Code: publish with vsce      │
  └──────────────────────────────────┘
```

---

## Channel Details

### Homebrew

**Directory:** `Formula/`

**How it works:**

- `Formula/shebe.rb` is a standard Homebrew formula
- Points to release tarballs on GitHub
- Installs two binaries: `shebe` and `shebe-mcp`
- SHA256 checksums verified from release assets

**Update flow:**

1. New shebe release published on GitHub
2. Maintainer updates version and SHA256 values in `Formula/shebe.rb`
3. Change committed via feature branch and MR

**User install:**

```bash
brew tap rhobimd-oss/shebe-releases
brew install shebe
```

Homebrew automatically finds the `Formula/` directory in any repo
named with the `homebrew-` prefix or tapped explicitly.

### Zed Extension

**Directory:** `extensions/zed/`

**How it works:**

- Zed extensions can register MCP servers
- The extension downloads the correct `shebe-mcp` binary for the
  user's platform on first use
- Binary is cached in the extension's data directory

**Key files:**

```
extensions/zed/
├── extension.toml       # Extension metadata (name, version, etc.)
├── src/
│   └── lib.rs           # Extension logic (binary download, MCP registration)
└── Cargo.toml           # Rust dependencies (Zed extension SDK)
```

**Publish flow:**

Zed extensions are distributed via submodules in
`zed-industries/extensions`. No publish workflow is needed.

1. Bump `version` in `extension.toml` and `Cargo.toml`
2. Push to the GitHub mirror
3. Zed's registry CI detects the version change and builds
   the extension from source automatically

### VS Code Extension

**Directory:** `extensions/vscode/`

**How it works:**

- VS Code extension that manages the `shebe-mcp` binary lifecycle
- Downloads platform-specific binary on activation
- Registers as an MCP server (via VS Code MCP support or bridge)
- Provides search commands via command palette

**Key files:**

```
extensions/vscode/
├── package.json         # Extension manifest
├── src/
│   └── extension.ts     # Activation, binary management, MCP registration
└── tsconfig.json
```

**Publish flow:**

1. `publish-vscode.yml` workflow triggered by new release
2. Packages the extension with `vsce`
3. Publishes to VS Code marketplace

---

## Binary Matrix

Each release produces binaries for these targets:

| Target | Built By | Used By |
|--------|----------|---------|
| `x86_64-unknown-linux-gnu` | GitHub Actions (shebe repo) | Homebrew (Linux) |
| `x86_64-apple-darwin` | GitHub Actions (shebe repo) | Homebrew (macOS Intel), Zed, VS Code |
| `aarch64-apple-darwin` | GitHub Actions (shebe repo) | Homebrew (macOS ARM), Zed, VS Code |

Each target produces two binaries:
- `shebe` - CLI for standalone use
- `shebe-mcp` - MCP server for editor integration

Release artifacts follow the naming convention:
```
shebe-v{VERSION}-{TARGET}.tar.gz
shebe-v{VERSION}-{TARGET}.tar.gz.sha256
```

---

## GitHub Actions Workflows

### `test-zed-extension.yml`

**Trigger:** Push or PR affecting `extensions/zed/` files

**Steps:**
1. Build Zed extension on Linux (musl + glibc) and macOS
2. Run integration tests against GitHub releases

---

## Design Decisions

### Why a separate repo?

- Homebrew taps must be standalone Git repositories
- Editor extensions benefit from independent release cycles
- Keeps the main shebe repo focused on source code
- Single place to manage all distribution concerns

### Why GitHub for distribution?

- Homebrew expects GitHub release URLs by convention
- Zed extension registry integrates with GitHub
- VS Code marketplace `vsce` tool works with any Git host but
  GitHub Actions simplifies CI

### Why download binaries at runtime (editor extensions)?

- Avoids bundling large binaries in extension packages
- Enables platform-specific binary selection
- Allows binary updates independent of extension updates
- Standard pattern used by rust-analyzer, gopls and similar tools

---

## Key Invariants

1. **No Shebe source code** - this repo only handles distribution
2. **All releases originate from shebe repo tags** - no manual
   releases from this repo
3. **Binary checksums are always verified** - SHA256 from release
   assets
4. **Platform detection is automatic** - extensions detect OS and
   architecture at runtime
5. **Version consistency** - all channels publish the same version
   simultaneously
