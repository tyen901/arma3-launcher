use crate::error::{Arma3Error, Result};
use std::path::{Path, PathBuf};

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

/// A validated Arma 3 installation plus optional workshop directory.
#[derive(Debug, Clone)]
pub struct Arma3Install {
    game_dir: PathBuf,
    workshop_dir: Option<PathBuf>,
    executable: PathBuf,
    kind: InstallKind,
}

impl Arma3Install {
    /// Create and validate an install from a game directory and an optional workshop directory.
    pub fn new(
        game_dir: impl Into<PathBuf>,
        workshop_dir: Option<impl Into<PathBuf>>,
    ) -> Result<Self> {
        let game_dir = game_dir.into();
        if !game_dir.is_dir() {
            return Err(Arma3Error::InvalidInstallDir { path: game_dir });
        }

        let workshop_dir = workshop_dir.map(|p| p.into());

        let (executable, kind) =
            find_executable(&game_dir).ok_or_else(|| Arma3Error::ExecutableNotFound {
                install_dir: game_dir.clone(),
            })?;

        Ok(Self {
            game_dir,
            workshop_dir,
            executable,
            kind,
        })
    }

    /// Game directory (Arma 3 install).
    pub fn game_dir(&self) -> &Path {
        &self.game_dir
    }

    /// Optional workshop directory (`.../workshop/content/107410`).
    pub fn workshop_dir(&self) -> Option<&Path> {
        self.workshop_dir.as_deref()
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
    ///
    /// You can override this via `Arma3Launcher::cfg_path_override`.
    pub fn default_cfg_path(&self) -> Result<PathBuf> {
        match self.kind {
            InstallKind::LinuxProton => Ok(self.game_dir.join(
                "../../compatdata/107410/pfx/drive_c/users/steamuser/My Documents/Arma 3/Arma3.cfg",
            )),
            InstallKind::LinuxNative => {
                let home = home_dir()?;
                Ok(home
                    .join(".local/share/bohemiainteractive/arma3/GameDocuments/Arma 3/Arma3.cfg"))
            }
            InstallKind::WindowsNative => {
                let docs = dirs_next::document_dir().ok_or_else(|| Arma3Error::Parse {
                    message: "cannot locate Documents directory".to_string(),
                })?;
                Ok(docs.join("Arma 3").join("Arma3.cfg"))
            }
        }
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
        return None;
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = game_dir;
        None
    }
}

#[cfg(target_os = "linux")]
fn home_dir() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| Arma3Error::Parse {
            message: "HOME environment variable is not set".to_string(),
        })
}

#[cfg(not(target_os = "linux"))]
fn home_dir() -> Result<PathBuf> {
    Err(Arma3Error::Parse {
        message: "home_dir() is not supported on this platform".to_string(),
    })
}
