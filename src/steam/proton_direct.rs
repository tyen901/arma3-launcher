use crate::command::CommandSpec;
use crate::error::{Arma3Error, Result};
use crate::install::Arma3Install;
use crate::steam::consts::*;
use crate::steam::paths;
use crate::steam::steam_root;
use crate::steam::vdf::Vdf;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn build_proton_direct_spec(
    install: &Arma3Install,
    user_args: &[OsString],
    mut env: Vec<(OsString, OsString)>,
    working_dir: Option<&Path>,
    disable_esync: bool,
) -> Result<CommandSpec> {
    let steam_root = steam_root::detect_steam_root().ok_or(Arma3Error::SteamNotFound)?;

    let config_vdf_path = steam_root.join("config/config.vdf");
    let config_txt = fs::read_to_string(&config_vdf_path).map_err(|e| Arma3Error::SteamConfig {
        message: format!("failed reading {}: {e}", config_vdf_path.display()),
    })?;
    let vdf = Vdf::parse(&config_txt)?;

    let filter = format!("CompatToolMapping/{}/name", ARMA3_APP_ID_STR);
    let mut values = vdf.values_with_filter(&filter);
    let shortname = values.pop().ok_or_else(|| Arma3Error::SteamConfig {
        message: format!("compatibility tool entry not found for appid {ARMA3_APP_ID_STR}"),
    })?;

    let tool_dir =
        find_compat_tool_dir(&steam_root, &shortname).ok_or_else(|| Arma3Error::SteamConfig {
            message: format!("cannot find compatibility tool directory for '{shortname}'"),
        })?;

    let toolmanifest_path = tool_dir.join("toolmanifest.vdf");
    let tm_txt = fs::read_to_string(&toolmanifest_path).map_err(|e| Arma3Error::SteamConfig {
        message: format!("failed reading {}: {e}", toolmanifest_path.display()),
    })?;
    let tm = Vdf::parse(&tm_txt)?;
    let cmdline =
        tm.kv
            .get("manifest/commandline")
            .cloned()
            .ok_or_else(|| Arma3Error::SteamConfig {
                message: format!(
                    "toolmanifest missing manifest/commandline in {}",
                    toolmanifest_path.display()
                ),
            })?;

    let mut parts = shlex_split(&cmdline);
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

    if let Some(overlay) = steam_root::linux_overlay_so() {
        let mut ld_preload = overlay.to_string_lossy().to_string();
        if let Some(old) = std::env::var_os("LD_PRELOAD") {
            if !old.is_empty() {
                ld_preload.push(':');
                ld_preload.push_str(&old.to_string_lossy());
            }
        }
        env.push((OsString::from("LD_PRELOAD"), OsString::from(ld_preload)));
    }

    let compat_data = paths::compatdata_dir_for_game_dir(install.game_dir());

    env.push((
        OsString::from(ENV_STEAM_GAME_ID),
        OsString::from(ARMA3_APP_ID_STR),
    ));
    env.push((
        OsString::from(ENV_STEAM_COMPAT_DATA_PATH),
        OsString::from(compat_data.to_string_lossy().to_string()),
    ));

    if disable_esync {
        env.push((OsString::from(ENV_PROTON_NO_ESYNC), OsString::from("1")));
    }

    let mut args: Vec<OsString> = parts.into_iter().map(OsString::from).collect();
    args.push(OsString::from(
        install.executable().to_string_lossy().to_string(),
    ));
    args.extend_from_slice(user_args);

    if missing_libpng12() {
        if let Some(runsh) = steam_root::steam_runtime_runsh() {
            let mut wrapped_args = Vec::with_capacity(1 + args.len());
            wrapped_args.push(OsString::from(tool_program.to_string_lossy().to_string()));
            wrapped_args.extend(args);

            return Ok(CommandSpec {
                program: runsh,
                args: wrapped_args,
                cwd: working_dir
                    .map(|p| p.to_path_buf())
                    .or_else(|| Some(install.game_dir().to_path_buf())),
                env,
            });
        }
    }

    Ok(CommandSpec {
        program: tool_program,
        args,
        cwd: working_dir
            .map(|p| p.to_path_buf())
            .or_else(|| Some(install.game_dir().to_path_buf())),
        env,
    })
}

fn find_compat_tool_dir(steam_root: &Path, shortname: &str) -> Option<PathBuf> {
    let user = steam_root.join("compatibilitytools.d").join(shortname);
    if user.is_dir() {
        return Some(user);
    }
    let system = PathBuf::from("/usr/share/steam/compatibilitytools.d").join(shortname);
    if system.is_dir() {
        return Some(system);
    }
    None
}

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

fn shlex_split(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;

    for c in s.chars() {
        if escape {
            cur.push(c);
            escape = false;
            continue;
        }
        if c == '\\' && in_double {
            escape = true;
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if c.is_whitespace() && !in_single && !in_double {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
