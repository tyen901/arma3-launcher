use crate::install::Arma3Install;
use std::collections::BTreeSet;
use std::path::PathBuf;

const ARMA3_STEAM_GAME_DIR_NAME: &str = "Arma 3";

/// Return Arma 3 install candidates (best-effort), ordered by likelihood.
///
/// This uses:
/// - `ARMA3_DIR` env override (if set and valid)
/// - Steam libraries (`libraryfolders.vdf`) when Steam is detected
/// - OS-specific default Steam locations (if valid)
pub fn detect_install_candidates() -> Vec<Arma3Install> {
    let mut out: Vec<Arma3Install> = Vec::new();
    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();

    if let Some(p) = std::env::var_os("ARMA3_DIR").map(PathBuf::from) {
        if let Ok(install) = Arma3Install::new(p.clone()) {
            if seen.insert(p) {
                out.push(install);
            }
        }
    }

    for lib in crate::steam::library::detect_steam_library_roots() {
        let game_dir = lib
            .join("steamapps")
            .join("common")
            .join(ARMA3_STEAM_GAME_DIR_NAME);
        if let Ok(install) = Arma3Install::new(game_dir.clone()) {
            if seen.insert(game_dir) {
                out.push(install);
            }
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
            if let Ok(install) = Arma3Install::new(p.clone()) {
                if seen.insert(p) {
                    out.push(install);
                }
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
                if let Ok(install) = Arma3Install::new(p.clone()) {
                    if seen.insert(p) {
                        out.push(install);
                    }
                }
            }
        }
    }

    out
}

/// Return the single most likely Arma 3 install (best-effort).
pub fn detect_best_install() -> Option<Arma3Install> {
    detect_install_candidates().into_iter().next()
}
