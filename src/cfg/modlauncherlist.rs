use crate::error::Result;
use crate::install::Arma3Install;
use crate::mods::{read_mod_meta, ModSet};
use crate::platform::path::arma_path_string;
use std::fmt::Write;

pub(crate) fn build_modlauncherlist_block(install: &Arma3Install, mods: &ModSet) -> Result<String> {
    let mut out = String::new();
    out.push_str("class ModLauncherList\n{\n");

    let mut idx = 1;
    for local in mods.iter() {
        let mod_dir = local.path();
        let meta = read_mod_meta(mod_dir).unwrap_or_default();
        let dir_name = mod_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("mod")
            .to_string();

        let mut name = meta.name_or(&dir_name);
        name = name.replace('"', "_");

        let full_path = arma_path_string(mod_dir, install.is_proton());
        let _ = writeln!(out, "    class Mod{idx}");
        let _ = writeln!(out, "    {{");
        let _ = writeln!(out, "        dir=\"{}\";", escape_cfg(&dir_name));
        let _ = writeln!(out, "        name=\"{}\";", escape_cfg(&name));
        let _ = writeln!(out, "        origin=\"GAME DIR\";");
        let _ = writeln!(out, "        fullPath=\"{}\";", escape_cfg(&full_path));
        let _ = writeln!(out, "    }};");
        idx += 1;
    }

    out.push_str("};");
    Ok(out)
}

fn escape_cfg(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
