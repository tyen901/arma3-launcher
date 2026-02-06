use crate::error::{Arma3Error, Result};
use crate::launch::backend::{collect_env, Backend, BackendParams};
use crate::launch::plan::CommandSpec;
use crate::steam::{
    compat, detect, runtime, ARMA3_APP_ID_STR, ENV_PROTON_NO_ESYNC, ENV_STEAM_COMPAT_DATA_PATH,
    ENV_STEAM_GAME_ID,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ProtonBackend;

impl Backend for ProtonBackend {
    fn plan(&self, params: &BackendParams<'_>) -> Result<CommandSpec> {
        #[cfg(target_os = "linux")]
        {
            build_proton_direct_spec(params)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = params;
            Err(Arma3Error::Parse {
                message: "proton direct launch is only supported on Linux".to_string(),
            })
        }
    }
}

#[cfg(target_os = "linux")]
fn build_proton_direct_spec(params: &BackendParams<'_>) -> Result<CommandSpec> {
    let steam_root = detect::detect_steam_root().ok_or(Arma3Error::SteamNotFound)?;

    let shortname = compat::compat_tool_shortname(&steam_root, ARMA3_APP_ID_STR)?;
    let tool_dir = compat::compat_tool_dir(&steam_root, &shortname).ok_or_else(|| {
        Arma3Error::SteamConfig {
            message: format!("cannot find compatibility tool directory for '{shortname}'"),
        }
    })?;

    let cmdline = compat::toolmanifest_commandline(&tool_dir)?;

    let mut parts = shell_words::split(&cmdline).map_err(|e| Arma3Error::SteamConfig {
        message: format!("failed parsing tool commandline: {e}"),
    })?;
    if parts.is_empty() {
        return Err(Arma3Error::SteamConfig {
            message: "empty tool commandline".into(),
        });
    }

    let tool_rel = parts.remove(0);
    let tool_program = tool_dir.join(tool_rel);

    for p in &mut parts {
        if p == "%verb%" {
            *p = "run".to_string();
        } else if p.contains("%verb%") {
            *p = p.replace("%verb%", "run");
        }
    }

    let mut env = collect_env(params.user_env);

    if let Some(overlay) = runtime::linux_overlay_so() {
        let mut ld_preload = overlay.to_string_lossy().to_string();
        if let Some(old) = std::env::var_os("LD_PRELOAD") {
            if !old.is_empty() {
                ld_preload.push(':');
                ld_preload.push_str(&old.to_string_lossy());
            }
        }
        env.push((OsString::from("LD_PRELOAD"), OsString::from(ld_preload)));
    }

    let compat_data = compatdata_dir_for_game_dir(params.install.game_dir(), &steam_root);

    env.push((
        OsString::from(ENV_STEAM_GAME_ID),
        OsString::from(ARMA3_APP_ID_STR),
    ));
    env.push((
        OsString::from(ENV_STEAM_COMPAT_DATA_PATH),
        OsString::from(compat_data.to_string_lossy().to_string()),
    ));

    if params.disable_esync {
        env.push((OsString::from(ENV_PROTON_NO_ESYNC), OsString::from("1")));
    }

    let mut args: Vec<OsString> = parts.into_iter().map(OsString::from).collect();
    args.push(OsString::from(
        params.install.executable().to_string_lossy().to_string(),
    ));
    args.extend_from_slice(params.user_args);

    if missing_libpng12() {
        if let Some(runsh) = runtime::steam_runtime_runsh() {
            let mut wrapped_args = Vec::with_capacity(1 + args.len());
            wrapped_args.push(OsString::from(tool_program.to_string_lossy().to_string()));
            wrapped_args.extend(args);

            return Ok(CommandSpec {
                program: runsh,
                args: wrapped_args,
                cwd: params
                    .working_dir
                    .map(|p| p.to_path_buf())
                    .or_else(|| Some(params.install.game_dir().to_path_buf())),
                env,
            });
        }
    }

    Ok(CommandSpec {
        program: tool_program,
        args,
        cwd: params
            .working_dir
            .map(|p| p.to_path_buf())
            .or_else(|| Some(params.install.game_dir().to_path_buf())),
        env,
    })
}

#[cfg(target_os = "linux")]
fn compatdata_dir_for_game_dir(game_dir: &Path, steam_root: &Path) -> PathBuf {
    if let Some(root) = steam_library_root_from_game_dir(game_dir) {
        return root
            .join("steamapps")
            .join("compatdata")
            .join(ARMA3_APP_ID_STR);
    }

    if steam_root.join("steamapps").is_dir() {
        return steam_root
            .join("steamapps")
            .join("compatdata")
            .join(ARMA3_APP_ID_STR);
    }

    game_dir
        .join("../../../")
        .join("steamapps")
        .join("compatdata")
        .join(ARMA3_APP_ID_STR)
}

#[cfg(target_os = "linux")]
fn steam_library_root_from_game_dir(game_dir: &Path) -> Option<PathBuf> {
    let common = game_dir.parent()?;
    if common.file_name()?.to_string_lossy() != "common" {
        return None;
    }
    let steamapps = common.parent()?;
    if steamapps.file_name()?.to_string_lossy() != "steamapps" {
        return None;
    }
    steamapps.parent().map(|p| p.to_path_buf())
}

#[cfg(target_os = "linux")]
fn missing_libpng12() -> bool {
    let candidates = [
        "/usr/lib/libpng12.so.0",
        "/usr/lib/libpng12.so",
        "/usr/lib64/libpng12.so.0",
        "/usr/lib64/libpng12.so",
        "/lib/libpng12.so.0",
        "/lib/libpng12.so",
        "/lib64/libpng12.so.0",
        "/lib64/libpng12.so",
        "/usr/lib/x86_64-linux-gnu/libpng12.so.0",
        "/usr/lib/x86_64-linux-gnu/libpng12.so",
    ];
    !candidates.iter().any(|p| Path::new(p).is_file())
}
