# Zed Extension Test Plan

**Document:** 001-zed-extension-test-plan-01.md
**Status:** Implemented
**Created:** 2026-01-30
**Related:** extensions/zed/src/lib.rs, extensions/zed/tests/github_release.rs
**Philosophy:** Follows shebe test envelope philosophy
(predict, test, validate -- center outward)

---

## Scope

The Zed extension (`ShebeExtension`) has two responsibilities:

1. **Binary acquisition** -- download the correct shebe-mcp
   binary from GitHub releases, extract it and make it
   executable
2. **Command construction** -- return the binary path as a
   `zed::Command` for the context server

---

## Testing Strategy

All tests are **integration tests** that hit the real GitHub
Releases API at `https://api.github.com/repos/rhobimd-oss/shebe/releases`.
This validates the full chain: API availability, asset naming
conventions, archive integrity, binary executability and MCP
protocol compliance.

### Test Harness

A standalone Rust integration test binary
(`tests/github_release.rs`) that:

- Calls the GitHub Releases API directly via `reqwest`
  (blocking client)
- Caches the release fetch in a `OnceLock<Release>` so all
  tests share a single API call
- Downloads real release archives
- Extracts and verifies binaries
- Spawns the binary and exercises the MCP JSON-RPC protocol
- Runs in CI with `cargo test --test github_release -- --ignored --test-threads=1`
  (not inside Zed WASM)

The tests do NOT use the Zed extension SDK. They replicate
the extension's logic against the real API to verify that
the assumptions encoded in `lib.rs` hold true.

### Key Helpers

- `github_client()` -- builds a `reqwest::blocking::Client`
  with optional `GITHUB_TOKEN` bearer auth
- `cached_release()` -- returns a `OnceLock`-cached latest
  release (single API call across all tests)
- `expected_asset_name(version, os, arch)` -- constructs the
  asset filename the same way `lib.rs` does, including the
  `-musl` suffix for Linux
- `download_and_extract(client, asset)` -- downloads and
  unpacks a tar.gz into a temp dir
- `download_current_platform_binary()` -- convenience wrapper
  that downloads the binary for the test runner's platform
- `McpProcess` -- spawns `shebe-mcp` with stdin/stdout pipes,
  sends JSON-RPC requests and reads newline-delimited
  responses. Handles initialize handshake on construction.
  Kills the child process on drop.

### Prerequisites

- Network access to `api.github.com` and `github.com`
- At least one published release in `rhobimd-oss/shebe`
  with assets
- `GITHUB_TOKEN` environment variable (optional, avoids
  rate limits)

### CI Configuration

Tests run in `.gitlab-ci.yml` as two parallel jobs:

- **test:musl** -- Alpine image
  (`rust-alpine:20260121-b1.88-alpine3.22`), exercises the
  musl-linked binary
- **test:glibc** -- Debian image
  (`rust-debian:20260123-b1.88-slim-trixie`), exercises the
  glibc-linked binary

Both jobs:
- Trigger on MR changes or default-branch pushes to
  `extensions/zed/` files (Cargo.lock, Cargo.toml, src, tests)
- Run `cargo test --test github_release -- --ignored --test-threads=1`
- Use a shared cargo registry cache (`zed-ext-deps` key)
- Have a 10-minute timeout

Local equivalents via Makefile:
- `make test-musl` -- runs in Alpine docker-compose service
- `make test-glibc` -- runs in Debian docker-compose service
- `make test` -- runs both

### Why --test-threads=1

All tests share a `OnceLock<Release>` cache to avoid
redundant API calls. Single-threaded execution ensures
deterministic test ordering (the first test to call
`cached_release()` populates the cache for all subsequent
tests).

---

## Test Envelope Layers

### Layer 1: Center (Happy Path)

**T1.1 -- Latest release has assets**
(`latest_release_has_assets`)
- Prediction: `GET /repos/rhobimd-oss/shebe/releases/latest`
  returns a release with a non-empty `assets` array
- Validates: The release pipeline is publishing assets

**T1.2 -- darwin-aarch64 asset exists and downloads**
(`darwin_aarch64_asset_downloads`)
- Prediction: The latest release contains an asset
  matching `shebe-{version}-darwin-aarch64.tar.gz`.
  Downloading and extracting it yields a file named
  `shebe-mcp`
- Validates: Apple Silicon binary is published correctly

**T1.3 -- darwin-x86_64 asset exists and downloads**
(`darwin_x86_64_asset_downloads`)
- Prediction: Same as T1.2 for `darwin-x86_64`
- Validates: Intel Mac binary is published correctly

**T1.4 -- linux-x86_64 asset exists and downloads**
(`linux_x86_64_asset_downloads`)
- Prediction: Same as T1.2 for `linux-x86_64-musl`
  (Linux uses the musl variant name:
  `shebe-{version}-linux-x86_64-musl.tar.gz`)
