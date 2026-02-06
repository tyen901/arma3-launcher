pub(crate) mod compat;
pub(crate) mod detect;
pub(crate) mod library;
#[cfg(target_os = "linux")]
pub(crate) mod runtime;
pub(crate) mod vdf;

pub(crate) const ARMA3_APP_ID_STR: &str = "107410";
pub(crate) const STEAM_ARG_APPLAUNCH: &str = "-applaunch";
pub(crate) const STEAM_ARG_NO_LAUNCHER: &str = "-nolauncher";

pub(crate) const ENV_STEAM_GAME_ID: &str = "SteamGameId";
pub(crate) const ENV_STEAM_COMPAT_DATA_PATH: &str = "STEAM_COMPAT_DATA_PATH";
pub(crate) const ENV_PROTON_NO_ESYNC: &str = "PROTON_NO_ESYNC";
