#![doc = r#"
`arma3-launcher` is a small library for launching Arma 3 with local mods and custom arguments.

Core capabilities:
- Validate an Arma 3 installation directory
- Launch via Steam (indirect) or direct execution

Supported platforms:
- Linux
- Windows
"#]

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
compile_error!("arma3-launcher currently supports Linux and Windows only.");

mod error;
mod install;
mod launch;
mod mods;
mod platform;
mod steam;

pub use crate::error::{Arma3Error, Result};
pub use crate::install::{detect_best_install, detect_install_candidates};
pub use crate::install::{Arma3Install, InstallKind};
pub use crate::launch::{LaunchMode, LaunchPlan, Launcher};
pub use crate::mods::{LocalMod, ModSet};
