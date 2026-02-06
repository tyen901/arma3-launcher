use crate::steam::detect::detect_steam_root;
use std::path::PathBuf;

pub(crate) fn linux_overlay_so() -> Option<PathBuf> {
    let steam = detect_steam_root()?;
    let a = steam.join("ubuntu12_64/gameoverlayrenderer.so");
    if a.is_file() {
        return Some(a);
    }
    let b = steam.join("linux64/gameoverlayrenderer.so");
    if b.is_file() {
        return Some(b);
    }
    None
}

pub(crate) fn steam_runtime_runsh() -> Option<PathBuf> {
    let steam = detect_steam_root()?;
    let p = steam.join("ubuntu12_32/steam-runtime/run.sh");
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}
