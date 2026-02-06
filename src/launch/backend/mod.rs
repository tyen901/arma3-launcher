use crate::error::Result;
use crate::install::Arma3Install;
use crate::launch::env::EnvVars;
use crate::launch::plan::CommandSpec;
use std::ffi::OsString;
use std::path::Path;

pub(crate) mod direct;
pub(crate) mod proton;
pub(crate) mod steam;

pub(crate) struct BackendParams<'a> {
    pub(crate) install: &'a Arma3Install,
    pub(crate) user_args: &'a [OsString],
    pub(crate) user_env: &'a EnvVars,
    pub(crate) working_dir: Option<&'a Path>,
    pub(crate) disable_esync: bool,
}

pub(crate) trait Backend {
    fn plan(&self, params: &BackendParams<'_>) -> Result<CommandSpec>;
}

pub(crate) fn collect_env(user_env: &EnvVars) -> Vec<(OsString, OsString)> {
    let mut env = Vec::new();
    for (k, v) in user_env.iter() {
        env.push((k.clone(), v.clone()));
    }
    env
}
