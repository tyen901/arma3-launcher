use crate::steam::detect::detect_steam_root;
use crate::steam::vdf::Vdf;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

pub(crate) fn detect_steam_library_roots() -> Vec<PathBuf> {
    let Some(root) = detect_steam_root() else {
        return vec![];
    };

    let mut seen: BTreeSet<PathBuf> = BTreeSet::new();
    let mut out: Vec<PathBuf> = Vec::new();

    if root.join("steamapps").is_dir() && seen.insert(root.clone()) {
        out.push(root.clone());
    }

    let libraryfolders = root.join("steamapps/libraryfolders.vdf");
    let Ok(txt) = fs::read_to_string(&libraryfolders) else {
        return out;
    };
    let Ok(vdf) = Vdf::parse(&txt) else {
        return out;
    };

    for (k, v) in &vdf.kv {
        let mut parts = k.split('/');
        let a = parts.next();
        let b = parts.next();
        let c = parts.next();
        let d = parts.next();

        if a != Some("libraryfolders") {
            continue;
        }
        if b.is_none() {
            continue;
        }
        if c != Some("path") && d.is_some() {
            continue;
        }
        if c != Some("path") {
            continue;
        }

        let p = PathBuf::from(v);
        if p.join("steamapps").is_dir() && seen.insert(p.clone()) {
            out.push(p);
        }
    }

    out
}
