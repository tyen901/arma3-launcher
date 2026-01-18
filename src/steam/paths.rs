use crate::install::Arma3Install;
use crate::steam::consts::*;
use crate::steam::steam_root;
use crate::steam::vdf::Vdf;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn steamapps_dir(library_root: &Path) -> PathBuf {
    library_root.join("steamapps")
}

pub(crate) fn common_dir(library_root: &Path) -> PathBuf {
    steamapps_dir(library_root).join("common")
}

pub(crate) fn arma3_game_dir_in_library(library_root: &Path) -> PathBuf {
    common_dir(library_root).join(ARMA3_STEAM_GAME_DIR_NAME)
}

pub(crate) fn compatdata_dir_in_library(library_root: &Path) -> PathBuf {
    steamapps_dir(library_root)
        .join("compatdata")
        .join(ARMA3_APP_ID_STR)
}

#[allow(dead_code)]
pub(crate) fn workshop_content_dir_in_library(library_root: &Path) -> PathBuf {
    steamapps_dir(library_root)
        .join("workshop")
        .join("content")
        .join(ARMA3_APP_ID_STR)
}

/// Attempt to derive Steam library root from a canonical Steam install layout:
/// `<root>/steamapps/common/<Game>`.
pub(crate) fn steam_library_root_from_game_dir(game_dir: &Path) -> Option<PathBuf> {
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

/// Resolve compatdata dir for a given game_dir (best-effort).
/// Prefers deriving from the game dir, then falls back to detected Steam root.
pub(crate) fn compatdata_dir_for_game_dir(game_dir: &Path) -> PathBuf {
    if let Some(root) = steam_library_root_from_game_dir(game_dir) {
        return compatdata_dir_in_library(&root);
    }
    if let Some(root) = steam_root::detect_steam_root() {
        return compatdata_dir_in_library(&root);
    }

    game_dir
        .join("../../../")
        .join("steamapps")
        .join("compatdata")
        .join(ARMA3_APP_ID_STR)
}

/// Resolve Proton Arma3.cfg path for a Proton install (under compatdata/<appid>/...).
pub(crate) fn proton_cfg_path_for_game_dir(game_dir: &Path) -> PathBuf {
    compatdata_dir_for_game_dir(game_dir).join(PROTON_ARMA3_CFG_REL)
}

/// Read Steam library roots from `steamapps/libraryfolders.vdf` (best-effort).
pub(crate) fn detect_steam_library_roots() -> Vec<PathBuf> {
    let Some(root) = steam_root::detect_steam_root() else {
        return vec![];
    };

    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut out: Vec<PathBuf> = Vec::new();

    if root.join("steamapps").is_dir() && seen.insert(root.clone()) {
        out.push(root.clone());
    }

    let libraryfolders = root.join("steamapps/libraryfolders.vdf");
    let Ok(txt) = fs::read_to_string(&libraryfolders) else {
        return out;
    };
    let Ok(vdf) = Vdf::parse(&txt) else {
        return out;
    };

    for (k, v) in &vdf.kv {
        let kl = k.to_ascii_lowercase();
        if !kl.contains("libraryfolders/") {
            continue;
        }
        if !kl.ends_with("/path") {
            continue;
        }

        let p = PathBuf::from(v);
        if p.join("steamapps").is_dir() && seen.insert(p.clone()) {
            out.push(p);
        }
    }

    out
}

/// Convert a host path to the argument string expected by Arma.
/// For Proton, map into `Z:\` and use Windows-style separators.
pub fn arma_path_arg(path: &Path, is_proton: bool) -> String {
    if is_proton {
        let mut s = path.to_string_lossy().replace('/', "\\");
        if path.is_absolute() {
            s = format!("Z:{s}");
        }
        s
    } else {
        path.to_string_lossy().into_owned()
    }
}

/// Return validated Arma 3 install dir candidates, ordered by “most likely”.
/// - `ARMA3_DIR` env override (if valid)
/// - Steam library roots (libraryfolders.vdf)
/// - OS-specific default Steam locations (if they exist)
pub(crate) fn detect_arma3_install_candidates() -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();

    if let Some(p) = std::env::var_os("ARMA3_DIR").map(PathBuf::from) {
        if Arma3Install::new(p.clone(), None::<PathBuf>).is_ok() && seen.insert(p.clone()) {
            out.push(p);
        }
    }

    for lib in detect_steam_library_roots() {
        let game_dir = arma3_game_dir_in_library(&lib);
        if Arma3Install::new(game_dir.clone(), None::<PathBuf>).is_ok()
            && seen.insert(game_dir.clone())
        {
            out.push(game_dir);
        }
    }

    #[cfg(target_os = "windows")]
    {
        let defaults = [
            PathBuf::from(r"C:\Program Files (x86)\Steam")
                .join("steamapps\\common")
                .join(ARMA3_STEAM_GAME_DIR_NAME),
            PathBuf::from(r"C:\Program Files\Steam")
                .join("steamapps\\common")
                .join(ARMA3_STEAM_GAME_DIR_NAME),
        ];
        for p in defaults {
            if Arma3Install::new(p.clone(), None::<PathBuf>).is_ok() && seen.insert(p.clone()) {
                out.push(p);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            let roots = [
                home.join(".steam/steam"),
                home.join(".steam/root"),
                home.join(".local/share/Steam"),
                home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
                home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
            ];

            for r in roots {
                let p = r
                    .join("steamapps")
                    .join("common")
                    .join(ARMA3_STEAM_GAME_DIR_NAME);
                if Arma3Install::new(p.clone(), None::<PathBuf>).is_ok() && seen.insert(p.clone()) {
                    out.push(p);
                }
            }
        }
    }

    out
}
