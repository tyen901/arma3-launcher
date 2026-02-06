use crate::error::{Arma3Error, Result};
use crate::steam::vdf::Vdf;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn compat_tool_shortname(steam_root: &Path, app_id: &str) -> Result<String> {
    let config_vdf_path = steam_root.join("config/config.vdf");
    let config_txt = fs::read_to_string(&config_vdf_path).map_err(|e| Arma3Error::SteamConfig {
        message: format!("failed reading {}: {e}", config_vdf_path.display()),
    })?;
    let vdf = Vdf::parse(&config_txt)?;

    let keys = [
        format!("InstallConfigStore/Software/Valve/Steam/CompatToolMapping/{app_id}/name"),
        format!("Software/Valve/Steam/CompatToolMapping/{app_id}/name"),
        format!("CompatToolMapping/{app_id}/name"),
    ];

    for key in keys {
        if let Some(v) = vdf.get(&key) {
            return Ok(v.clone());
        }
    }

    Err(Arma3Error::SteamConfig {
        message: format!("compatibility tool entry not found for appid {app_id}"),
    })
}

pub(crate) fn compat_tool_dir(steam_root: &Path, shortname: &str) -> Option<PathBuf> {
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

pub(crate) fn toolmanifest_commandline(tool_dir: &Path) -> Result<String> {
    let toolmanifest_path = tool_dir.join("toolmanifest.vdf");
    let tm_txt = fs::read_to_string(&toolmanifest_path).map_err(|e| Arma3Error::SteamConfig {
        message: format!("failed reading {}: {e}", toolmanifest_path.display()),
    })?;
    let tm = Vdf::parse(&tm_txt)?;
    tm.get("manifest/commandline")
        .cloned()
        .ok_or_else(|| Arma3Error::SteamConfig {
            message: format!(
                "toolmanifest missing manifest/commandline in {}",
                toolmanifest_path.display()
            ),
        })
}
