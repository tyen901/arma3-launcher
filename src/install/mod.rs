use crate::error::{Arma3Error, Result};
use std::path::{Path, PathBuf};

mod cfg_path;
mod detect;

pub use detect::{detect_best_install, detect_install_candidates};

/// Platform/runtime kind for this install.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    /// Native Linux install (arma3.x86_64)
    LinuxNative,
    /// Proton-based install on Linux (arma3_x64.exe present)
    LinuxProton,
    /// Native Windows install (arma3_x64.exe or arma3.exe)
    WindowsNative,
}

/// A validated Arma 3 installation.
#[derive(Debug, Clone)]
pub struct Arma3Install {
    game_dir: PathBuf,
    executable: PathBuf,
    kind: InstallKind,
}

impl Arma3Install {
    /// Create and validate an install from a game directory.
    pub fn new(game_dir: impl Into<PathBuf>) -> Result<Self> {
        let game_dir = game_dir.into();
        if !game_dir.is_dir() {
            return Err(Arma3Error::InvalidInstallDir { path: game_dir });
        }

        let (executable, kind) =
            find_executable(&game_dir).ok_or_else(|| Arma3Error::ExecutableNotFound {
                install_dir: game_dir.clone(),
            })?;

        Ok(Self {
            game_dir,
            executable,
            kind,
        })
    }

    /// Game directory (Arma 3 install).
    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    /// Executable path (native binary or `arma3_x64.exe`).
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Install kind (Linux native, Linux Proton, Windows).
    pub fn kind(&self) -> InstallKind {
        self.kind
    }

    /// True only for Linux Proton installs.
    pub fn is_proton(&self) -> bool {
        self.kind == InstallKind::LinuxProton
    }

    /// Default Arma3.cfg path.
    ///
    /// - Linux native: `~/.local/share/bohemiainteractive/arma3/GameDocuments/Arma 3/Arma3.cfg`
    /// - Linux Proton: Steam compatdata prefix `.../steamapps/compatdata/107410/.../My Documents/Arma 3/Arma3.cfg`
    /// - Windows: `Documents/Arma 3/Arma3.cfg`
    pub fn default_cfg_path(&self) -> Result<PathBuf> {
        cfg_path::default_cfg_path(self.kind, &self.game_dir)
    }
}

fn find_executable(game_dir: &Path) -> Option<(PathBuf, InstallKind)> {
    #[cfg(target_os = "linux")]
    {
        let p = game_dir.join("arma3.x86_64");
        if p.is_file() {
            return Some((p, InstallKind::LinuxNative));
        }
        let p = game_dir.join("arma3_x64.exe");
        if p.is_file() {
            return Some((p, InstallKind::LinuxProton));
        }
        None
    }

    #[cfg(target_os = "windows")]
    {
        let p = game_dir.join("arma3_x64.exe");
        if p.is_file() {
            return Some((p, InstallKind::WindowsNative));
        }
        let p = game_dir.join("arma3.exe");
        if p.is_file() {
            return Some((p, InstallKind::WindowsNative));
        }
        None
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = game_dir;
        None
    }
}
