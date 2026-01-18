//! Integration tests for parsing Arma 3 mod metadata.
use arma3_launcher::{read_mod_meta, validate_local_mod_dir};
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

    validate_local_mod_dir(&mod_dir).unwrap();
    let meta = read_mod_meta(&mod_dir).unwrap();
    assert_eq!(meta.kv.get("name").unwrap(), "ACE3");
    assert_eq!(meta.kv.get("tooltip").unwrap(), "ACE Tooltip");
    assert_eq!(meta.kv.get("publishedid").unwrap(), "463939057");
}
