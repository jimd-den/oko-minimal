//! The label a run of text becomes, and the directive that draws it.
//!
//! The rules are the decision's: a run is trimmed and its whitespace collapses,
//! every character must exist in the built-in font, and the size of the font
//! becomes the scale of the glyph. What the document grammar cannot hold, a
//! double quote in a label, is refused rather than escaped.

use super::super::emit::Writer;
use super::super::error::SvgError;
use super::Anchor;
use super::Style;
use crate::core::font;
use crate::core::geom::Rgba;
use crate::core::syntax::{MAX_COORDINATE, MAX_TEXT_CHARS};

/// What a run of text turned out to be.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    /// A label the document can hold.
    Text(String),
    /// A label the built-in font cannot draw, and why.
    Undrawable(String),
}

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
/// The run is trimmed and its internal whitespace collapses to one space, so an
/// exporter's line breaks do not reach the document. Every character must exist
/// in the built-in font: a label the console cannot draw is reported and left
/// out, rather than painted as something else.
pub fn label(run: &str, line: usize) -> Result<Option<Label>, SvgError> {
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
    if label.chars().count() > MAX_TEXT_CHARS {
        return Err(SvgError::new(
            line,
            format!("a label may hold at most {MAX_TEXT_CHARS} characters"),
        ));
    }
    for character in label.chars() {
        if !font::supports(character) {
            return Ok(Some(Label::Undrawable(format!(
                "the character `{character}` is not in the built-in font"
            ))));
        }
    }
    Ok(Some(Label::Text(label)))
}

/// Write one label as a document directive.
///
/// A label is one operation, whatever its length: the document counts
/// directives, not characters.
pub fn emit(
    document: &mut Writer,
    style: &Style,
    label: &str,
    colour: Rgba,
    line: usize,
) -> Result<Option<String>, SvgError> {
    let scale = scale(style.size);
    let (spacing, note) = fitted_spacing(style, label, scale);
    let note = note.or_else(|| {
        style.stretches.then(|| {
            "the text is fitted by its spacing; the glyphs are the font's own and are not stretched"
                .to_string()
        })
    });
    let width = crate::core::text::width(label, scale, spacing);
    let x = match style.anchor {
        Anchor::Start => style.x,
        Anchor::Middle => style.x - (width / 2) as i32,
        Anchor::End => style.x - width as i32,
    };

    // SVG places a label by its baseline; the document places it by its top
    // edge. The built-in font has no descenders, so its bottom row is the
    // baseline and the distance between the two is exactly the glyph height.
    let height = font::GLYPH_HEIGHT as i32 * i32::from(scale);
    let top = style.y - height;

    for value in [x, top] {
        if i64::from(value).abs() > MAX_COORDINATE {
            return Err(SvgError::new(
                line,
                format!("coordinates must be within \u{b1}{MAX_COORDINATE}"),
            ));
        }
    }
    let spacing = if spacing == 0 {
        String::new()
    } else {
        format!(" spacing {spacing}")
    };
    document.write(
        &format!(
            "text {} {} \"{}\" {} {}{}\n",
            x,
            top,
            label,
            scale,
            hex(colour),
            spacing
        ),
        line,
    )?;
    Ok(note)
}

/// The spacing that fills the length the design asked for.
///
/// The letters keep their own width and the space between them grows or
/// shrinks, which is what a browser does with `lengthAdjust="spacing"`. A
/// design that stretches the glyphs as well is fitted the same way, and the
/// report says that the letterforms are the font's own.
fn fitted_spacing(style: &Style, label: &str, scale: u8) -> (i64, Option<String>) {
    let Some(length) = style.length else {
        return (style.spacing, None);
    };
    let gaps = label.chars().count().saturating_sub(1) as i64;
    if gaps == 0 {
        return (style.spacing, None);
    }
    let natural = crate::core::text::width(label, scale, style.spacing);
    let extra = length - natural;
    if extra < 0 {
        // The built-in font is wider than the face the design was set in, so
        // a length shorter than the text would overlap the letters. The
        // panel is drawn at the font's own width and the report says so.
        return (
            style.spacing,
            Some(format!(
                "the text is wider than the length the design gave it ({length} against {natural}), so it is drawn at the font's own width"
            )),
        );
    }
    (style.spacing + extra / gaps, None)
}

fn hex(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}
