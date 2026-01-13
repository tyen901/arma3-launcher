use std::path::PathBuf;

/// Result alias for this crate.
pub type Result<T> = std::result::Result<T, Arma3Error>;

/// Error type for `arma3-launcher`.
#[derive(thiserror::Error, Debug)]
pub enum Arma3Error {
    /// Install directory is missing or invalid.
    #[error("invalid Arma 3 install directory: {path}")]
    InvalidInstallDir {
        /// Path that failed validation.
        path: PathBuf,
    },

    /// Expected executable was not found in the install directory.
    #[error("arma executable not found in install directory: {install_dir}")]
    ExecutableNotFound {
        /// Install directory where the executable was expected.
        install_dir: PathBuf,
    },

    /// Workshop directory is required but missing.
    #[error("workshop directory is required for workshop IDs but was not configured")]
    WorkshopDirMissing,

    /// A mod directory is invalid (e.g., missing `addons`).
    #[error("invalid mod directory: {path} (expected an 'addons' directory inside)")]
    InvalidModDir {
        /// Mod directory path.
        path: PathBuf,
    },

    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// UTF-8 conversion failed (only used for internal parsing needs).
    #[error("utf-8 conversion error")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Steam detection required but failed.
    #[error("steam installation could not be detected (required for Proton direct launch)")]
    SteamNotFound,

    /// Steam/Proton configuration parsing error.
    #[error("steam config error: {message}")]
    SteamConfig {
        /// Human-readable message.
        message: String,
    },

    /// Configuration format/parsing error.
    #[error("config parsing error: {message}")]
    Parse {
        /// Human-readable message.
        message: String,
    },

    /// Spawn failed.
    #[error("failed to spawn process: {message}")]
    Spawn {
        /// Human-readable message.
        message: String,
    },
}
