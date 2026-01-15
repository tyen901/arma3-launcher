use clap::Parser;
use std::path::{Path, PathBuf};
use std::process::Command;

use arma3_launcher::{detect_arma3_install_path, Arma3Install};

/// Simple standalone launcher for Arma 3 (sanity-check binary)
#[derive(Parser)]
struct Args {
    /// Optional Arma 3 install directory (Steam game dir).
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Optional path to the Arma 3 executable.
    #[arg(short, long)]
    exe: Option<PathBuf>,

    /// Additional arguments to pass to Arma 3
    #[arg(last = true)]
    extra: Vec<String>,
}

#[cfg(target_os = "windows")]
fn default_windows_paths() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles(x86)") {
        v.push(PathBuf::from(program_files).join("Steam/steamapps/common/Arma 3/arma3.exe"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        v.push(PathBuf::from(program_files).join("Steam/steamapps/common/Arma 3/arma3.exe"));
    }
    v
}

fn find_install_dir(provided: Option<PathBuf>) -> Option<PathBuf> {
    provided.or_else(detect_arma3_install_path)
}

fn find_arma3_exe(provided_exe: Option<PathBuf>, install_dir: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = provided_exe {
        if p.is_file() {
            return Some(p);
        }
        eprintln!("Provided --exe path is not a file: {}", p.display());
        return None;
    }

    if let Some(game_dir) = find_install_dir(install_dir) {
        match Arma3Install::new(&game_dir, None::<PathBuf>) {
            Ok(install) => return Some(install.executable().to_path_buf()),
            Err(e) => eprintln!(
                "Failed validating Arma 3 install dir {}: {e}",
                game_dir.display()
            ),
        }
    }

    #[cfg(target_os = "windows")]
    {
        for p in default_windows_paths() {
            if p.is_file() {
                return Some(p);
            }
        }

        if let Ok(steam_path) = std::env::var("STEAM") {
            let candidate = PathBuf::from(steam_path).join("steamapps/common/Arma 3/arma3.exe");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    which::which("arma3").ok().filter(|p| p.is_file())
}

fn infer_game_dir_from_exe(exe: &Path) -> Option<PathBuf> {
    exe.parent().map(|p| p.to_path_buf())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let exe = find_arma3_exe(args.exe.clone(), args.dir.clone()).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not locate Arma 3 executable. Provide --dir <install_dir> or --exe <path> (or set ARMA3_DIR)."
        )
    })?;

    if args.dir.is_none() && args.exe.is_some() {
        if let Some(dir) = infer_game_dir_from_exe(&exe) {
            let _ = Arma3Install::new(dir, None::<PathBuf>);
        }
    }

    println!("Launching Arma 3: {}", exe.display());

    let mut cmd = Command::new(exe);
    if !args.extra.is_empty() {
        cmd.args(&args.extra);
    }

    let child = cmd.spawn()?;
    println!("Spawned pid: {}", child.id());

    Ok(())
}
