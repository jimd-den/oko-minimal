//! Drawing a label in the built-in font.
//!
//! The built-in font is five columns by seven rows, one bit per pixel, and
//! it is drawn with the rectangle primitive: a letter is a set of blocks,
//! and a block is a rectangle. The pen moves by the width of the letter it
//! drew, so a word is as wide as its letters and not as wide as the widest
//! letter times the number of letters.

use super::Text;
use crate::core::font;
use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};

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

/// Paint the text in the built-in font.
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

/// A block of the letter: a square of pixels, the font's own pixel.
fn block(frame: &mut Frame, x: i64, y: i64, size: i64, colour: Rgba) {
    for dy in 0..size {
        for dx in 0..size {
            put(frame, x + dx, y + dy, colour);
        }
    }
}

/// Put one pixel, if it is on the frame.
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
