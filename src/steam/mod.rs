#[cfg(target_os = "linux")]
pub(crate) mod proton_direct;

pub(crate) mod vdf;

#[cfg(target_os = "linux")]
pub(crate) mod steam_root;

#[cfg(target_os = "windows")]
pub(crate) mod steam_exe;
