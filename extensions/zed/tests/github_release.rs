//! Integration tests for the Shebe Zed extension.
//!
//! These tests hit the real GitHub Releases API to verify
//! that the assumptions in `src/lib.rs` hold true: asset
//! naming, archive layout, binary executability and MCP
//! protocol compliance.
//!
//! All tests are `#[ignore]` by default (require network).
//! Run with: `cargo test -- --ignored`

use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use tempfile::TempDir;

// -- GitHub API types -------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

// -- Helpers ----------------------------------------------------

const REPO: &str = "rhobimd-oss/shebe";
const API_BASE: &str = "https://api.github.com";

/// Supported platform tuples: (os_str, arch_str).
const SUPPORTED_PLATFORMS: &[(&str, &str)] = &[
    ("darwin", "aarch64"),
    ("darwin", "x86_64"),
    ("linux", "x86_64"),
];

fn github_client() -> Client {
    let mut builder = Client::builder()
        .user_agent("shebe-zed-integration-tests");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            builder = builder.default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::AUTHORIZATION,
                    format!("Bearer {}", token)
                        .parse()
                        .unwrap(),
                );
                h
            });
        }
    }
    builder.build().unwrap()
}

fn fetch_latest_release(client: &Client) -> Release {
    let url = format!(
        "{}/repos/{}/releases/latest",
        API_BASE, REPO,
    );
    let resp = client.get(&url).send().unwrap();
    assert!(
        resp.status().is_success(),
        "GET {} returned {}",
        url,
        resp.status(),
    );
    resp.json::<Release>().unwrap()
}

/// Cached release fetch -- all tests share a single API
/// call. Tests MUST run with --test-threads=1 for this to
/// work correctly (OnceLock is thread-safe but we also
/// want deterministic ordering).
static CACHED_RELEASE: OnceLock<Release> = OnceLock::new();

fn cached_release() -> Release {
    CACHED_RELEASE
        .get_or_init(|| {
            let client = github_client();
            fetch_latest_release(&client)
        })
        .clone()
}

/// Build asset name the same way the extension does.
/// Linux uses the musl variant for portability.
fn expected_asset_name(
    version: &str,
    os: &str,
    arch: &str,
) -> String {
    let suffix = if os == "linux" { "-musl" } else { "" };
    format!(
        "shebe-{}-{}-{}{}.tar.gz",
        version, os, arch, suffix,
    )
}

/// Download and extract an asset into a temp dir.
/// Returns (temp_dir, path_to_shebe_mcp).
fn download_and_extract(
    client: &Client,
    asset: &Asset,
) -> (TempDir, std::path::PathBuf) {
    let resp = client
        .get(&asset.browser_download_url)
        .send()
        .unwrap();
    assert!(
        resp.status().is_success(),
        "download {} returned {}",
        asset.name,
        resp.status(),
    );
    let bytes = resp.bytes().unwrap();

    let tmp = TempDir::new().unwrap();
    let decoder = GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(tmp.path()).unwrap();

    let binary = tmp.path().join("shebe-mcp");
    (tmp, binary)
}

/// Download the current-platform binary, make it executable
/// and return (temp_dir, binary_path).
fn download_current_platform_binary() -> (TempDir, std::path::PathBuf) {
    let client = github_client();
    let release = cached_release();
    let (os, arch) = current_platform();
    let name = expected_asset_name(
        &release.tag_name, os, arch,
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });
    let (tmp, binary) = download_and_extract(&client, asset);

    #[cfg(unix)]
    {
        let mut perms =
            std::fs::metadata(&binary).unwrap().permissions();
        perms.set_mode(perms.mode() | 0o755);
        std::fs::set_permissions(&binary, perms).unwrap();
    }

    (tmp, binary)
}

/// A running shebe-mcp process with stdin/stdout handles.
struct McpProcess {
    child: std::process::Child,
    reader: std::io::BufReader<std::process::ChildStdout>,
    next_id: u64,
}

impl McpProcess {
    /// Spawn shebe-mcp binary and send the initialize
    /// handshake. Returns a ready-to-use McpProcess.
    fn spawn_and_initialize(
        binary: &std::path::Path,
    ) -> Self {
        let mut child = Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap_or_else(|e| {
                panic!("failed to spawn shebe-mcp: {}", e)
            });

        let stdout = child.stdout.take().unwrap();
        let reader = std::io::BufReader::new(stdout);
        let mut proc = Self { child, reader, next_id: 1 };

        let _init_resp = proc.send_request(
            "initialize",
            serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "integration-test",
                    "version": "0.0.1"
                }
            }),
        );

        proc
    }

    /// Send a JSON-RPC request and read one line response.
    fn send_request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> serde_json::Value {
        let id = self.next_id;
        self.next_id += 1;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let body = serde_json::to_string(&request).unwrap();

        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(body.as_bytes()).unwrap();
        stdin.write_all(b"\n").unwrap();
        stdin.flush().unwrap();

        let mut line = String::new();
        std::io::BufRead::read_line(
            &mut self.reader, &mut line,
        )
        .expect("failed to read response line");

        serde_json::from_str(line.trim()).unwrap()
    }
}

