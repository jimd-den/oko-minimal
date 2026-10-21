//! Reading numbers and command letters out of a `d` attribute.
//!
//! The path grammar lives one level up, in `path`. This reader only knows what a
//! number looks like, which characters separate tokens, and where a command
//! letter is. It never decides what a command means.

use super::super::error::SvgError;

/// The largest coordinate a path may name.
const MAX_COORDINATE: f64 = 1_000_000.0;

pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    line: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn new(source: &'a str, line: usize) -> Self {
        Self {
            bytes: source.as_bytes(),
            at: 0,
            line,
        }
    }

    pub(super) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    pub(super) fn separators(&mut self) {
        while matches!(
            self.peek(),
            Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b',')
        ) {
            self.at += 1;
        }
    }

    /// The next command letter, consumed, or `None` when the next token is not
    /// one. A letter is always a command; a number is never mistaken for one.
    pub(super) fn command(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        if byte.is_ascii_alphabetic() {
            self.at += 1;
            Some(byte)
        } else {
            None
        }
    }

    pub(super) fn number(&mut self) -> Result<f64, SvgError> {
        self.separators();
        let start = self.at;
        if matches!(self.peek(), Some(b'-') | Some(b'+')) {
            self.at += 1;
        }

        let mut digits = false;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            digits = true;
            self.at += 1;
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                digits = true;
                self.at += 1;
            }
        }
        if !digits {
            return Err(SvgError::new(self.line, "expected a number in the path"));
        }

        let text = std::str::from_utf8(&self.bytes[start..self.at])
            .map_err(|_| SvgError::new(self.line, "the path is not valid UTF-8"))?;
        let value: f64 = text
            .parse()
            .map_err(|_| SvgError::new(self.line, format!("`{text}` is not a number")))?;
        if !value.is_finite() || value.abs() > MAX_COORDINATE {
            return Err(SvgError::new(
                self.line,
                "a path coordinate is outside the drawable range",
            ));
        }
        Ok(value)
    }

    pub(super) fn pair(&mut self) -> Result<(f64, f64), SvgError> {
        Ok((self.number()?, self.number()?))
    }
}
