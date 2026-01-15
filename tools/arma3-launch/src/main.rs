use std::path::PathBuf;
use std::process::Command;
use clap::Parser;

/// Simple standalone launcher for Arma 3 (sanity-check binary)
#[derive(Parser)]
struct Args {
    /// Optional path to the Arma 3 executable.
    #[arg(short, long)]
    exe: Option<PathBuf>,

    /// Additional arguments to pass to Arma 3
    #[arg(last = true)]
    extra: Vec<String>,
}

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

fn find_arma3_exe(provided: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = provided {
        if p.exists() {
            return Some(p);
        } else {
            eprintln!("Provided path does not exist: {}", p.display());
            return None;
        }
    }

    #[cfg(target_os = "windows")]
    {
        for p in default_windows_paths() {
            if p.exists() {
                return Some(p);
            }
        }

        if let Ok(steam_path) = std::env::var("STEAM") {
            let candidate = PathBuf::from(steam_path).join("steamapps/common/Arma 3/arma3.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if which::which("arma3").is_ok() {
            return which::which("arma3").ok();
        }
    }

    None
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let exe = find_arma3_exe(args.exe).ok_or_else(|| anyhow::anyhow!("Could not locate arma3 executable. Provide --exe path."))?;

    println!("Launching Arma 3: {}", exe.display());

    let mut cmd = Command::new(exe);
    if !args.extra.is_empty() {
        cmd.args(&args.extra);
    }

    let child = cmd.spawn()?;
    println!("Spawned pid: {}", child.id());

    Ok(())
}
