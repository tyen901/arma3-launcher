## arma3-launcher

A small library that:
- Validates an Arma 3 install directory
- Builds a `ModLauncherList` section in `Arma3.cfg` for enabled local mods
- Launches Arma 3 with user-provided arguments and environment variables
- Supports Linux and Windows launch styles similar to common Steam setups

## Quick example

```rust
use arma3_launcher::{Arma3Install, Launcher, LaunchMode, LocalMod, ModSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let install = Arma3Install::new(
        "/home/USERNAME/.local/share/Steam/steamapps/common/Arma 3",
    )?;

    let mut mods = ModSet::new();
    mods.push(LocalMod::new("/home/USERNAME/mods/@ace")?);

    let launcher = Launcher::new(install)
        .launch_mode(LaunchMode::ThroughSteam) // default
        .disable_esync(true) // Linux Proton direct only (no-op on Windows / ThroughSteam)
        .arg("-noSplash")
        .arg("-skipIntro")
        .arg("-world=empty")
        .mods(mods);

    let plan = launcher.plan()?; // writes cfg
    let _child = plan.spawn()?;
    Ok(())
}
```

## Notes

* You can override cfg path if you need a non-standard location.
* Linux Proton “direct” launching requires Steam detection; if not found, the library returns an error.

## Tools

This repo includes a small standalone binary under `tools/arma3-launch` for quick sanity checks.

Run it with:

```bash
cargo run --manifest-path tools/arma3-launch/Cargo.toml -- --help
```

## Inspiration

This library was inspired by https://github.com/muttleyxd/arma3-unix-launcher which provided a very useful reference for launching Arma 3 on Linux systems.
