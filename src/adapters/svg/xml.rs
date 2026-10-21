//! The shape of an export, as the reader sees it.
//!
//! Nothing here reads bytes; `scanner` does that. These are the plain values
//! the translator can read without thinking about quoting or whitespace.

use super::error::SvgError;

pub const MAX_ELEMENTS: usize = 4096;
pub const MAX_ATTRIBUTES: usize = 64;

/// An attribute value is bounded by the file that holds it, which is capped at
/// one mebibyte: an export embeds a base64 image in an `href`.
pub const MAX_VALUE: usize = 1 << 20;

/// One element of an export.
#[derive(Clone, Debug, Default)]
pub struct Element {
    /// The tag name, as it was written.
    pub name: String,
    /// The attributes, in the order they were written.
    pub attributes: Vec<(String, String)>,
    /// The line the tag was on.
    pub line: usize,
    /// What the element holds.
    pub children: Vec<Node>,
}

/// What an element holds.
#[derive(Clone, Debug)]
pub enum Node {
    /// Another element.
    Element(Element),
    /// A run of character data.
    Text(Text),
}

/// A run of character data.
#[derive(Clone, Debug)]
pub struct Text {
    /// The characters.
    pub content: String,
    /// The line the run was on.
    pub line: usize,
}

impl Element {
    /// The value of an attribute, if it has one.
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// A refusal, with this element's line.
    pub fn error(&self, problem: impl Into<String>) -> SvgError {
        SvgError::new(self.line, problem)
    }
}
