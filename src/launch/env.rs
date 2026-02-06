use std::ffi::{OsStr, OsString};

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
