//! The text properties an export states.
//!
//! The export writes the space between the letters, the length it holds the
//! text to, and the end of the text its position refers to. Each is read
//! here, once, and handed to the emitter as a plain value.

use super::super::error::SvgError;
use super::super::xml::Element;
use super::pixels;

/// Which end of a text a position refers to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    /// The position is the left edge of the text.
    Start,
    /// The position is the middle of the text.
    Middle,
    /// The position is the right edge of the text.
    End,
}

/// A property an element states, in an attribute or in a style.
fn declared(element: &Element, name: &str) -> Option<String> {
    element.attribute(name).map(str::to_string).or_else(|| {
        element
            .attribute("style")
            .and_then(|style| super::super::style::property(style, name))
    })
}

/// The space between the letters, in pixels.
///
/// The unit is the one the design wrote: pixels, or a share of the font's
/// size. `normal` is no space at all.
pub(super) fn letter_spacing(element: &Element, size: f64) -> Result<Option<i64>, SvgError> {
    let Some(word) = declared(element, "letter-spacing") else {
        return Ok(None);
    };
    if word == "normal" {
        return Ok(Some(0));
    }
    let digits = word
        .chars()
        .take_while(|character| {
            character.is_ascii_digit() || *character == '.' || *character == '-'
        })
        .count();
    let (number, unit) = word.split_at(digits);
    let value: f64 = number
        .parse()
        .map_err(|_| SvgError::new(element.line, "a letter-spacing must be a number"))?;
    let pixels = match unit {
        "" | "px" => value,
        "em" => value * size,
        other => {
            return Err(SvgError::new(
                element.line,
                format!("a letter-spacing may not be in `{other}`"),
            ));
        }
    };
    Ok(Some(pixels.round() as i64))
}

/// Which end of the text a position refers to.
pub(super) fn anchor(element: &Element) -> Result<Option<Anchor>, SvgError> {
    match declared(element, "text-anchor").as_deref() {
        None | Some("start") => Ok(Some(Anchor::Start)),
        Some("middle") => Ok(Some(Anchor::Middle)),
        Some("end") => Ok(Some(Anchor::End)),
        Some(other) => Err(SvgError::new(
            element.line,
            format!("`{other}` is not a text anchor"),
        )),
    }
}

/// The length the design holds the text to, in pixels.
pub(super) fn length(element: &Element) -> Result<Option<i64>, SvgError> {
    let Some(word) = element.attribute("textLength") else {
        return Ok(None);
    };
    Ok(Some(pixels(word, element.line)? as i64))
}

/// Whether the design stretches the glyphs as well as the spacing.
pub(super) fn stretches(element: &Element) -> Result<bool, SvgError> {
    match element.attribute("lengthAdjust") {
        None | Some("spacing") => Ok(false),
        Some("spacingAndGlyphs") => Ok(true),
        Some(other) => Err(SvgError::new(
            element.line,
            format!("`{other}` is not a length adjustment"),
        )),
    }
}
