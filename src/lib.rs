#![doc = r#"
`arma3-launcher` is a small library for launching Arma 3 with mods and custom arguments.

Core capabilities:
- Validate an Arma 3 installation directory
- Generate a `ModLauncherList` section in `Arma3.cfg` based on enabled mods
- Launch via Steam (indirect) or direct execution

Supported platforms:
- Linux
- Windows
"#]

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("arma3-launcher currently supports Linux and Windows only.");

mod cfg;
mod command;
mod detect;
mod error;
mod install;
mod mods;
mod steam;

pub use crate::cfg::{CfgMode, CfgPaths};
pub use crate::command::{CommandSpec, EnvVars};
pub use crate::error::{Arma3Error, Result};
pub use crate::install::{Arma3Install, InstallKind};
pub use crate::mods::{ModMeta, ModSpec};

pub use crate::cfg::strip_cpp_class;
pub use crate::detect::{detect_arma3_install_candidates, detect_arma3_install_path};
pub use crate::mods::{read_mod_meta, validate_local_mod_dir};
pub use crate::steam::paths::arma_path_arg;

pub use crate::steam::vdf::Vdf;

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// How the game should be launched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchMode {
    /// Launch using Steam (Linux: `steam -applaunch ...` or Flatpak Steam; Windows: `steam.exe -applaunch ...`).
    #[default]
    ThroughSteam,
    /// Launch the executable directly:
    /// - Linux: native (`arma3.x86_64`) or Proton direct (`arma3_x64.exe` via compatibility tool)
    /// - Windows: `arma3_x64.exe` (or `arma3.exe` fallback)
    Direct,
}

/// Main entry point: configure mods/args/env, write cfg, and launch.
#[derive(Debug, Clone)]
pub struct Arma3Launcher {
    install: Arma3Install,
    launch_mode: LaunchMode,
    disable_esync: bool,
    cfg_mode: CfgMode,
    cfg_override: Option<PathBuf>,
    enabled_mods: Vec<ModSpec>,
    args: Vec<OsString>,
    env: EnvVars,
    working_dir: Option<PathBuf>,
}

impl Arma3Launcher {
    /// Create a launcher for a validated install.
    pub fn new(install: Arma3Install) -> Self {
        Self {
            install,
            launch_mode: LaunchMode::default(),
            disable_esync: false,
            cfg_mode: CfgMode::MergeOrCreate,
            cfg_override: None,
            enabled_mods: Vec::new(),
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
    pub fn mod_enabled(mut self, m: ModSpec) -> Self {
        self.enabled_mods.push(m);
        self
    }

    /// Extend enabled mods.
    pub fn mods_enabled<I>(mut self, mods: I) -> Self
    where
        I: IntoIterator<Item = ModSpec>,
    {
        self.enabled_mods.extend(mods);
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

    /// Generate the `ModLauncherList` block (text only).
    pub fn build_modlauncherlist_block(&self) -> Result<String> {
        cfg::build_modlauncherlist_block(&self.install, &self.enabled_mods)
    }

    /// Write cfg based on `cfg_mode` and enabled mods.
    pub fn write_cfg(&self) -> Result<PathBuf> {
        let cfg_path = self.cfg_path()?;
        cfg::write_cfg(&cfg_path, self.cfg_mode, &self.install, &self.enabled_mods)?;
        Ok(cfg_path)
    }

    /// Build the command spec that would be executed (does not spawn).
    pub fn build_command(&self) -> Result<CommandSpec> {
        command::build_command(
            &self.install,
            self.launch_mode,
            self.disable_esync,
            &self.args,
            &self.env,
            self.working_dir.as_deref(),
        )
    }

    /// Spawn the game process and return the `Child`.
    pub fn launch(&self) -> Result<std::process::Child> {
        self.build_command()?.spawn()
    }
}

/// Convenience: write cfg and launch using local mod directories + args.
///
/// - `mods` are local directories (each must contain `addons/`)
/// - `extra_args` are passed to the game/steam launch
///
/// This is a small wrapper around [`Arma3Launcher`]:
/// it writes the cfg (using [`CfgMode::MergeOrCreate`]) and then spawns the game.
pub fn launch_with_local_mods(
    game_dir: impl Into<PathBuf>,
    workshop_dir: Option<impl Into<PathBuf>>,
    mods: impl IntoIterator<Item = PathBuf>,
    extra_args: impl IntoIterator<Item = OsString>,
) -> Result<std::process::Child> {
    let install = Arma3Install::new(game_dir, workshop_dir)?;
    let launcher = Arma3Launcher::new(install)
        .mods_enabled(mods.into_iter().map(ModSpec::Local))
        .args(extra_args)
        .cfg_mode(CfgMode::MergeOrCreate);

    launcher.write_cfg()?;
    launcher.launch()
}
