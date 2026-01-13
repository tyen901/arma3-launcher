//! Integration tests for command building and config writing.
use arma3_launcher::{Arma3Install, Arma3Launcher, LaunchMode, ModSpec};
use std::fs;
use tempfile::tempdir;

#[test]
fn builds_through_steam_command() {
    // Fake install layout: create platform-specific stub executable
    let d = tempdir().unwrap();
    let game = d.path().join("Arma 3");
    fs::create_dir_all(&game).unwrap();
    let exe = if cfg!(target_os = "windows") {
        "arma3_x64.exe"
    } else {
        "arma3.x86_64"
    };
    fs::write(game.join(exe), b"").unwrap(); // stub file

    let install = Arma3Install::new(&game, None::<&str>).unwrap();
    let launcher = Arma3Launcher::new(install)
        .launch_mode(LaunchMode::ThroughSteam)
        .arg("-noSplash");

    let spec = launcher.build_command().unwrap();
    assert!(
        spec.program.to_string_lossy().contains("steam")
            || spec.program.to_string_lossy().contains("flatpak")
    );
    assert!(spec.args.iter().any(|a| a == "-applaunch"));
    assert!(spec.args.iter().any(|a| a == "107410"));
    assert!(spec.args.iter().any(|a| a == "-noSplash"));
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
    fs::write(mod_dir.join("mod.cpp"), r#"name="My Mod";"#).unwrap();

    let install = Arma3Install::new(&game, None::<&str>).unwrap();
    let cfg_path = d.path().join("Arma3.cfg");

    let launcher = Arma3Launcher::new(install)
        .cfg_path_override(&cfg_path)
        .mod_enabled(ModSpec::Local(mod_dir));

    launcher.write_cfg().unwrap();
    let text = fs::read_to_string(cfg_path).unwrap();
    assert!(text.contains("class ModLauncherList"));
    assert!(text.contains("My Mod"));
}
