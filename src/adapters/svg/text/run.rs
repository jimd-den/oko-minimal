//! The label a run of text becomes, and the directive that draws it.
//!
//! The rules are the decision's: a run is trimmed and its whitespace collapses,
//! every character must exist in the built-in font, and the size of the font
//! becomes the scale of the glyph. What the document grammar cannot hold, a
//! double quote in a label, is refused rather than escaped.

use super::Style;
use crate::core::font;
use crate::core::geom::Rgba;
use crate::core::syntax::{MAX_COORDINATE, MAX_TEXT_CHARS};

use super::super::error::SvgError;

/// The scale a label is drawn at: the font size measured in glyph heights.
///
/// Seven pixels is the height of the built-in glyph, so a seven-pixel label is
/// scale one and a twenty-one-pixel label is scale three.
pub fn scale(size: f64) -> u8 {
    let wanted = (size / font::GLYPH_HEIGHT as f64).round() as i64;
    wanted.clamp(1, i64::from(font::MAX_SCALE)) as u8
}

/// The label a run of text becomes, or `None` when the run holds no word.
///
/// The run is trimmed and its internal whitespace collapses to one space, so
/// an exporter's line breaks do not reach the document. Every character must
/// exist in the built-in font: a label the console cannot draw is refused
/// here rather than painted as something else.
pub fn label(run: &str, line: usize) -> Result<Option<String>, SvgError> {
    let words: Vec<&str> = run.split_whitespace().collect();
    if words.is_empty() {
        return Ok(None);
    }
    let label = words.join(" ");
    if label.contains('"') {
        return Err(SvgError::new(
            line,
            "a label may not contain a double quote",
        ));
    }
    for character in label.chars() {
        if !font::supports(character) {
            return Err(SvgError::new(
                line,
                format!("`{character}` is not in the built-in font"),
            ));
        }
    }
    if label.chars().count() > MAX_TEXT_CHARS {
        return Err(SvgError::new(
            line,
            format!("a label may hold at most {MAX_TEXT_CHARS} characters"),
        ));
    }
    Ok(Some(label))
}

/// Write one label as a document directive.
pub fn emit(
    document: &mut String,
    style: &Style,
    label: &str,
    line: usize,
) -> Result<(), SvgError> {
    let Some(colour) = style.colour else {
        return Ok(());
    };
    let scale = scale(style.size);

    // SVG places a label by its baseline; the document places it by its top
    // edge. The built-in font has no descenders, so its bottom row is the
    // baseline and the distance between the two is exactly the glyph height.
    let height = font::GLYPH_HEIGHT as i32 * i32::from(scale);
    let top = style.y - height;

    for value in [style.x, top] {
        if i64::from(value).abs() > MAX_COORDINATE {
            return Err(SvgError::new(
                line,
                format!("coordinates must be within \u{b1}{MAX_COORDINATE}"),
            ));
        }
    }
    document.push_str(&format!(
        "text {} {} \"{}\" {} {}\n",
        style.x,
        top,
        label,
        scale,
        hex(colour)
    ));
    Ok(())
}

fn hex(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_twenty_one_pixel_label_is_scale_three() {
        assert_eq!(scale(21.0), 3);
    }

    #[test]
    fn a_scale_above_the_limit_is_capped() {
        assert_eq!(scale(200.0), font::MAX_SCALE);
    }

    #[test]
    fn a_label_trims_and_collapses_its_whitespace() {
        assert_eq!(label("  A   B ", 1), Ok(Some("A B".to_string())));
    }

    #[test]
    fn a_run_of_whitespace_is_no_label() {
        assert_eq!(label("   ", 1), Ok(None));
    }

    #[test]
    fn a_character_outside_the_font_is_named() {
        let error = label("A~", 3).expect_err("a tilde");

        assert_eq!(error.line, 3);
        assert!(error.problem.contains("`~`"));
    }
}
