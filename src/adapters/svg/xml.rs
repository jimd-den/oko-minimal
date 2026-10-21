//! The shape of an XML element, as the translator sees it.
//!
//! Nothing here reads bytes; `scanner` does that. These are the plain values it
//! produces, so the translator can be read without thinking about quoting or
//! whitespace.

use super::error::SvgError;

pub const MAX_ELEMENTS: usize = 4096;
pub const MAX_ATTRIBUTES: usize = 64;
pub const MAX_VALUE: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub closed: bool,
    pub line: usize,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn error(&self, problem: impl Into<String>) -> SvgError {
        SvgError::new(self.line, problem)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Start(Element),
    End {
        name: String,
        line: usize,
    },
    /// A run of character data. Whitespace-only runs are not reported.
    Text {
        content: String,
        line: usize,
    },
}
