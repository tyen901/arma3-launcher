use crate::error::{Arma3Error, Result};
use crate::install::Arma3Install;
use crate::mods::{read_mod_meta, validate_local_mod_dir, ModSpec};
use std::fs;
use std::path::{Path, PathBuf};

/// How cfg should be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfgMode {
    /// Read existing cfg if present, remove existing `class ModLauncherList`, then append a new one.
    MergeOrCreate,
    /// Overwrite cfg entirely with only the new `class ModLauncherList` section.
    Overwrite,
}

/// Helper describing cfg-related paths.
#[derive(Debug, Clone)]
pub struct CfgPaths {
    /// The cfg file path.
    pub cfg_path: PathBuf,
}

/// Write cfg per `mode`.
pub fn write_cfg(
    cfg_path: &Path,
    mode: CfgMode,
    install: &Arma3Install,
    enabled_mods: &[ModSpec],
) -> Result<()> {
    if let Some(parent) = cfg_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let block = build_modlauncherlist_block(install, enabled_mods)?;
    let new_text = match mode {
        CfgMode::Overwrite => format!("{block}\n"),
        CfgMode::MergeOrCreate => {
            let existing = fs::read_to_string(cfg_path).unwrap_or_default();
            let stripped = strip_cpp_class(&existing, "class ModLauncherList")?;
            if stripped.trim().is_empty() {
                format!("{block}\n")
            } else {
                format!("{stripped}\n\n{block}\n")
            }
        }
    };

    fs::write(cfg_path, new_text)?;
    Ok(())
}

/// Build the `class ModLauncherList { ... };` block.
pub fn build_modlauncherlist_block(
    install: &Arma3Install,
    enabled_mods: &[ModSpec],
) -> Result<String> {
    let mut out = String::new();
    out.push_str("class ModLauncherList\n{\n");

    let mut idx = 1;
    for spec in enabled_mods {
        let mod_dir = resolve_mod_path(install, spec)?;
        validate_local_mod_dir(&mod_dir)?;

        let meta = read_mod_meta(&mod_dir).unwrap_or_default();
        let dir_name = mod_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("mod")
            .to_string();

        let mut name = meta.name_or(&dir_name);
        name = name.replace('"', "_");

        let full_path = launcher_full_path(&mod_dir, install);
        out.push_str(&format!(
            "    class Mod{idx}\n    {{\n        dir=\"{dir}\";\n        name=\"{name}\";\n        origin=\"GAME DIR\";\n        fullPath=\"{full}\";\n    }};\n",
            idx = idx,
            dir = escape_cfg(&dir_name),
            name = escape_cfg(&name),
            full = escape_cfg(&full_path),
        ));
        idx += 1;
    }

    out.push_str("};");
    Ok(out)
}

fn resolve_mod_path(install: &Arma3Install, spec: &ModSpec) -> Result<PathBuf> {
    match spec {
        ModSpec::Local(p) => Ok(p.clone()),
        ModSpec::WorkshopId(id) => {
            let w = install
                .workshop_dir()
                .ok_or(Arma3Error::WorkshopDirMissing)?;
            Ok(w.join(id.to_string()))
        }
    }
}

fn escape_cfg(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The `fullPath` field in Arma cfg is typically expected to look like a Windows path.
/// - Windows: use native path string
/// - Linux Proton: map into `Z:\` and convert slashes
/// - Linux native: preserve prior behavior (prefix `C:` for absolute paths)
fn launcher_full_path(path: &Path, install: &Arma3Install) -> String {
    #[cfg(target_os = "windows")]
    {
        return path.to_string_lossy().to_string();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let drive = if install.is_proton() { 'Z' } else { 'C' };
        let mut s = path.to_string_lossy().replace('/', "\\");
        if path.is_absolute() {
            s = format!("{drive}:{s}");
        }
        s
    }
}

/// Strip a C++-style class block, similar to the original project's `CppFilter`.
/// This scans braces while respecting quoted strings.
pub fn strip_cpp_class(text: &str, class_decl: &str) -> Result<String> {
    let mut s = text.to_string();
    let mut positions = Vec::new();
    let mut start = 0usize;

    while let Some(pos) = s[start..].find(class_decl) {
        positions.push(start + pos);
        start = start + pos + class_decl.len();
    }

    // Remove from the end so indices remain valid.
    for &pos in positions.iter().rev() {
        let (a, b) = class_boundaries(&s, class_decl, pos)?;
        s.replace_range(a..b, "");
    }

    Ok(s)
}

fn class_boundaries(text: &str, class_decl: &str, start: usize) -> Result<(usize, usize)> {
    if !text[start..].starts_with(class_decl) {
        return Err(Arma3Error::Parse {
            message: "class boundary mismatch".into(),
        });
    }

    let open = text[start..]
        .find('{')
        .map(|i| start + i)
        .ok_or_else(|| Arma3Error::Parse {
            message: "cannot find opening '{'".into(),
        })?;

    let mut depth = 1i32;
    let mut in_str = false;
    let mut escape = false;
    let mut i = open + 1;

    let bytes = text.as_bytes();
    while i < bytes.len() && depth > 0 {
        let c = bytes[i] as char;

        if escape {
            escape = false;
        } else if in_str && c == '\\' {
            escape = true;
        } else if c == '"' {
            in_str = !in_str;
        } else if !in_str {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
            }
        }

        i += 1;
    }

    if depth != 0 {
        return Err(Arma3Error::Parse {
            message: "unclosed '{' in cfg".into(),
        });
    }

    // Consume until ';' then end of line, or until next token.
    let mut end = i;
    let mut saw_semicolon = false;
    let mut saw_newline = false;

    while end < bytes.len() {
        let c = bytes[end] as char;
        if c == ';' {
            saw_semicolon = true;
        } else if c == '\n' {
            saw_newline = true;
        } else if c.is_ascii_alphanumeric() && saw_semicolon {
            break;
        }
        end += 1;
        if saw_semicolon && saw_newline {
            break;
        }
    }

    Ok((start, end))
}
