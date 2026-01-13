//! Integration tests for VDF parsing.
use arma3_launcher::Vdf;

#[test]
fn parses_basic_vdf() {
    let txt = r#"
"InstallConfigStore"
{
  "Software"
  {
    "Valve"
    {
      "Steam"
      {
        "CompatToolMapping"
        {
          "107410"
          {
            "name" "GE-Proton"
          }
        }
      }
    }
  }
}
"#;

    let vdf = Vdf::parse(txt).unwrap();
    let vals = vdf.values_with_filter("CompatToolMapping/107410/name");
    assert_eq!(vals, vec!["GE-Proton".to_string()]);
}
