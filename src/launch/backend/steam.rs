use crate::error::Result;
use crate::launch::backend::{collect_env, Backend, BackendParams};
use crate::launch::plan::CommandSpec;
use crate::steam::{ARMA3_APP_ID_STR, STEAM_ARG_APPLAUNCH, STEAM_ARG_NO_LAUNCHER};
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SteamBackend;

impl Backend for SteamBackend {
    fn plan(&self, params: &BackendParams<'_>) -> Result<CommandSpec> {
        let env = collect_env(params.user_env);
        let mut args: Vec<OsString> = Vec::new();

        #[cfg(target_os = "linux")]
        let (program, mut prefix_args) = if crate::steam::detect::is_flatpak_steam() {
            (
                PathBuf::from("flatpak"),
                vec![
                    OsString::from("run"),
                    OsString::from("com.valvesoftware.Steam"),
                ],
            )
        } else {
            let exe = crate::steam::detect::detect_steam_exe();
            (exe, vec![])
        };

        #[cfg(target_os = "windows")]
        let (program, mut prefix_args) = {
            let exe = crate::steam::detect::detect_steam_exe();
            (exe, vec![])
        };

        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        let (program, mut prefix_args) = {
            let _ = params;
            (PathBuf::from("steam"), vec![])
        };

        prefix_args.push(OsString::from(STEAM_ARG_APPLAUNCH));
        prefix_args.push(OsString::from(ARMA3_APP_ID_STR));
        prefix_args.push(OsString::from(STEAM_ARG_NO_LAUNCHER));
        prefix_args.extend_from_slice(params.user_args);
        args.extend(prefix_args);

        Ok(CommandSpec {
            program,
            args,
            cwd: params
                .working_dir
                .map(|p| p.to_path_buf())
                .or_else(|| Some(params.install.game_dir().to_path_buf())),
            env,
        })
    }
}
