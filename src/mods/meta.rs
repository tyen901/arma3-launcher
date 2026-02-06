use crate::error::Result;
use crate::mods::validate::validate_local_mod_dir;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ModMeta {
    pub(crate) kv: BTreeMap<String, String>,
}

impl ModMeta {
    pub(crate) fn name_or(&self, fallback: &str) -> String {
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

pub(crate) fn read_mod_meta(mod_dir: &Path) -> Result<ModMeta> {
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

    if !meta.kv.contains_key("publishedid") {
        if let Some(name) = mod_dir.file_name().and_then(|s| s.to_str()) {
            meta.kv.insert("publishedid".into(), name.to_string());
        }
    }

    Ok(meta)
}

fn parse_cpp_assignments(text: &str, out: &mut BTreeMap<String, String>) {
    let stripped = strip_comments(text);

    for stmt in split_outside_quotes(&stripped, ';') {
        let stmt = stmt.trim();
        if stmt.is_empty() {
            continue;
        }
        if let Some(eq) = find_outside_quotes(stmt, '=') {
            let (k, v) = stmt.split_at(eq);
            let key = k.trim();
            let mut val = v[1..].trim();
            if key.is_empty() || val.is_empty() {
                continue;
            }
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
            if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
                i += 2;
                while i < bytes.len() && (bytes[i] as char) != '\n' {
                    i += 1;
                }
                continue;
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parses_cpp_key_values_best_effort() {
        let d = tempdir().unwrap();
        let mod_dir = d.path().join("@ace");
        fs::create_dir_all(mod_dir.join("addons")).unwrap();
        fs::write(mod_dir.join("addons").join("stub.pbo"), "data").unwrap();

        fs::write(
            mod_dir.join("mod.cpp"),
            r#"
            // comment
            name = "ACE3";
            tooltip="ACE Tooltip";
            /* block comment */ publishedid = "463939057";
            "#,
        )
        .unwrap();

        let meta = read_mod_meta(&mod_dir).unwrap();
        assert_eq!(meta.kv.get("name").unwrap(), "ACE3");
        assert_eq!(meta.kv.get("tooltip").unwrap(), "ACE Tooltip");
        assert_eq!(meta.kv.get("publishedid").unwrap(), "463939057");
    }
}
