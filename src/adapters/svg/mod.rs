//! SVG in, a panel document out.
//!
//! Penpot and Figma can both export SVG. Rather than embed either editor, the
//! console accepts their export through one bounded translator whose output is
//! the same text a person edits by hand. The document stays the source of truth:
//! readable, diffable, hot-reloadable, and independent of the tool that drew it.
//!
//! The translator is a whitelist, not a renderer. It reads `rect`, `line`,
//! `polyline`, `path` with its curves, `text` with its `tspan` lines, and `g`
//! with a translation. What it cannot draw it does not approximate: an arc, a
//! paint server, a transform it does not implement, an element outside the
//! whitelist, and a character the built-in font does not have are left out of
//! the document, and the report says which and why. A panel with holes is
//! still a panel, and the holes are visible.
//!
//! This is the only new component in the authoring path. If the translator is
//! wrong, the document it wrote is still readable, and the fix is a diff.

mod colour;
mod cursor;
mod elements;
mod emit;
mod entity;
mod error;
mod label;
mod names;
mod outcome;
mod paint;
pub mod path;
mod rectangle;
mod scanner;
mod scope;
mod shapes;
mod style;
mod tag;
mod text;
mod transform;
mod translator;
mod values;
mod xml;

#[cfg(test)]
mod tests;

pub use error::SvgError;

/// One mebibyte of SVG is a large hand-drawn panel; beyond it, the file is not
/// an export of a console screen.
pub const MAX_INPUT: usize = 1 << 20;

/// What one translation produced.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Translation {
    /// The panel document, one directive per line.
    pub document: String,
    /// The shapes that were left out, and why.
    pub skipped: Vec<Skipped>,
}

/// One shape that was left out of the document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    /// The line of the export the shape was on.
    pub line: usize,
    /// Why the shape was not drawn, in words.
    pub reason: String,
}

/// Translate an export into a panel document and a report of what was left out.
///
/// Contract: the document is a document the parser accepts, and every shape
/// that is not in it is named in `skipped` with the line it was on. A file that
/// cannot be read at all is an error.
pub fn translate(source: &str) -> Result<Translation, SvgError> {
    if source.len() > MAX_INPUT {
        return Err(SvgError::new(1, "the SVG is larger than 1 MiB"));
    }
    translator::Translator::default().run(&scanner::document(source)?)
}
