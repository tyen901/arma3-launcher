use std::path::PathBuf;

pub(crate) fn detect_steam_root() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let candidates = [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ];
    candidates
        .into_iter()
        .find(|p| p.join("config/config.vdf").is_file())
}

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
