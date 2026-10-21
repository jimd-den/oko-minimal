//! The colour names an export writes.
//!
//! Figma writes `fill="white"` and `stroke="black"` where a colour is meant,
//! and a translator that knows only hexadecimal digits refuses a whole file
//! over two words. This is the CSS basic set: the sixteen names a browser has
//! always known.

use crate::core::geom::Rgba;

/// The CSS basic colour names, and the colour each one names.
const NAMES: [(&str, (u8, u8, u8)); 16] = [
    ("aqua", (0, 255, 255)),
    ("black", (0, 0, 0)),
    ("blue", (0, 0, 255)),
    ("fuchsia", (255, 0, 255)),
    ("gray", (128, 128, 128)),
    ("green", (0, 128, 0)),
    ("lime", (0, 255, 0)),
    ("maroon", (128, 0, 0)),
    ("navy", (0, 0, 128)),
    ("olive", (128, 128, 0)),
    ("purple", (128, 0, 128)),
    ("red", (255, 0, 0)),
    ("silver", (192, 192, 192)),
    ("teal", (0, 128, 128)),
    ("white", (255, 255, 255)),
    ("yellow", (255, 255, 0)),
];

/// The colour a name names, if it names one.
///
/// A name is matched without regard to case, as CSS matches it.
pub fn colour(word: &str) -> Option<Rgba> {
    let word = word.trim().to_ascii_lowercase();
    NAMES
        .iter()
        .find(|(name, _)| *name == word)
        .map(|(_, (r, g, b))| Rgba::rgb(*r, *g, *b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_are_named() {
        assert_eq!(colour("black"), Some(Rgba::rgb(0, 0, 0)));
        assert_eq!(colour("white"), Some(Rgba::rgb(255, 255, 255)));
    }

    #[test]
    fn a_name_is_matched_without_regard_to_case() {
        assert_eq!(colour("White"), Some(Rgba::rgb(255, 255, 255)));
    }

    #[test]
    fn a_word_that_is_not_a_name_names_no_colour() {
        assert_eq!(colour("chartreuse"), None);
    }
}
