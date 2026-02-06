use crate::error::{Arma3Error, Result};
use std::collections::BTreeMap;

/// Minimal VDF parser suitable for Steam `config.vdf`, `libraryfolders.vdf`, and `toolmanifest.vdf`.
///
/// This is a tolerant parser:
/// - keys/values are quoted strings
/// - nesting is `{ ... }`
///
/// Output keys are stored as `Path/To/Key`
#[derive(Debug, Default, Clone)]
pub(crate) struct Vdf {
    pub(crate) kv: BTreeMap<String, String>,
}

impl Vdf {
    pub(crate) fn parse(text: &str) -> Result<Self> {
        let mut p = Parser::new(text);
        p.parse()?;
        Ok(Self { kv: p.out })
    }

    pub(crate) fn get(&self, key: &str) -> Option<&String> {
        self.kv.get(key)
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    out: BTreeMap<String, String>,
    stack: Vec<String>,
    pending_key: Option<String>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            s: text.as_bytes(),
            i: 0,
            out: BTreeMap::new(),
            stack: Vec::new(),
            pending_key: None,
        }
    }

    fn parse(&mut self) -> Result<()> {
        while self.skip_ws() {
            match self.peek_char() {
                Some(b'"') => {
                    let tok = self.read_quoted()?;
                    self.skip_ws();
                    match self.peek_char() {
                        Some(b'"') => {
                            let val = self.read_quoted()?;
                            self.add_kv(tok, val);
                        }
                        Some(b'{') => {
                            self.i += 1;
                            self.stack.push(tok);
                        }
                        Some(b'}') => {
                            self.i += 1;
                            self.pop_stack();
                            self.pending_key = Some(tok);
                        }
                        _ => {
                            self.pending_key = Some(tok);
                        }
                    }
                }
                Some(b'{') => {
                    self.i += 1;
                    if let Some(k) = self.pending_key.take() {
                        self.stack.push(k);
                    }
                }
                Some(b'}') => {
                    self.i += 1;
                    self.pop_stack();
                }
                _ => {
                    self.i += 1;
                }
            }
        }

        if !self.stack.is_empty() {
            return Err(Arma3Error::SteamConfig {
                message: "unclosed braces in VDF".into(),
            });
        }
        Ok(())
    }

    fn add_kv(&mut self, k: String, v: String) {
        let mut path = String::new();
        for seg in &self.stack {
            path.push_str(seg);
            path.push('/');
        }
        path.push_str(&k);
        self.out.insert(path, v);
    }

    fn pop_stack(&mut self) {
        let _ = self.stack.pop();
    }

    fn skip_ws(&mut self) -> bool {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
        self.i < self.s.len()
    }

    fn peek_char(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn read_quoted(&mut self) -> Result<String> {
        if self.peek_char() != Some(b'"') {
            return Err(Arma3Error::SteamConfig {
                message: "expected quoted string".into(),
            });
        }
        self.i += 1;
        let mut out = Vec::new();
        let mut escape = false;

        while self.i < self.s.len() {
            let c = self.s[self.i];
            self.i += 1;

            if escape {
                out.push(c);
                escape = false;
                continue;
            }

            if c == b'\\' {
                escape = true;
                continue;
            }

            if c == b'"' {
                break;
            }

            out.push(c);
        }

        Ok(String::from_utf8(out)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let val = vdf
            .get("InstallConfigStore/Software/Valve/Steam/CompatToolMapping/107410/name")
            .unwrap();
        assert_eq!(val, "GE-Proton");
    }
}
