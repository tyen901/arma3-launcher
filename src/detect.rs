use std::path::PathBuf;

/// Return Arma 3 install dir candidates (best-effort), ordered by likelihood.
///
/// This uses:
/// - `ARMA3_DIR` env override (if set and valid)
/// - Steam libraries (`libraryfolders.vdf`) when Steam is detected
/// - OS-specific default Steam locations (if valid)
pub fn detect_arma3_install_candidates() -> Vec<PathBuf> {
    crate::steam::paths::detect_arma3_install_candidates()
}

/// Return the single most likely Arma 3 install dir (best-effort).
pub fn detect_arma3_install_path() -> Option<PathBuf> {
    detect_arma3_install_candidates().into_iter().next()
}