- Validates: Linux musl binary is published correctly

**T1.5 -- Extracted binary is executable**
(`extracted_binary_is_executable`)
- Prediction: After extracting the archive for the
  current platform, `shebe-mcp` has the executable bit
  set (mode & 0o111 != 0)
- Validates: Archive preserves file permissions

**T1.6 -- Binary responds to JSON-RPC initialize**
(`binary_responds_to_jsonrpc_initialize`)
- Prediction: Spawning `./shebe-mcp` and sending a
  newline-delimited JSON-RPC `initialize` request over
  stdin produces a valid response. The process remains
  alive after the handshake
- Validates: Binary is not corrupt, correct architecture
  for the test runner and speaks MCP protocol
- Note: shebe-mcp uses newline-delimited JSON framing
  (not HTTP Content-Length framing)

**T1.7 -- tools/list contains expected tools**
(`tools_list_contains_expected_tools`)
- Prediction: After initialize, sending `tools/list`
  returns a result containing at least:
  `index_repository`, `search_code`, `find_references`,
  `show_shebe_config` and `get_server_info`
- Validates: Binary exposes the expected MCP tool surface

### Layer 2: Boundary (Edge Cases)

**T2.1 -- Asset naming convention matches extension logic**
(`asset_names_match_extension_logic`)
- Prediction: For each supported platform tuple
  (darwin/aarch64, darwin/x86_64, linux/x86_64), the
  asset name constructed by `expected_asset_name()` --
  which mirrors the extension's format string including
  the `-musl` suffix for Linux -- matches an actual asset
  name in the release
- Validates: Extension naming logic stays in sync with
  the release pipeline

**T2.2 -- No Windows asset exists**
(`no_windows_asset`)
- Prediction: No asset in the latest release matches
  `*windows*`
- Validates: The extension's Windows rejection is
  consistent with what the release pipeline produces

**T2.3 -- No Linux ARM asset exists**
(`no_linux_arm_asset`)
- Prediction: No asset in the latest release matches
  both `*linux*` and `*aarch64*`
- Validates: The extension's Linux ARM rejection is
  consistent with the release pipeline

**T2.4 -- Release version format**
(`release_version_format`)
- Prediction: The release tag starts with `v` followed by
  exactly three dot-separated integers (semver).
  The extension uses this string directly in asset names
- Validates: Version format assumption in the extension

**T2.5 -- Archive contains shebe-mcp at root**
(`archive_contains_binary_at_root`)
- Prediction: Extracting the tar.gz and listing entries
  produces `shebe-mcp` at the root (not nested in
  subdirectories)
- Validates: The extension's `format!("{}/shebe-mcp", dir)`
  path construction is correct

### Layer 3: Beyond Boundary (Failure Modes)

**T3.1 -- Invalid repo returns client error**
(`invalid_repo_returns_client_error`)
- Prediction: `GET /repos/rhobimd-oss/nonexistent/releases/latest`
  returns HTTP 404 or 403 (GitHub returns 404 for
  authenticated requests and 403 for unauthenticated
  requests to nonexistent repos)
- Validates: A misconfigured repo name produces a clear
  failure rather than a confusing empty response

**T3.2 -- Nonexistent asset URL returns error**
(`nonexistent_asset_url_returns_error`)
- Prediction: Requesting a fabricated asset download URL
  (`v0.0.0-fake`) returns an HTTP 4xx status
- Validates: Missing assets fail loudly

**T3.3 -- Truncated archive fails extraction**
(`truncated_archive_fails_extraction`)
- Prediction: Downloading an asset but truncating the
  response body to 50% causes `tar::Archive::unpack()`
  to return an error (not produce a corrupt binary)
- Validates: Partial downloads are detected

**T3.4 -- Rate-limited request (observational)**
- Not implemented as a test
- When rate-limited (no auth token, exhausted quota), the
  API returns 403 with `X-RateLimit-Remaining: 0`
- Difficult to trigger reliably in CI; documented here
  as expected behavior

---

## Test File Structure

```
extensions/zed/
  tests/
    github_release.rs    # All integration tests (16 tests)
  Cargo.toml             # dev-dependencies: reqwest, serde,
                         #   serde_json, flate2, tar, tempfile
```

---

## References

- Test envelope philosophy:
  `shebe/docs/testing/000-test-envelope-philosophy.md`
- Extension source: `extensions/zed/src/lib.rs`
- CI configuration: `.gitlab-ci.yml` (test:musl, test:glibc)
- Local test runner: `Makefile` (test, test-musl, test-glibc)
- GitHub Releases API:
  `https://docs.github.com/en/rest/releases/releases`
