//! Integration tests for planning.
use arma3_launcher::{Arma3Install, LaunchMode, Launcher, LocalMod, ModSet};
use std::fs;
use tempfile::tempdir;

#[test]
fn builds_through_steam_plan() {
    let d = tempdir().unwrap();
    let game = d.path().join("Arma 3");
    fs::create_dir_all(&game).unwrap();
    let exe = if cfg!(target_os = "windows") {
        "arma3_x64.exe"
    } else {
        "arma3.x86_64"
    };
    fs::write(game.join(exe), b"").unwrap();

    let install = Arma3Install::new(&game).unwrap();
    let launcher = Launcher::new(install)
        .launch_mode(LaunchMode::ThroughSteam)
        .arg("-noSplash");

    let plan = launcher.plan().unwrap();
    let program = plan.program().to_string_lossy();
    assert!(program.contains("steam") || program.contains("flatpak"));
    assert!(plan.args().iter().any(|a| a == "-applaunch"));
    assert!(plan.args().iter().any(|a| a == "107410"));
    assert!(plan.args().iter().any(|a| a == "-noSplash"));
}

#[test]
fn adds_mod_arg_when_missing() {
    let d = tempdir().unwrap();
    let game = d.path().join("Arma 3");
    fs::create_dir_all(&game).unwrap();
    let exe = if cfg!(target_os = "windows") {
        "arma3_x64.exe"
    } else {
        "arma3.x86_64"
    };
    fs::write(game.join(exe), b"").unwrap();

    let mod_dir = d.path().join("@mymod");
    fs::create_dir_all(mod_dir.join("addons")).unwrap();
    fs::write(mod_dir.join("addons").join("stub.pbo"), "data").unwrap();
    fs::write(mod_dir.join("mod.cpp"), r#"name="My Mod";"#).unwrap();

    let install = Arma3Install::new(&game).unwrap();

    let mut mods = ModSet::new();
    mods.push(LocalMod::new(mod_dir).unwrap());

    let launcher = Launcher::new(install).mods(mods);

    let plan = launcher.plan().unwrap();
    let mod_arg = plan
        .args()
        .iter()
        .map(|a| a.to_string_lossy())
        .find(|a| a.starts_with("-mod="));
    assert!(mod_arg.is_some(), "expected -mod= argument to be added");
}
