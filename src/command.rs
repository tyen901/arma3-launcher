use crate::error::{Arma3Error, Result};
use crate::install::Arma3Install;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Environment variables for the spawned process.
#[derive(Debug, Clone, Default)]
pub struct EnvVars {
    inner: std::collections::BTreeMap<OsString, OsString>,
}

impl EnvVars {
    /// Insert a `key=value` environment variable.
    pub fn insert(&mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) {
        self.inner
            .insert(key.as_ref().to_os_string(), val.as_ref().to_os_string());
    }

    /// Iterate over environment variables in a stable order.
    pub fn iter(&self) -> impl Iterator<Item = (&OsString, &OsString)> {
        self.inner.iter()
    }
}

/// A spawn-ready command description (testable without executing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    /// Executable to run.
    pub program: PathBuf,
    /// Arguments passed to the executable.
    pub args: Vec<OsString>,
    /// Working directory for the spawned process.
    pub cwd: Option<PathBuf>,
    /// Environment variables set for the spawned process.
    pub env: Vec<(OsString, OsString)>,
}

impl CommandSpec {
    /// Spawn the described process.
    pub fn spawn(&self) -> Result<std::process::Child> {
        let mut cmd = std::process::Command::new(&self.program);
        cmd.args(&self.args);
        if let Some(cwd) = &self.cwd {
            cmd.current_dir(cwd);
        }
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        cmd.spawn().map_err(|e| Arma3Error::Spawn {
            message: format!("{e}"),
        })
    }
}

pub(crate) fn build_command(
    install: &Arma3Install,
    mode: crate::LaunchMode,
    disable_esync: bool,
    user_args: &[OsString],
    user_env: &EnvVars,
    working_dir: Option<&Path>,
) -> Result<CommandSpec> {
    #[cfg(target_os = "linux")]
    {
        build_linux_command(
            install,
            mode,
            disable_esync,
            user_args,
            user_env,
            working_dir,
        )
    }

    #[cfg(target_os = "windows")]
    {
        let _ = disable_esync; // no-op on Windows
        build_windows_command(install, mode, user_args, user_env, working_dir)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        let _ = (
            install,
            mode,
            disable_esync,
            user_args,
            user_env,
            working_dir,
        );
        Err(Arma3Error::Parse {
            message: "unsupported platform".into(),
        })
    }
}

#[cfg(target_os = "windows")]
fn build_windows_command(
    install: &Arma3Install,
    mode: crate::LaunchMode,
    user_args: &[OsString],
    user_env: &EnvVars,
    working_dir: Option<&Path>,
) -> Result<CommandSpec> {
    let mut env = vec![];
    for (k, v) in user_env.iter() {
        env.push((k.clone(), v.clone()));
    }

    match mode {
        crate::LaunchMode::ThroughSteam => {
            let program = crate::steam::steam_exe::detect_steam_exe();

            let mut args: Vec<OsString> = vec![];
            args.push(OsString::from("-applaunch"));
            args.push(OsString::from("107410"));
            args.push(OsString::from("-nolauncher"));
            args.extend_from_slice(user_args);

            Ok(CommandSpec {
                program,
                args,
                cwd: working_dir
                    .map(|p| p.to_path_buf())
                    .or_else(|| Some(install.game_dir().to_path_buf())),
                env,
            })
        }
        crate::LaunchMode::Direct => Ok(CommandSpec {
            program: install.executable().to_path_buf(),
            args: user_args.to_vec(),
            cwd: working_dir
                .map(|p| p.to_path_buf())
                .or_else(|| Some(install.game_dir().to_path_buf())),
            env,
        }),
    }
}

#[cfg(target_os = "linux")]
fn build_linux_command(
    install: &Arma3Install,
    mode: crate::LaunchMode,
    disable_esync: bool,
    user_args: &[OsString],
    user_env: &EnvVars,
    working_dir: Option<&Path>,
) -> Result<CommandSpec> {
    let mut env = vec![];
    for (k, v) in user_env.iter() {
        env.push((k.clone(), v.clone()));
    }

    match mode {
        crate::LaunchMode::ThroughSteam => {
            // Match a "launcher library" use-case: always suppress the BI launcher.
            let (program, mut args) = if is_flatpak_steam() {
                (
                    PathBuf::from("flatpak"),
                    vec![
                        OsString::from("run"),
                        OsString::from("com.valvesoftware.Steam"),
                    ],
                )
            } else {
                (PathBuf::from("steam"), vec![])
            };

            args.push(OsString::from("-applaunch"));
            args.push(OsString::from("107410"));
            args.push(OsString::from("-nolauncher"));

            args.extend_from_slice(user_args);

            Ok(CommandSpec {
                program,
                args,
                cwd: working_dir
                    .map(|p| p.to_path_buf())
                    .or_else(|| Some(install.game_dir().to_path_buf())),
                env,
            })
        }
        crate::LaunchMode::Direct => {
            if install.is_proton() {
                #[cfg(target_os = "linux")]
                {
                    return crate::steam::proton_direct::build_proton_direct_spec(
                        install,
                        user_args,
                        env,
                        working_dir,
                        disable_esync,
                    );
                }
                #[cfg(not(target_os = "linux"))]
                {
                    return Err(crate::Arma3Error::Parse {
                        message: "proton direct launch is only supported on Linux".to_string(),
                    });
                }
            }

            Ok(CommandSpec {
                program: install.executable().to_path_buf(),
                args: user_args.to_vec(),
                cwd: working_dir
                    .map(|p| p.to_path_buf())
                    .or_else(|| Some(install.game_dir().to_path_buf())),
                env,
            })
        }
    }
}

#[cfg(target_os = "linux")]
fn is_flatpak_steam() -> bool {
    // Stronger than just checking ~/.var/app: also respects FLATPAK_ID.
    if std::env::var_os("FLATPAK_ID").is_some() {
        return true;
    }
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return false;
    };
    home.join(".var/app/com.valvesoftware.Steam").is_dir()
}
