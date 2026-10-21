//! The words of a line, and the values among them.
//!
//! A document line is a list of words, but a label is written in double
//! quotes and may hold spaces, so a quoted run is one word however many
//! spaces it holds. The quotes stay on the word: a label that holds the
//! word `to` is not the keyword `to`.
//!
//! The values among the words are read by `core::syntax`, which knows a
//! coordinate's range, a size's positivity and a colour's six digits. This
//! module cuts a line into words and reads the values among them; it does
//! not decide what a line means.

use super::super::geom::{Point, Rgba};
use super::super::syntax;

/// The word that states a delay.
pub(crate) const AFTER: &str = "after";
/// The word that starts a leg.
pub(crate) const TO: &str = "to";
/// The word that states how long a leg takes.
pub(crate) const OVER: &str = "over";
/// The word that states the flag a shape is drawn under.
pub(crate) const WHEN: &str = "when";
/// The word that states the space between the letters of a text.
pub(crate) const SPACING: &str = "spacing";

/// The words of a line, and where we are among them.
#[derive(Debug)]
pub(crate) struct Words<'a> {
    words: Vec<&'a str>,
    at: usize,
}

impl<'a> Words<'a> {
    /// The words of a line, or a refusal when a label has no closing quote.
    pub(crate) fn new(source: &'a str) -> Result<Self, String> {
        Ok(Words {
            words: split(source)?,
            at: 0,
        })
    }

    /// The next word, and the step past it.
    pub(crate) fn next(&mut self) -> Option<&'a str> {
        let word = self.words.get(self.at).copied();
        if word.is_some() {
            self.at += 1;
        }
        word
    }

    /// The word `ahead` steps on, without moving.
    pub(crate) fn peek(&self, ahead: usize) -> Option<&'a str> {
        self.words.get(self.at + ahead).copied()
    }

    /// Whether the word `ahead` steps on is `keyword`.
    pub(crate) fn is(&self, ahead: usize, keyword: &str) -> bool {
        self.peek(ahead) == Some(keyword)
    }

    /// A coordinate, read from the next word.
    pub(crate) fn coordinate(&mut self) -> Result<i32, String> {
        let word = self.value().ok_or("expected a coordinate")?;
        Ok(syntax::coordinate_prefix(word)?.0)
    }

    /// A point, read from the next two words.
    pub(crate) fn point(&mut self) -> Result<Point, String> {
        let group = self.group(2);
        let mut parts = group.split_whitespace();
        syntax::point(&mut parts)
    }

    /// A colour, read from the next word.
    pub(crate) fn colour(&mut self) -> Result<Rgba, String> {
        let word = self.value().ok_or("expected a colour like #ff9d00")?;
        let mut parts = word.split_whitespace();
        syntax::colour(&mut parts)
    }

    /// A label, read from the next word.
    pub(crate) fn label(&mut self) -> Result<String, String> {
        let word = self.value().ok_or("expected a label in double quotes")?;
        Ok(syntax::quoted(word)?.0)
    }

    /// A scale, read from the next word.
    pub(crate) fn scale(&mut self) -> Result<u8, String> {
        let word = self.value().ok_or("expected a scale")?;
        Ok(syntax::scale_prefix(word)?.0)
    }

    /// A size, read from the next word: a width or a height.
    pub(crate) fn extent(&mut self) -> Result<u32, String> {
        let word = self.value().ok_or("expected a width or height")?;
        let value: i64 = word
            .parse()
            .map_err(|_| format!("`{word}` is not a number"))?;
        if !(1..=syntax::MAX_SIDE).contains(&value) {
            return Err(format!("size {value} is outside 1..={}", syntax::MAX_SIDE));
        }
        Ok(value as u32)
    }

    /// A whole number, read from the next word.
    pub(crate) fn number(&mut self) -> Result<i64, String> {
        let word = self.value().ok_or("expected a number")?;
        word.parse()
            .map_err(|_| format!("`{word}` is not a whole number"))
    }

    /// The end of the words: nothing may be left.
    pub(crate) fn end(&mut self) -> Result<(), String> {
        match self.next() {
            Some(word) => Err(format!("unexpected word `{word}`")),
            None => Ok(()),
        }
    }

    /// The next word, unless it is a keyword: a value is never a keyword.
    pub(crate) fn value(&mut self) -> Option<&'a str> {
        match self.peek(0) {
            Some(word) if !is_keyword(word) => {
                self.at += 1;
                Some(word)
            }
            _ => None,
        }
    }

    /// The next `count` words, joined into one piece of text.
    ///
    /// The syntax layer reads whitespace-separated text, and the words of a
    /// line are not contiguous in the source, so they are joined back
    /// together for it to read. A keyword ends the run: a value is never a
    /// keyword, so a keyword here means the value is missing.
    fn group(&mut self, count: usize) -> String {
        let mut taken = Vec::new();
        while taken.len() < count {
            match self.peek(0) {
                Some(word) if !is_keyword(word) => {
                    taken.push(word.to_string());
                    self.at += 1;
                }
                _ => break,
            }
        }
        taken.join(" ")
    }
}

/// Whether a word is one the document reserves for its grammar.
pub(crate) fn is_keyword(word: &str) -> bool {
    matches!(word, AFTER | TO | OVER | WHEN | SPACING)
}

/// The words of a line, a quoted run being one word.
fn split(source: &str) -> Result<Vec<&str>, String> {
    let bytes = source.as_bytes();
    let mut words = Vec::new();
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at].is_ascii_whitespace() {
            at += 1;
            continue;
        }
        let start = at;
        if bytes[at] == b'"' {
            let end = source[at + 1..]
                .find('"')
                .ok_or("a label needs a closing quote")?
                + at
                + 1;
            words.push(&source[start..=end]);
            at = end + 1;
        } else {
            while at < bytes.len() && !bytes[at].is_ascii_whitespace() {
                at += 1;
            }
            words.push(&source[start..at]);
        }
    }

    Ok(words)
}
