use crate::error::Result;
use crate::install::Arma3Install;
use crate::mods::ModSet;
use std::fs;
use std::path::Path;

mod modlauncherlist;
mod strip;

use modlauncherlist::build_modlauncherlist_block;
use strip::strip_cpp_class;

/// How cfg should be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgMode {
    /// Read existing cfg if present, remove existing `class ModLauncherList`, then append a new one.
    MergeOrCreate,
    /// Overwrite cfg entirely with only the new `class ModLauncherList` section.
    Overwrite,
}

pub(crate) fn write_cfg(
    cfg_path: &Path,
    mode: CfgMode,
    install: &Arma3Install,
    mods: &ModSet,
) -> Result<()> {
    if let Some(parent) = cfg_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let block = build_modlauncherlist_block(install, mods)?;
    let new_text = match mode {
        CfgMode::Overwrite => format!("{block}\n"),
        CfgMode::MergeOrCreate => {
            let existing = fs::read_to_string(cfg_path).unwrap_or_default();
            let stripped = strip_cpp_class(&existing, "class ModLauncherList")?;
            if stripped.trim().is_empty() {
                format!("{block}\n")
            } else {
                format!("{stripped}\n\n{block}\n")
            }
        }
    };

    fs::write(cfg_path, new_text)?;
    Ok(())
}
