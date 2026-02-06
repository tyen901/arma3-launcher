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
        let p = PathBuf::from(dir.trim_matches('"'));
        if p.join("steamapps").is_dir() {
            return Some(p);
        }
    }

    if let Ok(exe) = steam.get_value::<String, _>("SteamExe") {
        let p = PathBuf::from(exe.trim_matches('"'));
        if let Some(parent) = p.parent() {
            let parent = parent.to_path_buf();
            if parent.join("steamapps").is_dir() {
                return Some(parent);
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
pub(crate) fn detect_steam_exe() -> PathBuf {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;
    use winreg::HKEY;

    if let Some(p) = std::env::var_os("STEAM_EXE").map(PathBuf::from) {
        if p.is_file() {
            return p;
        }
    }

    if let Some(p) = detect_exe_from_registry(HKEY_CURRENT_USER) {
        return p;
    }
    if let Some(p) = detect_exe_from_registry(HKEY_LOCAL_MACHINE) {
        return p;
    }

    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe"),
        PathBuf::from(r"C:\Program Files\Steam\steam.exe"),
    ];
    if let Some(p) = candidates.into_iter().find(|p| p.is_file()) {
        return p;
    }

    PathBuf::from("steam")
}

#[cfg(target_os = "windows")]
fn detect_exe_from_registry(root: HKEY) -> Option<PathBuf> {
    let hk = RegKey::predef(root);
    let steam = hk.open_subkey(r"Software\Valve\Steam").ok()?;

    if let Ok(exe) = steam.get_value::<String, _>("SteamExe") {
        let p = PathBuf::from(exe.trim_matches('"'));
        if p.is_file() {
            return Some(p);
        }
    }

    if let Ok(dir) = steam.get_value::<String, _>("SteamPath") {
        let p = PathBuf::from(dir.trim_matches('"')).join("steam.exe");
        if p.is_file() {
            return Some(p);
        }
    }

    None
}

#[cfg(target_os = "linux")]
pub(crate) fn is_flatpak_steam() -> bool {
    if std::env::var_os("FLATPAK_ID").is_some() {
        return true;
    }
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return false;
    };
    home.join(".var/app/com.valvesoftware.Steam").is_dir()
}

#[cfg(target_os = "linux")]
pub(crate) fn detect_steam_exe() -> PathBuf {
    if let Some(p) = std::env::var_os("STEAM_EXE").map(PathBuf::from) {
        if p.is_file() {
            return p;
        }
    }

    if let Some(root) = detect_steam_root() {
        let candidates = [
            root.join("steam.sh"),
            root.join("steam"),
            root.join("steam-runtime"),
        ];
        if let Some(p) = candidates.into_iter().find(|p| p.is_file()) {
            return p;
        }
    }

    let candidates = [
        PathBuf::from("/usr/bin/steam"),
        PathBuf::from("/usr/bin/steam-runtime"),
        PathBuf::from("/usr/lib/steam/steam"),
        PathBuf::from("/usr/lib/steam/steam.sh"),
        PathBuf::from("/usr/lib64/steam/steam"),
        PathBuf::from("/usr/lib64/steam/steam.sh"),
    ];
    if let Some(p) = candidates.into_iter().find(|p| p.is_file()) {
        return p;
    }

    PathBuf::from("steam")
}
