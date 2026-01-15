/// Steam App ID for Arma 3.
#[allow(dead_code)]
pub(crate) const ARMA3_APP_ID: u32 = 107410;
pub(crate) const ARMA3_APP_ID_STR: &str = "107410";

/// Steam common install folder name for Arma 3.
pub(crate) const ARMA3_STEAM_GAME_DIR_NAME: &str = "Arma 3";

/// Common Steam launch flags used by this crate.
pub(crate) const STEAM_ARG_APPLAUNCH: &str = "-applaunch";
pub(crate) const STEAM_ARG_NO_LAUNCHER: &str = "-nolauncher";

/// Env vars used for Proton direct launching.
pub(crate) const ENV_STEAM_GAME_ID: &str = "SteamGameId";
pub(crate) const ENV_STEAM_COMPAT_DATA_PATH: &str = "STEAM_COMPAT_DATA_PATH";
pub(crate) const ENV_PROTON_NO_ESYNC: &str = "PROTON_NO_ESYNC";

/// Proton prefix relative path to Arma 3 cfg (under compatdata/<appid>/).
pub(crate) const PROTON_ARMA3_CFG_REL: &str =
    "pfx/drive_c/users/steamuser/My Documents/Arma 3/Arma3.cfg";
