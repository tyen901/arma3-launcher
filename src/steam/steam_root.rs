use std::path::PathBuf;

pub(crate) fn detect_steam_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("STEAM_ROOT").map(PathBuf::from) {
        if p.join("steamapps").is_dir() {
            return Some(p);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home = std::env::var_os("HOME").map(PathBuf::from)?;
        let candidates = [
            home.join(".steam/steam"),
            home.join(".steam/root"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ];

        candidates
            .into_iter()
            .find(|p| p.join("config/config.vdf").is_file() || p.join("steamapps").is_dir())
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(p) = detect_from_registry(winreg::enums::HKEY_CURRENT_USER) {
            return Some(p);
        }
        if let Some(p) = detect_from_registry(winreg::enums::HKEY_LOCAL_MACHINE) {
            return Some(p);
        }

        let candidates = [
            PathBuf::from(r"C:\Program Files (x86)\Steam"),
            PathBuf::from(r"C:\Program Files\Steam"),
        ];

        candidates
            .into_iter()
            .find(|p| p.join("steamapps").is_dir())
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn detect_from_registry(root: winreg::HKEY) -> Option<PathBuf> {
    use winreg::RegKey;

    let hk = RegKey::predef(root);
    let steam = hk.open_subkey(r"Software\Valve\Steam").ok()?;

    if let Ok(dir) = steam.get_value::<String, _>("SteamPath") {
        let p = PathBuf::from(dir.trim_matches('\"'));
        if p.join("steamapps").is_dir() {
            return Some(p);
        }
    }

    if let Ok(exe) = steam.get_value::<String, _>("SteamExe") {
        let p = PathBuf::from(exe.trim_matches('\"'));
        if let Some(parent) = p.parent() {
            let parent = parent.to_path_buf();
            if parent.join("steamapps").is_dir() {
                return Some(parent);
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
pub(crate) fn steam_runtime_runsh() -> Option<PathBuf> {
    let steam = detect_steam_root()?;
    let p = steam.join("ubuntu12_32/steam-runtime/run.sh");
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}
