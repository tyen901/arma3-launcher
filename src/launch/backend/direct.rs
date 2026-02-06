use crate::error::Result;
use crate::launch::backend::{collect_env, Backend, BackendParams};
use crate::launch::plan::CommandSpec;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct DirectBackend;

impl Backend for DirectBackend {
    fn plan(&self, params: &BackendParams<'_>) -> Result<CommandSpec> {
        let env = collect_env(params.user_env);

        Ok(CommandSpec {
            program: params.install.executable().to_path_buf(),
            args: params.user_args.to_vec(),
            cwd: params
                .working_dir
                .map(|p| p.to_path_buf())
                .or_else(|| Some(params.install.game_dir().to_path_buf())),
            env,
        })
    }
}