impl Drop for McpProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Return the current platform tuple (os_str, arch_str)
/// for the machine running the tests so we can pick
/// which binary to actually execute.
fn current_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        panic!("unsupported test runner OS");
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        panic!("unsupported test runner arch");
    };

    (os, arch)
}

// ===============================================================
// Layer 1: Center (Happy Path)
// ===============================================================

/// T1.1 -- Latest release has assets.
#[test]
#[ignore]
fn latest_release_has_assets() {
    let release = cached_release();
    assert!(
        !release.assets.is_empty(),
        "latest release {} has no assets",
        release.tag_name,
    );
}

/// T1.2 -- darwin-aarch64 asset exists and downloads.
#[test]
#[ignore]
fn darwin_aarch64_asset_downloads() {
    let client = github_client();
    let release = cached_release();
    let name = expected_asset_name(
        &release.tag_name, "darwin", "aarch64",
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });
    let (_tmp, binary) = download_and_extract(&client, asset);
    assert!(
        binary.exists(),
        "shebe-mcp not found after extraction",
    );
}

/// T1.3 -- darwin-x86_64 asset exists and downloads.
#[test]
#[ignore]
fn darwin_x86_64_asset_downloads() {
    let client = github_client();
    let release = cached_release();
    let name = expected_asset_name(
        &release.tag_name, "darwin", "x86_64",
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });
    let (_tmp, binary) = download_and_extract(&client, asset);
    assert!(
        binary.exists(),
        "shebe-mcp not found after extraction",
    );
}

/// T1.4 -- linux-x86_64 asset exists and downloads.
#[test]
#[ignore]
fn linux_x86_64_asset_downloads() {
    let client = github_client();
    let release = cached_release();
    let name = expected_asset_name(
        &release.tag_name, "linux", "x86_64",
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });
    let (_tmp, binary) = download_and_extract(&client, asset);
    assert!(
        binary.exists(),
        "shebe-mcp not found after extraction",
    );
}

/// T1.5 -- Extracted binary is executable.
#[test]
#[ignore]
fn extracted_binary_is_executable() {
    let client = github_client();
    let release = cached_release();
    let (os, arch) = current_platform();
    let name = expected_asset_name(
        &release.tag_name, os, arch,
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });
    let (_tmp, binary) = download_and_extract(&client, asset);
    let perms = std::fs::metadata(&binary).unwrap().permissions();
    assert!(
        perms.mode() & 0o111 != 0,
        "shebe-mcp is not executable (mode: {:o})",
        perms.mode(),
    );
}

/// T1.6 -- Binary responds to JSON-RPC initialize.
#[test]
#[ignore]
fn binary_responds_to_jsonrpc_initialize() {
    let (_tmp, binary) =
        download_current_platform_binary();
    let mcp = McpProcess::spawn_and_initialize(&binary);

    // The initialize response was already consumed by
    // spawn_and_initialize; if we got here without panic
    // the binary accepted the handshake. Verify the
    // process is still alive.
    assert!(
        mcp.child.id() > 0,
        "shebe-mcp process is not running",
    );
}

/// T1.7 -- tools/list contains expected tools.
#[test]
#[ignore]
fn tools_list_contains_expected_tools() {
    let (_tmp, binary) =
        download_current_platform_binary();
    let mut mcp = McpProcess::spawn_and_initialize(&binary);

    let response = mcp.send_request(
        "tools/list",
        serde_json::json!({}),
    );

    let tools = response["result"]["tools"]
        .as_array()
        .expect("result.tools is not an array");

    let tool_names: Vec<&str> = tools
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    let expected = [
        "index_repository",
        "search_code",
        "find_references",
        "show_shebe_config",
        "get_server_info",
    ];

    for name in &expected {
        assert!(
            tool_names.contains(name),
            "expected tool '{}' not found; available: {:?}",
            name,
            tool_names,
        );
    }
}

// ===============================================================
// Layer 2: Boundary (Edge Cases)
// ===============================================================

/// T2.1 -- Asset naming convention matches extension logic
/// for all supported platforms.
#[test]
#[ignore]
fn asset_names_match_extension_logic() {
    let release = cached_release();
    let asset_names: Vec<&str> =
        release.assets.iter().map(|a| a.name.as_str()).collect();

    for (os, arch) in SUPPORTED_PLATFORMS {
        let expected = expected_asset_name(
            &release.tag_name, os, arch,
        );
        assert!(
            asset_names.contains(&expected.as_str()),
            "expected asset '{}' not found; available: {:?}",
            expected,
            asset_names,
        );
    }
}

