use std::{env, fs, path::Path};
use zed_extension_api::{
    self as zed, LanguageServerId, Result,
    serde_json::{self},
    settings::LspSettings,
};

// the general expected server path (excluded for windows)
const SERVER_PATH: &str = "node_modules/oxlint/bin/oxc_language_server";

const PACKAGE_NAME: &str = "oxlint";

struct OxcExtension;

impl OxcExtension {
    fn server_exists(&self, path: &Path) -> bool {
        fs::metadata(path).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // This is a workaround, as reading the file from wasm doesn't work.
        // Instead we try to read the `package.json`, see if `oxlint` is installed
        let package_json = worktree
            .read_text_file("package.json")
            .unwrap_or(String::from(r#"{}"#));
        let package_json: Option<serde_json::Value> =
            serde_json::from_str(package_json.as_str()).ok();

        let server_package_exists = package_json.is_some_and(|f| {
            !f["dependencies"][PACKAGE_NAME].is_null()
                || !f["devDependencies"][PACKAGE_NAME].is_null()
        });

        let is_windows = zed::current_platform().0 == zed::Os::Windows;

        // On Windows, the direct server path is never used because Windows always requires the `.CMD` wrapper
        // from the `.bin` directory. Therefore, we only use the direct server path on non-Windows platforms.
        if server_package_exists && !is_windows {
            let worktree_root_path = worktree.root_path();
            let path = Path::new(worktree_root_path.as_str())
                .join(SERVER_PATH)
                .to_string_lossy()
                .to_string();
            return Ok(path);
        }

        // fallback to extension owned biome
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let fallback_server_path = Path::new(if is_windows {
            "./node_modules/.bin/oxc_language_server.CMD"
        } else {
            "./node_modules/.bin/oxc_language_server"
        });
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !self.server_exists(fallback_server_path)
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists(fallback_server_path) {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{fallback_server_path:?}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists(fallback_server_path) {
                        Err(format!(
                            "failed to install package '{PACKAGE_NAME}': {error}"
                        ))?;
                    }
                }
            }
        }

        Ok(fallback_server_path.to_string_lossy().to_string())
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
        let path = self.server_script_path(language_server_id, worktree)?;
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        let args = vec![];

        let bin = env::current_dir()
            .unwrap()
            .join(path)
            .to_string_lossy()
            .to_string();

        if let Some(binary) = settings.binary {
            return Ok(zed::Command {
                command: binary.path.map_or(bin, |path| path),
                args: binary.arguments.map_or(args, |args| args),
                env: Default::default(),
            });
        }

        Ok(zed::Command {
            command: bin,
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
