use crate::error::{Arma3Error, Result};

/// Strip a C++-style class block, similar to the original project's `CppFilter`.
/// This scans braces while respecting quoted strings.
pub(crate) fn strip_cpp_class(text: &str, class_decl: &str) -> Result<String> {
    let mut s = text.to_string();
    let mut positions = Vec::new();
    let mut start = 0usize;

    while let Some(pos) = s[start..].find(class_decl) {
        positions.push(start + pos);
        start = start + pos + class_decl.len();
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