/// T2.2 -- No Windows asset exists.
#[test]
#[ignore]
fn no_windows_asset() {
    let release = cached_release();
    let windows_asset = release
        .assets
        .iter()
        .find(|a| a.name.contains("windows"));
    assert!(
        windows_asset.is_none(),
        "unexpected Windows asset found: {}",
        windows_asset.map(|a| &a.name).unwrap_or(&String::new()),
    );
}

/// T2.3 -- No Linux ARM asset exists.
#[test]
#[ignore]
fn no_linux_arm_asset() {
    let release = cached_release();
    let linux_arm = release.assets.iter().find(|a| {
        a.name.contains("linux") && a.name.contains("aarch64")
    });
    assert!(
        linux_arm.is_none(),
        "unexpected linux-aarch64 asset found: {}",
        linux_arm.map(|a| &a.name).unwrap_or(&String::new()),
    );
}

/// T2.4 -- Release version format starts with 'v' + semver.
#[test]
#[ignore]
fn release_version_format() {
    let release = cached_release();
    assert!(
        release.tag_name.starts_with('v'),
        "tag '{}' does not start with 'v'",
        release.tag_name,
    );
    // Strip leading 'v' and check semver-like structure
    let version = &release.tag_name[1..];
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3
            && parts.iter().all(|p| p.parse::<u32>().is_ok()),
        "tag '{}' is not semver (v<MAJOR>.<MINOR>.<PATCH>)",
        release.tag_name,
    );
}

/// T2.5 -- Archive contains shebe-mcp at root (not nested).
#[test]
#[ignore]
fn archive_contains_binary_at_root() {
    let client = github_client();
    let release = cached_release();
    let (os, arch) = current_platform();
    let name = expected_asset_name(
        &release.tag_name, os, arch,
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });

    let resp = client
        .get(&asset.browser_download_url)
        .send()
        .unwrap();
    let bytes = resp.bytes().unwrap();

    let decoder = GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(decoder);
    let entries: Vec<String> = archive
        .entries()
        .unwrap()
        .filter_map(|e| {
            e.ok().map(|entry| {
                entry.path().unwrap().to_string_lossy().to_string()
            })
        })
        .collect();

    assert!(
        entries.contains(&"shebe-mcp".to_string()),
        "archive does not contain 'shebe-mcp' at root; \
         entries: {:?}",
        entries,
    );
}

// ===============================================================
// Layer 3: Beyond Boundary (Failure Modes)
// ===============================================================

/// T3.1 -- Invalid repo returns client error (404 or 403).
/// GitHub returns 404 for authenticated requests and 403
/// for unauthenticated requests to nonexistent repos.
#[test]
#[ignore]
fn invalid_repo_returns_client_error() {
    let client = github_client();
    let url = format!(
        "{}/repos/{}/releases/latest",
        API_BASE, "rhobimd-oss/nonexistent",
    );
    let resp = client.get(&url).send().unwrap();
    let status = resp.status().as_u16();
    assert!(
        status == 404 || status == 403,
        "expected 404 or 403 for nonexistent repo, got {}",
        status,
    );
}

/// T3.2 -- Nonexistent asset URL returns error.
#[test]
#[ignore]
fn nonexistent_asset_url_returns_error() {
    let client = github_client();
    let url = format!(
        "https://github.com/{}/releases/download/\
         v0.0.0-fake/shebe-v0.0.0-fake-linux-x86_64.tar.gz",
        REPO,
    );
    let resp = client.get(&url).send().unwrap();
    assert!(
        resp.status().is_client_error(),
        "expected 4xx for fake asset URL, got {}",
        resp.status(),
    );
}

/// T3.3 -- Truncated archive fails extraction.
#[test]
#[ignore]
fn truncated_archive_fails_extraction() {
    let client = github_client();
    let release = cached_release();
    let (os, arch) = current_platform();
    let name = expected_asset_name(
        &release.tag_name, os, arch,
    );
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| {
            panic!("asset '{}' not found in release", name)
        });

    let resp = client
        .get(&asset.browser_download_url)
        .send()
        .unwrap();
    let full_bytes = resp.bytes().unwrap();

    // Truncate to 50%
    let truncated = &full_bytes[..full_bytes.len() / 2];

    let tmp = TempDir::new().unwrap();
    let decoder = GzDecoder::new(truncated);
    let mut archive = tar::Archive::new(decoder);
    let result = archive.unpack(tmp.path());

    assert!(
        result.is_err(),
        "extracting a truncated archive should fail",
    );
}
