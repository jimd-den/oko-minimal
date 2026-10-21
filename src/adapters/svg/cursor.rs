//! A byte cursor over the SVG text.
//!
//! All the low-level reading lives here: peeking, skipping, names, quoted
//! values, line counting. The grammar above it never indexes a string, and this
//! layer never decides what a tag means.

use super::error::SvgError;
use super::xml::MAX_VALUE;

pub struct Cursor<'a> {
    source: &'a str,
    pub at: usize,
    pub line: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            at: 0,
            line: 1,
        }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.at).copied()
    }

    pub fn begins(&self, prefix: &str) -> bool {
        self.source[self.at..].starts_with(prefix)
    }

    pub fn skip_to(&mut self, at: usize) {
        let bytes = self.source.as_bytes();
        self.line += bytes[self.at..at]
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count();
        self.at = at;
    }

    pub fn whitespace(&mut self) {
        while let Some(byte) = self.byte() {
            if byte == b'\n' {
                self.line += 1;
                self.at += 1;
            } else if byte.is_ascii_whitespace() {
                self.at += 1;
            } else {
                break;
            }
        }
    }

    pub fn expect(&mut self, expected: u8, problem: &str) -> Result<(), SvgError> {
        if self.byte() != Some(expected) {
            return Err(self.error(problem));
        }
        self.skip_to(self.at + 1);
        Ok(())
    }

    pub fn skip_past(&mut self, marker: &str, problem: &str) -> Result<(), SvgError> {
        let bytes = self.source.as_bytes();
        let needle = marker.as_bytes();
        let mut at = self.at;
        while at + needle.len() <= bytes.len() {
            if &bytes[at..at + needle.len()] == needle {
                self.skip_to(at + needle.len());
                return Ok(());
            }
            at += 1;
        }
        Err(self.error(problem))
    }

    pub fn name(&mut self) -> Result<String, SvgError> {
        let start = self.at;
        while let Some(byte) = self.byte() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-') {
                self.at += 1;
            } else {
                break;
            }
        }
        if self.at == start {
            return Err(self.error("expected a name"));
        }
        Ok(self.source[start..self.at].to_string())
    }

    pub fn quoted(&mut self) -> Result<String, SvgError> {
        let quote = match self.byte() {
            Some(quote @ (b'"' | b'\'')) => quote,
            _ => return Err(self.error("attribute values must be quoted")),
        };
        self.skip_to(self.at + 1);
        let start = self.at;

        while let Some(byte) = self.byte() {
            if byte == quote {
                if self.at - start > MAX_VALUE {
                    return Err(self.error("an attribute value is too long"));
                }
                let value = self.source[start..self.at].to_string();
                self.skip_to(self.at + 1);
                if value.contains('&') {
                    return Err(self.error("entities are not allowed"));
                }
                return Ok(value);
            }
            if byte == b'<' {
                return Err(self.error("`<` inside an attribute value"));
            }
            self.skip_to(self.at + 1);
        }

        Err(self.error("an unterminated attribute value"))
    }

    pub fn error(&self, problem: impl Into<String>) -> SvgError {
        SvgError::new(self.line, problem)
    }
}
