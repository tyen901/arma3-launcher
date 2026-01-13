# README.md

## arma3-launcher

A small library that:
- Validates an Arma 3 install directory
- Builds a `ModLauncherList` section in `Arma3.cfg` for enabled mods
- Launches Arma 3 with user-provided arguments and environment variables
- Supports Linux and Windows launch styles similar to common Steam setups

## Quick example

```rust
use arma3_launcher::{Arma3Install, Arma3Launcher, LaunchMode, ModSpec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let install = Arma3Install::new(
        "/home/USERNAME/.local/share/Steam/steamapps/common/Arma 3",
        Some("/home/USERNAME/.local/share/Steam/steamapps/workshop/content/107410"),
    )?;

    let launcher = Arma3Launcher::new(install)
        .launch_mode(LaunchMode::ThroughSteam) // default
        .disable_esync(true) // Linux Proton direct only (no-op on Windows / ThroughSteam)
        .arg("-noSplash")
        .arg("-skipIntro")
        .arg("-world=empty")
        .mod_enabled(ModSpec::Local("/home/USERNAME/mods/@ace".into()))
        .mod_enabled(ModSpec::WorkshopId(463939057));

    launcher.write_cfg()?;
    let _child = launcher.launch()?;
    Ok(())
}
```

## Notes

* Workshop IDs require `workshop_dir` on `Arma3Install`.
* You can override cfg path if you need a non-standard location.
* Linux Proton “direct” launching requires Steam detection; if not found, the library returns an error.
