use crate::cfg::{write_cfg, CfgMode};
use crate::error::Result;
use crate::install::Arma3Install;
use crate::launch::backend::{Backend, BackendParams};
use crate::launch::plan::CommandSpec;
use crate::mods::{LocalMod, ModSet};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

mod backend;
mod env;
mod plan;

pub use env::EnvVars;

/// How the game should be launched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchMode {
    /// Launch using Steam (Linux: `steam -applaunch ...` or Flatpak Steam; Windows: `steam.exe -applaunch ...`).
    #[default]
    ThroughSteam,
    /// Launch the executable directly.
    Direct,
}

/// Plan for launching (cfg path + spawnable command).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    command: CommandSpec,
    cfg_path: PathBuf,
}

impl LaunchPlan {
    /// Path to the cfg written for this plan.
    pub fn cfg_path(&self) -> &Path {
        &self.cfg_path
    }

    /// Executable to run.
    pub fn program(&self) -> &Path {
        &self.command.program
    }

    /// Arguments passed to the executable.
    pub fn args(&self) -> &[OsString] {
        &self.command.args
    }

    /// Working directory for the spawned process.
    pub fn cwd(&self) -> Option<&Path> {
        self.command.cwd.as_deref()
    }

    /// Environment variables set for the spawned process.
    pub fn env(&self) -> &[(OsString, OsString)] {
        &self.command.env
    }

    /// Spawn the described process.
    pub fn spawn(&self) -> Result<std::process::Child> {
        self.command.spawn()
    }
}

/// Main entry point: configure mods/args/env, write cfg, and launch.
#[derive(Debug, Clone)]
pub struct Launcher {
    install: Arma3Install,
    launch_mode: LaunchMode,
    disable_esync: bool,
    cfg_mode: CfgMode,
    cfg_override: Option<PathBuf>,
    mods: ModSet,
    args: Vec<OsString>,
    env: EnvVars,
    working_dir: Option<PathBuf>,
}

impl Launcher {
    /// Create a launcher for a validated install.
    pub fn new(install: Arma3Install) -> Self {
        Self {
            install,
            launch_mode: LaunchMode::default(),
            disable_esync: false,
            cfg_mode: CfgMode::MergeOrCreate,
            cfg_override: None,
            mods: ModSet::new(),
            args: Vec::new(),
            env: EnvVars::default(),
            working_dir: None,
        }
    }

    /// Set launch mode.
    pub fn launch_mode(mut self, mode: LaunchMode) -> Self {
        self.launch_mode = mode;
        self
    }

    /// If true (Linux/Proton direct only), set `PROTON_NO_ESYNC=1` similarly to common launchers.
    /// On Windows and on ThroughSteam, this is effectively a no-op.
    pub fn disable_esync(mut self, value: bool) -> Self {
        self.disable_esync = value;
        self
    }

    /// Control how `Arma3.cfg` is updated.
    pub fn cfg_mode(mut self, mode: CfgMode) -> Self {
        self.cfg_mode = mode;
        self
    }

    /// Override the cfg path used for writing `ModLauncherList`.
    pub fn cfg_path_override(mut self, path: impl Into<PathBuf>) -> Self {
        self.cfg_override = Some(path.into());
        self
    }

    /// Set working directory for the spawned process. If unset, defaults to game directory.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Add an enabled mod (will be included in cfg).
    pub fn mod_enabled(mut self, m: LocalMod) -> Self {
        self.mods.push(m);
        self
    }

    /// Extend enabled mods.
    pub fn mods_enabled<I>(mut self, mods: I) -> Self
    where
        I: IntoIterator<Item = LocalMod>,
    {
        self.mods.extend(mods);
        self
    }

    /// Replace the entire mod set.
    pub fn mods(mut self, mods: ModSet) -> Self {
        self.mods = mods;
        self
    }

    /// Add a single argument.
    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    /// Extend arguments.
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|s| s.as_ref().to_os_string()));
        self
    }

    /// Add an environment variable for the spawned process.
    pub fn env(mut self, key: impl AsRef<OsStr>, val: impl AsRef<OsStr>) -> Self {
        self.env.insert(key, val);
        self
    }

    /// Extend environment variables.
    pub fn envs<I, K, V>(mut self, vars: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (k, v) in vars {
            self.env.insert(k, v);
        }
        self
    }

    /// Access the underlying install.
    pub fn install(&self) -> &Arma3Install {
        &self.install
    }

    /// Compute the cfg path used for writing.
    pub fn cfg_path(&self) -> Result<PathBuf> {
        if let Some(p) = &self.cfg_override {
            return Ok(p.clone());
        }
        self.install.default_cfg_path()
    }

    /// Build the plan that would be executed (writes cfg, does not spawn).
    pub fn plan(&self) -> Result<LaunchPlan> {
        let cfg_path = self.write_cfg()?;

        let params = BackendParams {
            install: &self.install,
            user_args: &self.args,
            user_env: &self.env,
            working_dir: self.working_dir.as_deref(),
            disable_esync: self.disable_esync,
        };

        let command = match self.launch_mode {
            LaunchMode::ThroughSteam => backend::steam::SteamBackend.plan(&params)?,
            LaunchMode::Direct => {
                if self.install.is_proton() {
                    backend::proton::ProtonBackend.plan(&params)?
                } else {
                    backend::direct::DirectBackend.plan(&params)?
                }
            }
        };

        Ok(LaunchPlan { command, cfg_path })
    }

    /// Spawn the game process and return the `Child`.
    pub fn launch(&self) -> Result<std::process::Child> {
        self.plan()?.spawn()
    }

    fn write_cfg(&self) -> Result<PathBuf> {
        let cfg_path = self.cfg_path()?;
        write_cfg(&cfg_path, self.cfg_mode, &self.install, &self.mods)?;
        Ok(cfg_path)
    }
}
