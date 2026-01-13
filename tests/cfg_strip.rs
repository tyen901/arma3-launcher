//! Integration tests for `strip_cpp_class`.
use arma3_launcher::strip_cpp_class;

#[test]
fn strips_existing_modlauncherlist() {
    let input = r#"
class SomethingElse { a=1; };
class ModLauncherList
{
    class Mod1 { dir="x"; };
};
class Tail { b=2; }
"#;

    let out = strip_cpp_class(input, "class ModLauncherList").unwrap();
    assert!(!out.contains("class ModLauncherList"));
    assert!(out.contains("class SomethingElse"));
    assert!(out.contains("class Tail"));
}
