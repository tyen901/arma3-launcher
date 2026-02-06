use crate::error::{Arma3Error, Result};
use std::fs;
use std::path::Path;

pub(crate) fn validate_local_mod_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(Arma3Error::InvalidModDir {
            path: path.to_path_buf(),
        });
    }
    let addons = path.join("addons");
    if !addons.is_dir() {
        return Err(Arma3Error::InvalidModDir {
            path: path.to_path_buf(),
        });
    }
    if fs::read_dir(&addons)?.next().is_none() {
        return Err(Arma3Error::InvalidModDir {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}
