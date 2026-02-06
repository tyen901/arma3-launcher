use crate::error::{Arma3Error, Result};
use std::ffi::OsString;
use std::path::PathBuf;

/// A spawn-ready command description (testable without executing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandSpec {
    pub(crate) program: PathBuf,
    pub(crate) args: Vec<OsString>,
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) env: Vec<(OsString, OsString)>,
}

impl CommandSpec {
    pub(crate) fn spawn(&self) -> Result<std::process::Child> {
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
