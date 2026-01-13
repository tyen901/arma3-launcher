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
pub struct Vdf {
    /// Flattened key/value store.
    pub kv: BTreeMap<String, String>,
}

impl Vdf {
    /// Parse VDF from text.
    pub fn parse(text: &str) -> Result<Self> {
        let mut p = Parser::new(text);
        p.parse()?;
        Ok(Self { kv: p.out })
    }

    /// Get all values whose key contains `filter`.
    pub fn values_with_filter(&self, filter: &str) -> Vec<String> {
        self.kv
            .iter()
            .filter_map(|(k, v)| {
                if k.contains(filter) {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect()
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
                            // key-value
                            let val = self.read_quoted()?;
                            self.add_kv(tok, val);
                        }
                        Some(b'{') => {
                            // key -> object
                            self.i += 1;
                            self.stack.push(tok);
                        }
                        Some(b'}') => {
                            // stray close; treat as pop then keep token as pending key
                            self.i += 1;
                            self.pop_stack();
                            self.pending_key = Some(tok);
                        }
                        _ => {
                            // Might be a bare brace or weird formatting; store as pending key.
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
                    // Unknown token; skip char.
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
        self.i += 1; // skip opening quote
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
