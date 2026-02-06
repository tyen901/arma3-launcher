use crate::error::{Arma3Error, Result};
use crate::install::InstallKind;
use std::path::{Path, PathBuf};

const PROTON_ARMA3_CFG_REL: &str = "pfx/drive_c/users/steamuser/My Documents/Arma 3/Arma3.cfg";

pub(crate) fn default_cfg_path(kind: InstallKind, game_dir: &Path) -> Result<PathBuf> {
    match kind {
        #[cfg(target_os = "linux")]
        InstallKind::LinuxProton => Ok(proton_cfg_path_for_game_dir(game_dir)),

        #[cfg(not(target_os = "linux"))]
        InstallKind::LinuxProton => Err(Arma3Error::Parse {
            message: "LinuxProton cfg path is only supported on Linux builds".to_string(),
        }),

        InstallKind::LinuxNative => {
            let home = home_dir()?;
            Ok(home.join(".local/share/bohemiainteractive/arma3/GameDocuments/Arma 3/Arma3.cfg"))
        }
        InstallKind::WindowsNative => {
            let docs = dirs_next::document_dir().ok_or_else(|| Arma3Error::Parse {
                message: "cannot locate Documents directory".to_string(),
            })?;
            Ok(docs.join("Arma 3").join("Arma3.cfg"))
        }
    }
}

#[cfg(target_os = "linux")]
fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| Arma3Error::Parse {
            message: "HOME environment variable is not set".to_string(),
        })
}

#[cfg(not(target_os = "linux"))]
fn home_dir() -> Result<PathBuf> {
    Err(Arma3Error::Parse {
        message: "home_dir() is not supported on this platform".to_string(),
    })
}

#[cfg(target_os = "linux")]
fn proton_cfg_path_for_game_dir(game_dir: &Path) -> PathBuf {
    compatdata_dir_for_game_dir(game_dir).join(PROTON_ARMA3_CFG_REL)
}

#[cfg(target_os = "linux")]
fn compatdata_dir_for_game_dir(game_dir: &Path) -> PathBuf {
    if let Some(root) = steam_library_root_from_game_dir(game_dir) {
        return root.join("steamapps/compatdata/107410");
    }

    // Best-effort relative fallback without probing Steam roots.
    game_dir
        .join("../../../")
        .join("steamapps")
        .join("compatdata")
        .join("107410")
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
