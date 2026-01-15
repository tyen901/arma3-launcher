use clap::Parser;
use std::path::{Path, PathBuf};

use arma3_launcher::{detect_arma3_install_path, Arma3Install, Arma3Launcher, LaunchMode};

/// Simple standalone launcher for Arma 3 (sanity-check binary)
#[derive(Parser)]
struct Args {
    /// Optional Arma 3 install directory (Steam game dir).
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Optional path to the Arma 3 executable.
    #[arg(short, long)]
    exe: Option<PathBuf>,

    /// Launch directly (bypasses Steam). On Linux Proton installs this may fail.
    #[arg(long)]
    direct: bool,

    /// Additional arguments to pass to Arma 3
    #[arg(last = true)]
    extra: Vec<String>,
}

fn find_install_dir(provided: Option<PathBuf>) -> Option<PathBuf> {
    provided.or_else(detect_arma3_install_path)
}

fn find_install(provided_exe: Option<PathBuf>, install_dir: Option<PathBuf>) -> Option<Arma3Install> {
    if let Some(p) = provided_exe {
        if !p.is_file() {
            eprintln!("Provided --exe path is not a file: {}", p.display());
            return None;
        }
        let dir = infer_game_dir_from_exe(&p)?;
        return Arma3Install::new(dir, None::<PathBuf>).ok();
    }

    let game_dir = find_install_dir(install_dir)?;
    Arma3Install::new(game_dir, None::<PathBuf>).ok()
}

fn infer_game_dir_from_exe(exe: &Path) -> Option<PathBuf> {
    exe.parent().map(|p| p.to_path_buf())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let install = find_install(args.exe.clone(), args.dir.clone()).ok_or_else(|| {
        anyhow::anyhow!(
            "Could not locate Arma 3 install. Provide --dir <install_dir> or --exe <path> (or set ARMA3_DIR)."
        )
    })?;

    let mode = if args.direct {
        LaunchMode::Direct
    } else {
        #[cfg(target_os = "linux")]
        {
            LaunchMode::ThroughSteam
        }
        #[cfg(not(target_os = "linux"))]
        {
            LaunchMode::Direct
        }
    };

    let launcher = Arma3Launcher::new(install)
        .launch_mode(mode)
        .args(args.extra);

    let spec = launcher.build_command()?;
    println!(
        "Launching Arma 3 via {}: {}",
        match mode {
            LaunchMode::ThroughSteam => "steam",
            LaunchMode::Direct => "direct",
        },
        spec.program.display()
    );
    let child = spec.spawn()?;
    println!("Spawned pid: {}", child.id());

    Ok(())
}
