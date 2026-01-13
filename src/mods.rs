use crate::error::{Arma3Error, Result};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// A mod reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModSpec {
    /// Local mod directory (must contain an `addons` directory).
    Local(PathBuf),
    /// Workshop mod ID (requires `workshop_dir` on `Arma3Install`).
    WorkshopId(u64),
}

/// Parsed metadata for a mod (best-effort).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModMeta {
    /// Key-value data parsed from `*.cpp` files inside the mod directory.
    pub kv: BTreeMap<String, String>,
}

impl ModMeta {
    /// Best-effort name resolution similar to common Arma mod configs.
    pub fn name_or(&self, fallback: &str) -> String {
        for k in ["name", "dir", "tooltip", "publishedid"] {
            if let Some(v) = self.kv.get(k) {
                if !v.trim().is_empty() {
                    return v.clone();
                }
            }
        }
        fallback.to_string()
    }
}

/// Validate local mod directory (exists, contains `addons`).
pub fn validate_local_mod_dir(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(Arma3Error::InvalidModDir {
            path: path.to_path_buf(),
        });
    }
    if !path.join("addons").is_dir() {
        return Err(Arma3Error::InvalidModDir {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

/// Parse mod metadata from `*.cpp` files in the mod directory
pub fn read_mod_meta(mod_dir: &Path) -> Result<ModMeta> {
    validate_local_mod_dir(mod_dir)?;
    let mut meta = ModMeta::default();

    for ent in fs::read_dir(mod_dir)? {
        let ent = ent?;
        let p = ent.path();
        if p.extension() == Some(OsStr::new("cpp")) && p.is_file() {
            let text = fs::read_to_string(&p)?;
            parse_cpp_assignments(&text, &mut meta.kv);
        }
    }

    // If workshop id is missing, a common fallback is the directory name.
    if !meta.kv.contains_key("publishedid") {
        if let Some(name) = mod_dir.file_name().and_then(|s| s.to_str()) {
            meta.kv.insert("publishedid".into(), name.to_string());
        }
    }

    Ok(meta)
}

/// Parse `key="value";` and `key=value;` patterns in a tolerant way.
/// This is not a full C++ parser; it’s tuned for typical Arma mod config snippets.
fn parse_cpp_assignments(text: &str, out: &mut BTreeMap<String, String>) {
    // Strip block and line comments while respecting quotes.
    let stripped = strip_comments(text);

    // Split into statements by ';' outside quotes.
    for stmt in split_outside_quotes(&stripped, ';') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        // Find '=' outside quotes.
        if let Some(eq) = find_outside_quotes(stmt, '=') {
            let (k, v) = stmt.split_at(eq);
            let key = k.trim();
            let mut val = v[1..].trim(); // skip '='
            if key.is_empty() || val.is_empty() {
                continue;
            }
            // Drop surrounding quotes if present.
            val = val.trim_matches('"');
            out.entry(key.to_string())
                .or_insert_with(|| val.to_string());
        }
    }
}

fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_str = false;
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        let c = bytes[i] as char;

        if c == '"' {
            out.push(c);
            in_str = !in_str;
            i += 1;
            continue;
        }

        if !in_str {
            // line comment //
            if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
                i += 2;
                while i < bytes.len() && (bytes[i] as char) != '\n' {
                    i += 1;
                }
                continue;
            }
            // block comment /* ... */
            if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '*' {
                i += 2;
                while i + 1 < bytes.len() {
                    if (bytes[i] as char) == '*' && (bytes[i + 1] as char) == '/' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
        }

        out.push(c);
        i += 1;
    }
    out
}

fn split_outside_quotes(s: &str, delim: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_str = false;

    for c in s.chars() {
        if c == '"' {
            in_str = !in_str;
            cur.push(c);
            continue;
        }
        if c == delim && !in_str {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn find_outside_quotes(s: &str, needle: char) -> Option<usize> {
    let mut in_str = false;
    for (i, c) in s.char_indices() {
        if c == '"' {
            in_str = !in_str;
            continue;
        }
        if c == needle && !in_str {
            return Some(i);
        }
    }
    None
}
