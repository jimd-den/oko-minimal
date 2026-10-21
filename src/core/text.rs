//! Laying out bitmap text into a frame.
//!
//! Text is drawn with the rectangle primitive, one glyph at a time, left to
//! right on a fixed six-pixel advance. There is no shaping, no kerning and no
//! fallback font: a character the built-in font does not have is a document
//! error, caught at parse time, so the renderer never has to guess.
//!
//! The pen is 64-bit and every write is clipped. A label that starts off the
//! frame, or runs past its right edge, paints the part that is visible and
//! nothing else.

use super::font;
use super::frame::Frame;
use super::geom::{Point, Rgba};

pub fn paint(frame: &mut Frame, at: Point, text: &str, scale: u8, colour: Rgba) {
    let scale = i64::from(scale.clamp(1, font::MAX_SCALE));
    let mut pen = (i64::from(at.x), i64::from(at.y));

    for character in text.chars() {
        let Some(glyph) = font::glyph(character) else {
            pen.0 += font::advance(scale);
            continue;
        };
        for row in 0..font::GLYPH_HEIGHT {
            for column in 0..font::GLYPH_WIDTH {
                if font::pixel(glyph, column, row) {
                    block(
                        frame,
                        pen.0 + column * scale,
                        pen.1 + row * scale,
                        scale,
                        colour,
                    );
                }
            }
        }
        pen.0 += font::advance(scale);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_i_is_a_column_with_two_rules() {
        let mut frame = Frame::new(5, 7).expect("a glyph-sized frame");

        paint(&mut frame, Point::new(0, 0), "I", 1, Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(4, 6)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 3)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 3)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_scale_multiplies_the_glyph() {
        let mut frame = Frame::new(10, 14).expect("a doubled frame");

        paint(&mut frame, Point::new(0, 0), "I", 2, Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(9, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(4, 6)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 6)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_space_leaves_the_frame_alone() {
        let mut frame = Frame::new(6, 7).expect("a small frame");

        paint(&mut frame, Point::new(0, 0), " ", 1, Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }
}
