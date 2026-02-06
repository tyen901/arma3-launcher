use std::path::Path;

pub(crate) fn arma_path_string(path: &Path, is_proton: bool) -> String {
    #[cfg(target_os = "windows")]
    {
        let _ = is_proton;
        return path.to_string_lossy().to_string();
    }

    #[cfg(not(target_os = "windows"))]
    {
        if is_proton {
            let mut s = path.to_string_lossy().replace('/', "\\");
            if path.is_absolute() {
                s = format!("Z:{s}");
            }
            return s;
        }

        path.to_string_lossy().to_string()
    }
}
