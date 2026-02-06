//! Integration tests for planning and config writing.
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
    let cfg_path = d.path().join("Arma3.cfg");

    let launcher = Launcher::new(install)
        .launch_mode(LaunchMode::ThroughSteam)
        .cfg_path_override(&cfg_path)
        .arg("-noSplash");

    let plan = launcher.plan().unwrap();
    let program = plan.program().to_string_lossy();
    assert!(program.contains("steam") || program.contains("flatpak"));
    assert!(plan.args().iter().any(|a| a == "-applaunch"));
    assert!(plan.args().iter().any(|a| a == "107410"));
    assert!(plan.args().iter().any(|a| a == "-noSplash"));
}

#[test]
fn writes_cfg_for_enabled_mods() {
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
    let cfg_path = d.path().join("Arma3.cfg");

    let mut mods = ModSet::new();
    mods.push(LocalMod::new(mod_dir).unwrap());

    let launcher = Launcher::new(install)
        .cfg_path_override(&cfg_path)
        .mods(mods);

    let _plan = launcher.plan().unwrap();
    let text = fs::read_to_string(cfg_path).unwrap();
    assert!(text.contains("class ModLauncherList"));
    assert!(text.contains("My Mod"));
}
