//! Laying out text into a frame, in the built-in font or in a named face.
//!
//! Two fonts draw a label. The built-in font is compiled into the program
//! and needs nothing else; a named face is a real font the program
//! carries. A panel names its font, and the text does not change when the
//! font does: the number in a text directive is always the built-in font's
//! scale (ADR 0014).
//!
//! A named face is drawn so that its capitals are as tall as the built-in
//! font's capitals at that scale, and its baseline is where the built-in
//! font puts it. The letters are the face's own outlines, and the pen
//! moves by the glyph's own advance.

use super::font;
use super::frame::Frame;
use super::geom::{Point, Rgba};
use super::typeface::Setting;

mod builtin;
mod named;
#[cfg(test)]
mod named_tests;
#[cfg(test)]
mod tests;

pub use builtin::width;

/// A text to draw: what it says, how big, how far apart its letters are,
/// and in what colour.
///
/// The spacing is the space the design put between the letters. The font's
/// own advance is the floor; the spacing is what the design asked for on
/// top of it (ADR 0010).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text<'a> {
    pub at: Point,
    pub text: &'a str,
    pub scale: u8,
    pub spacing: i64,
    pub colour: Rgba,
}

/// The height of a capital, in pixels.
///
/// The built-in font's capitals fill its cell, so the cell's height is the
/// capital height at every scale. A named face is drawn to the same
/// capital height, so that a change of font changes the letters and not
/// the size of the label (ADR 0014).
pub(crate) fn cap_height(scale: u8) -> i64 {
    font::GLYPH_HEIGHT * i64::from(scale.clamp(1, font::MAX_SCALE))
}

/// The baseline of a text: where the letters stand.
///
/// The built-in font's baseline is the bottom row of its cell, and its
/// capitals fill the cell, so the baseline is a capital height below the
/// place the text names.
pub(crate) fn baseline(at: Point, scale: u8) -> Point {
    Point::new(at.x, at.y + cap_height(scale) as i32)
}

/// Paint the text in the font given: the named face, or the built-in one.
pub fn paint(frame: &mut Frame, text: &Text, face: Option<&Setting>) {
    match face {
        Some(setting) => named::paint(frame, text, setting),
        None => builtin::paint(frame, text),
    }
}
