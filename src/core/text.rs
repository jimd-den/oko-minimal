//! Laying out bitmap text into a frame.
//!
//! Text is drawn with the rectangle primitive, one glyph at a time, left to
//! right. Each glyph carries its own width, so the pen moves by the width of
//! the letter that was drawn and not by a fixed pitch. There is no shaping,
//! no kerning and no fallback font: a character the built-in font does not
//! have is a document error, caught at parse time, so the renderer never has
//! to guess.
//!
//! The pen is 64-bit and every write is clipped. A label that starts off the
//! frame, or runs past its right edge, paints the part that is visible and
//! nothing else.

use super::font;
use super::frame::Frame;
use super::geom::{Point, Rgba};

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

/// The width of a text, in pixels: the width the design would give it.
///
/// A character the font does not have takes the widest advance, because a
/// hole in a word is still a space on the page.
pub fn width(text: &str, scale: u8, spacing: i64) -> i64 {
    let scale = i64::from(scale.clamp(1, font::MAX_SCALE));
    let mut wide = 0;
    for (index, character) in text.chars().enumerate() {
        let step = match font::glyph(character) {
            Some(glyph) => font::advance_of(glyph, scale),
            None => font::advance(scale),
        };
        wide += step;
        if index > 0 {
            wide += spacing;
        }
    }
    // The last advance carries a gap of its own, and the text does not.
    wide.saturating_sub(1)
}

pub fn paint(frame: &mut Frame, text: &Text) {
    let Text {
        at,
        text: label,
        scale,
        spacing,
        colour,
    } = text;
    let scale = i64::from((*scale).clamp(1, font::MAX_SCALE));
    let spacing = *spacing;
    let mut pen = (i64::from(at.x), i64::from(at.y));

    for character in label.chars() {
        let Some(glyph) = font::glyph(character) else {
            pen.0 += font::advance(scale);
            continue;
        };
        let (left, width) = font::ink(glyph);
        for row in 0..font::GLYPH_HEIGHT {
            for column in left..left + width {
                if font::pixel(glyph, column, row) {
                    block(
                        frame,
                        pen.0 + (column - left) * scale,
                        pen.1 + row * scale,
                        scale,
                        *colour,
                    );
                }
            }
        }
        pen.0 += font::advance_of(glyph, scale) + spacing;
    }
}

fn block(frame: &mut Frame, x: i64, y: i64, size: i64, colour: Rgba) {
    for dy in 0..size {
        for dx in 0..size {
            put(frame, x + dx, y + dy, colour);
        }
    }
}

fn put(frame: &mut Frame, x: i64, y: i64, colour: Rgba) {
    if x < 0 || y < 0 {
        return;
    }
    let (x, y) = (x as u32, y as u32);
    if x >= frame.width() || y >= frame.height() {
        return;
    }
    frame.set(Point::new(x as i32, y as i32), colour);
}
