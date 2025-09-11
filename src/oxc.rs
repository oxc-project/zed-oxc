use std::{
    fs,
    path::{Path, PathBuf},
};
use zed_extension_api::{
    self as zed, LanguageServerId, Result,
    serde_json::{self},
    settings::LspSettings,
};

// the general expected server path (excluded for windows)
const WORKTREE_SERVER_PATH: &str = "node_modules/oxlint/bin/oxc_language_server";

const PACKAGE_NAME: &str = "oxlint";

struct OxcExtension;

impl OxcExtension {
    fn extension_server_exists(&self, path: &Path) -> bool {
        fs::metadata(path).is_ok_and(|stat| stat.is_file())
    }

    fn binary_specifier(&self) -> Result<String, String> {
        let (platform, arch) = zed::current_platform();

        let binary_name = match platform {
            zed::Os::Windows => "oxc_language_server.exe",
            _ => "oxc_language_server",
        };

        Ok(format!(
            "@oxlint/{platform}-{arch}/{binary}",
            platform = match platform {
                zed::Os::Mac => "darwin",
                zed::Os::Linux => "linux",
                zed::Os::Windows => "win32",
            },
            arch = match arch {
                zed::Architecture::Aarch64 => "arm64",
                zed::Architecture::X8664 => "x64",
                _ => return Err(format!("unsupported architecture: {arch:?}")),
            },
            binary = binary_name,
        ))
    }

    fn workspace_oxc_exists(&self, worktree: &zed::Worktree) -> bool {
        // This is a workaround, as reading the file from wasm doesn't work.
        // Instead we try to read the `package.json`, see if `oxlint` is installed
        let package_json = worktree
            .read_text_file("package.json")
            .unwrap_or(String::from(r#"{}"#));

        let package_json: Option<serde_json::Value> =
            serde_json::from_str(package_json.as_str()).ok();

        package_json.is_some_and(|f| {
            !f["dependencies"][PACKAGE_NAME].is_null()
                || !f["devDependencies"][PACKAGE_NAME].is_null()
        })
    }

    fn check_oxc_updates(&mut self, language_server_id: &LanguageServerId) -> Result<()> {
        // fallback to extension owned oxlint
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let extension_server_path = &Path::new("./node_modules").join(self.binary_specifier()?);
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !self.extension_server_exists(extension_server_path)
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.extension_server_exists(extension_server_path) {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{extension_server_path:?}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.extension_server_exists(extension_server_path) {
                        Err(format!(
                            "failed to install package '{PACKAGE_NAME}': {error}"
                        ))?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl zed_extension_api::Extension for OxcExtension {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed_extension_api::LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> zed_extension_api::Result<zed_extension_api::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        let mut args = vec![];

        // check and run oxlint with custom binary
        if let Some(binary) = settings.binary {
            return Ok(zed::Command {
                command: binary
                    .path
                    .map_or(WORKTREE_SERVER_PATH.to_string(), |path| path),
                args: binary.arguments.map_or(args, |args| args),
                env: Default::default(),
            });
        }

        // try to run oxlint with workspace oxc
        if self.workspace_oxc_exists(worktree) {
            let server_path = Path::new(worktree.root_path().as_str())
                .join(WORKTREE_SERVER_PATH)
                .to_string_lossy()
                .to_string();
            let mut node_args = vec![server_path];
            node_args.append(&mut args);

            return Ok(zed::Command {
                command: zed::node_binary_path()?,
                args: node_args,
                env: Default::default(),
            });
        }

        // install/update and run oxlint for extension
        self.check_oxc_updates(language_server_id)?;

        let mut server_path = PathBuf::from("./node_modules");
        server_path.push(self.binary_specifier()?);

        Ok(zed::Command {
            command: server_path.to_string_lossy().to_string(),
            args,
            env: Default::default(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings
            .initialization_options
            .and_then(|data| data.get("options").cloned()))
    }
    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed_extension_api::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.initialization_options)
    }
}

zed_extension_api::register_extension!(OxcExtension);
