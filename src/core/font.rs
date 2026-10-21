//! The built-in bitmap font: five columns by seven rows, one bit per pixel.
//!
//! A console that needs a font file before it can show a login prompt is not a
//! console. This font is data compiled into the binary: 55 glyphs, no shaping,
//! no kerning, no licence to negotiate. It is drawn with the same rectangle
//! primitive as everything else, which is why text works on a framebuffer and
//! in a terminal without a second renderer.
//!
//! Each glyph is seven rows of five bits, packed into a `u64`, top row first.
//! The literal is written one group per row so the shape is visible in source.
//! Lowercase letters are drawn as uppercase; the document format is a panel,
//! not prose.
//!
//! The shapes are original work for this project and are released with it.

pub const GLYPH_WIDTH: i64 = 5;
pub const GLYPH_HEIGHT: i64 = 7;

/// Five columns of glyph plus one column of space.
pub const ADVANCE: i64 = 6;

/// The document format's largest scale. The renderer repeats the cap so a
/// hand-built scene cannot make one frame paint a hundred million pixels.
pub const MAX_SCALE: u8 = 8;

/// The packed glyph for `character`, or `None` when the font has no such shape.
pub fn glyph(character: char) -> Option<u64> {
    let wanted = character.to_ascii_uppercase();
    GLYPHS
        .iter()
        .find(|(candidate, _)| *candidate == wanted)
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

const GLYPHS: &[(char, u64)] = &[
    (' ', 0b00000_00000_00000_00000_00000_00000_00000),
    ('A', 0b01110_10001_10001_11111_10001_10001_10001),
    ('B', 0b11110_10001_10001_11110_10001_10001_11110),
    ('C', 0b01110_10001_10000_10000_10000_10001_01110),
    ('D', 0b11110_10001_10001_10001_10001_10001_11110),
    ('E', 0b11111_10000_10000_11110_10000_10000_11111),
    ('F', 0b11111_10000_10000_11110_10000_10000_10000),
    ('G', 0b01110_10001_10000_10111_10001_10001_01110),
    ('H', 0b10001_10001_10001_11111_10001_10001_10001),
    ('I', 0b11111_00100_00100_00100_00100_00100_11111),
    ('J', 0b00111_00010_00010_00010_00010_10010_01100),
    ('K', 0b10001_10010_10100_11000_10100_10010_10001),
    ('L', 0b10000_10000_10000_10000_10000_10000_11111),
    ('M', 0b10001_11011_10101_10101_10001_10001_10001),
    ('N', 0b10001_11001_10101_10011_10001_10001_10001),
    ('O', 0b01110_10001_10001_10001_10001_10001_01110),
    ('P', 0b11110_10001_10001_11110_10000_10000_10000),
    ('Q', 0b01110_10001_10001_10001_10101_10010_01101),
    ('R', 0b11110_10001_10001_11110_10100_10010_10001),
    ('S', 0b01111_10000_10000_01110_00001_00001_11110),
    ('T', 0b11111_00100_00100_00100_00100_00100_00100),
    ('U', 0b10001_10001_10001_10001_10001_10001_01110),
    ('V', 0b10001_10001_10001_10001_10001_01010_00100),
    ('W', 0b10001_10001_10001_10101_10101_11011_10001),
    ('X', 0b10001_10001_01010_00100_01010_10001_10001),
    ('Y', 0b10001_10001_01010_00100_00100_00100_00100),
    ('Z', 0b11111_00001_00010_00100_01000_10000_11111),
    ('0', 0b01110_10001_10011_10101_11001_10001_01110),
    ('1', 0b00100_01100_00100_00100_00100_00100_01110),
    ('2', 0b01110_10001_00001_00010_00100_01000_11111),
    ('3', 0b11111_00010_00100_00010_00001_10001_01110),
    ('4', 0b00010_00110_01010_10010_11111_00010_00010),
    ('5', 0b11111_10000_11110_00001_00001_10001_01110),
    ('6', 0b00110_01000_10000_11110_10001_10001_01110),
    ('7', 0b11111_00001_00010_00100_01000_01000_01000),
    ('8', 0b01110_10001_10001_01110_10001_10001_01110),
    ('9', 0b01110_10001_10001_01111_00001_00010_01100),
    ('.', 0b00000_00000_00000_00000_00000_01100_01100),
    (',', 0b00000_00000_00000_00000_01100_01100_01000),
    (':', 0b00000_01100_01100_00000_01100_01100_00000),
    ('-', 0b00000_00000_00000_11111_00000_00000_00000),
    ('+', 0b00000_00100_00100_11111_00100_00100_00000),
    ('/', 0b00001_00001_00010_00100_01000_10000_10000),
    ('%', 0b11001_11001_00010_00100_01000_10011_10011),
    ('?', 0b01110_10001_00001_00010_00100_00000_00100),
    ('!', 0b00100_00100_00100_00100_00100_00000_00100),
    ('\'', 0b00100_00100_00000_00000_00000_00000_00000),
    ('(', 0b00010_00100_01000_01000_01000_00100_00010),
    (')', 0b01000_00100_00010_00010_00010_00100_01000),
    ('[', 0b01110_01000_01000_01000_01000_01000_01110),
    (']', 0b01110_00010_00010_00010_00010_00010_01110),
    ('<', 0b00010_00100_01000_10000_01000_00100_00010),
    ('>', 0b01000_00100_00010_00001_00010_00100_01000),
    ('=', 0b00000_00000_11111_00000_11111_00000_00000),
    ('_', 0b00000_00000_00000_00000_00000_00000_11111),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_is_drawn_as_uppercase() {
        assert_eq!(glyph('a'), glyph('A'));
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
