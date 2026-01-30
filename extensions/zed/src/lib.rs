use std::env;
use zed_extension_api::{
    self as zed, ContextServerId, Project,
};

struct ShebeExtension {
    cached_binary_path: Option<String>,
}

impl ShebeExtension {
    fn get_or_download_binary(&self) -> zed::Result<String> {
        if let Some(path) = &self.cached_binary_path {
            return Ok(path.clone());
        }

        let release = zed::latest_github_release(
            "rhobimd-oss/shebe",
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let (os, arch) = zed::current_platform();

        let os_str = match os {
            zed::Os::Mac => "darwin",
            zed::Os::Linux => "linux",
            zed::Os::Windows => {
                return Err(
                    "shebe does not support Windows".into()
                );
            }
        };

        let arch_str = match arch {
            zed::Architecture::Aarch64 => {
                if os_str == "linux" {
                    return Err(
                        "shebe does not support Linux ARM"
                            .into()
                    );
                }
                "aarch64"
            }
            zed::Architecture::X8664 => "x86_64",
            zed::Architecture::X86 => {
                return Err(
                    "shebe does not support 32-bit x86"
                        .into()
                );
            }
        };

        let suffix = if os_str == "linux" {
            "-musl"
        } else {
            ""
        };

        let asset_name = format!(
            "shebe-{}-{}-{}{}.tar.gz",
            release.version, os_str, arch_str, suffix,
        );

        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "no release asset matching '{}'",
                    asset_name,
                )
            })?;

        let extract_dir = format!(
            "shebe-{}",
            release.version,
        );

        zed::download_file(
            &asset.download_url,
            &extract_dir,
            zed::DownloadedFileType::GzipTar,
        )?;

        let binary_path = format!(
            "{}/shebe-mcp",
            extract_dir,
        );

        zed::make_file_executable(&binary_path)?;

        Ok(binary_path)
    }
}

impl zed::Extension for ShebeExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn context_server_command(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> zed::Result<zed::Command> {
        let binary_path =
            self.get_or_download_binary()?;
        self.cached_binary_path =
            Some(binary_path.clone());

        let full_path = env::current_dir()
            .unwrap()
            .join(&binary_path)
            .to_string_lossy()
            .to_string();

        Ok(zed::Command {
            command: full_path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(ShebeExtension);
