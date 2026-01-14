#![cfg(target_os = "windows")]

use std::path::PathBuf;

use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;
use winreg::HKEY;

/// Best-effort Steam.exe detection on Windows:
/// - STEAM_EXE env var (full path)
/// - Registry (HKCU/HKLM) Software\Valve\Steam SteamExe or SteamPath
/// - Common default install paths
/// - Fallback to "steam" (PATH)
pub(crate) fn detect_steam_exe() -> PathBuf {
    // 1) Explicit override
    if let Some(p) = std::env::var_os("STEAM_EXE").map(PathBuf::from) {
        if p.is_file() {
            return p;
        }
    }

    // 2) Registry (HKCU then HKLM)
    if let Some(p) = detect_from_registry(HKEY_CURRENT_USER) {
        return p;
    }
    if let Some(p) = detect_from_registry(HKEY_LOCAL_MACHINE) {
        return p;
    }

    // 3) Default paths
    let candidates = [
        PathBuf::from(r"C:\Program Files (x86)\Steam\steam.exe"),
        PathBuf::from(r"C:\Program Files\Steam\steam.exe"),
    ];
    if let Some(p) = candidates.into_iter().find(|p| p.is_file()) {
        return p;
    }

    // 4) PATH fallback
    PathBuf::from("steam")
}

fn detect_from_registry(root: HKEY) -> Option<PathBuf> {
    let hk = RegKey::predef(root);
    let steam = hk.open_subkey(r"Software\Valve\Steam").ok()?;

    // SteamExe is often a full path.
    if let Ok(exe) = steam.get_value::<String, _>("SteamExe") {
        let p = PathBuf::from(exe.trim_matches('\"'));
        if p.is_file() {
            return Some(p);
        }
    }

    // SteamPath is often the install directory.
    if let Ok(dir) = steam.get_value::<String, _>("SteamPath") {
        let p = PathBuf::from(dir.trim_matches('\"')).join("steam.exe");
        if p.is_file() {
            return Some(p);
        }
    }

    None
}
