use crate::error::Result;
use std::path::{Path, PathBuf};

mod meta;
mod validate;

pub(crate) use meta::read_mod_meta;
use validate::validate_local_mod_dir;

/// A validated local mod directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalMod {
    path: PathBuf,
}

impl LocalMod {
    /// Validate and create a local mod reference.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        validate_local_mod_dir(&path)?;
        Ok(Self { path })
    }

    /// Mod directory path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Ordered collection of local mods.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModSet {
    mods: Vec<LocalMod>,
}

impl ModSet {
    /// Create an empty mod set.
    pub fn new() -> Self {
        Self { mods: Vec::new() }
    }

    /// Add a mod to the set.
    pub fn push(&mut self, m: LocalMod) {
        self.mods.push(m);
    }

    /// Extend the set with more mods.
    pub fn extend<I>(&mut self, mods: I)
    where
        I: IntoIterator<Item = LocalMod>,
    {
        self.mods.extend(mods);
    }

    /// Iterate over mods.
    pub fn iter(&self) -> impl Iterator<Item = &LocalMod> {
        self.mods.iter()
    }

    /// Access underlying mods slice.
    pub fn as_slice(&self) -> &[LocalMod] {
        &self.mods
    }

    /// True if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.mods.is_empty()
    }
}
