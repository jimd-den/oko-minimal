//! The built-in bitmap font: five columns by seven rows, one bit per pixel.
//!
//! A console that needs a font file before it can show a login prompt is not a
//! console. This font is data compiled into the binary: 81 glyphs, no shaping,
//! no kerning, no licence to negotiate. It is drawn with the same rectangle
//! primitive as everything else, which is why text works on a framebuffer and
//! in a terminal without a second renderer.
//!
//! The glyphs themselves are in `glyphs`; this module is the font's
//! interface: the shape of a character and the distance the pen moves after
//! it.

pub const GLYPH_WIDTH: i64 = 5;
pub const GLYPH_HEIGHT: i64 = 7;

/// Five columns of glyph plus one column of space.
pub const ADVANCE: i64 = 6;

/// The document format's largest scale. The renderer repeats the cap so a
/// hand-built scene cannot make one frame paint a hundred million pixels.
pub const MAX_SCALE: u8 = 8;

/// The packed glyph for `character`, or `None` when the font has no such shape.
pub fn glyph(character: char) -> Option<u64> {
    GLYPHS
        .iter()
        .find(|(candidate, _)| *candidate == character)
        .map(|(_, bits)| *bits)
}

pub fn supports(character: char) -> bool {
    glyph(character).is_some()
}

/// Whether the pixel at `column`, `row` of the glyph is set.
pub fn pixel(bits: u64, column: i64, row: i64) -> bool {
    if !(0..GLYPH_WIDTH).contains(&column) || !(0..GLYPH_HEIGHT).contains(&row) {
        return false;
    }
    let shift = ((GLYPH_HEIGHT - 1 - row) * GLYPH_WIDTH + (GLYPH_WIDTH - 1 - column)) as u32;
    (bits >> shift) & 1 == 1
}

pub fn advance(scale: i64) -> i64 {
    ADVANCE * scale
}

mod glyphs;

use glyphs::GLYPHS;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lowercase_letter_has_its_own_shape() {
        assert_ne!(glyph('a'), glyph('A'), "a lowercase 'a' is not an 'A'");
        assert_ne!(glyph('l'), glyph('L'), "a lowercase 'l' is not an 'L'");
    }

    #[test]
    fn a_lowercase_letter_sits_on_the_baseline() {
        let a = glyph('a').expect("a lowercase 'a'");

        assert!(!pixel(a, 0, 0), "the top of the cell is empty");
        assert!(!pixel(a, 0, 1), "the ascender's row is empty");
        assert!(pixel(a, 1, 2), "the x-height begins in the third row");
        assert!(pixel(a, 1, 6), "the letter stands on the last row");
    }

    #[test]
    fn an_i_has_a_rule_at_the_top_and_bottom() {
        let i = glyph('I').expect("an I");

        assert!(pixel(i, 0, 0));
        assert!(pixel(i, 4, 0));
        assert!(pixel(i, 2, 3));
        assert!(!pixel(i, 0, 3));
        assert!(pixel(i, 4, 6));
    }

    #[test]
    fn a_space_is_blank() {
        let space = glyph(' ').expect("a space");

        assert!(
            (0..GLYPH_HEIGHT).all(|row| (0..GLYPH_WIDTH).all(|column| !pixel(space, column, row)))
        );
    }
}
