//! Where a run of SVG text sits, and how big it is.
//!
//! SVG text carries a font, a size, a baseline and a fill. The document
//! carries a label, a scale and a colour, so the translator makes exactly one
//! approximation: a font size becomes a glyph scale, because the built-in font
//! is seven pixels tall and SVG measures an em box. That approximation is the
//! decision in `docs/adr/0003-svg-text-mapping.md`; everything here is that
//! decision written as code.
//!
//! This module holds the placement rules. The label rules live in `run`. The
//! translator hands over a `Style` for the element a run belongs to, and gets
//! back a document line or a refusal with a line number.

mod run;
#[cfg(test)]
mod tests;

pub use run::{emit, label};

use super::error::SvgError;
use super::paint;
use super::values;
use super::xml::Element;
use crate::core::geom::Rgba;

/// A missing `font-size` means a seven-pixel label, which is scale one.
const DEFAULT_SIZE: f64 = 7.0;

/// Where one run of text is drawn, and how big it is.
#[derive(Clone, Debug)]
pub struct Style {
    /// The origin of the text's own coordinate space, from the group around it.
    pub origin: (i32, i32),
    pub x: i32,
    pub y: i32,
    pub size: f64,
    /// `None` when the text is not painted at all, which `fill="none"` means.
    pub colour: Option<Rgba>,
}

impl Style {
    /// The style a `<text>` element starts with.
    pub fn of_text(element: &Element, origin: (i32, i32)) -> Result<Style, SvgError> {
        Ok(Style {
            origin,
            x: origin.0 + position(element, "x")?,
            y: origin.1 + position(element, "y")? + offset(element, "dy")?,
            size: declared_size(element)?.unwrap_or(DEFAULT_SIZE),
            colour: paint::fill(element)?,
        })
    }

    /// The style a `<tspan>` has inside its `<text>`.
    ///
    /// A span keeps what it does not say: its position, its size and its
    /// colour are the text's, unless the span states its own. `dy` adds to
    /// the baseline it inherits, which is how an exporter writes a second line.
    pub fn of_span(&self, element: &Element, shift: (i32, i32)) -> Result<Style, SvgError> {
        let origin = (self.origin.0 + shift.0, self.origin.1 + shift.1);
        let x = match element.attribute("x") {
            Some(_) => origin.0 + position(element, "x")?,
            None => self.x + shift.0,
        };
        let y = match element.attribute("y") {
            Some(_) => origin.1 + position(element, "y")?,
            None => self.y + shift.1,
        };
        Ok(Style {
            origin,
            x,
            y: y + offset(element, "dy")?,
            size: declared_size(element)?.unwrap_or(self.size),
            colour: match element.attribute("fill") {
                Some(_) => paint::fill(element)?,
                None => self.colour,
            },
        })
    }
}

/// The font size an element states, if it states one.
fn declared_size(element: &Element) -> Result<Option<f64>, SvgError> {
    if let Some(word) = element.attribute("font-size") {
        return Ok(Some(pixels(word, element.line)?));
    }
    if let Some(word) = element.attribute("style")
        && let Some(value) = style_property(word, "font-size")
    {
        return Ok(Some(pixels(&value, element.line)?));
    }
    Ok(None)
}

/// One property of a `style` attribute, if it is there.
fn style_property(style: &str, property: &str) -> Option<String> {
    style.split(';').find_map(|declaration| {
        let (name, value) = declaration.split_once(':')?;
        if name.trim() == property {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

/// A length in pixels. The unit is optional and must be `px`, the only unit
/// an absolute SVG length may carry without a font to measure it against.
fn pixels(word: &str, line: usize) -> Result<f64, SvgError> {
    let trimmed = word.trim();
    let number = trimmed.strip_suffix("px").unwrap_or(trimmed);
    values::scalar(number, line)
}

/// A position attribute, or zero when the element does not state one.
fn position(element: &Element, name: &str) -> Result<i32, SvgError> {
    match element.attribute(name) {
        Some(word) => values::integer(word, element.line),
        None => Ok(0),
    }
}

/// A shift attribute, such as `dy`, or zero when the element does not state one.
fn offset(element: &Element, name: &str) -> Result<i32, SvgError> {
    match element.attribute(name) {
        Some(word) => Ok(pixels(word, element.line)?.round() as i32),
        None => Ok(0),
    }
}

#[cfg(test)]
mod placement {
    use super::*;

    fn element(attributes: &[(&str, &str)]) -> Element {
        Element {
            name: "tspan".to_string(),
            attributes: attributes
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            closed: true,
            line: 1,
        }
    }

    #[test]
    fn a_span_keeps_the_size_of_its_text() {
        let text = Style::of_text(&element(&[("font-size", "21")]), (0, 0)).expect("a text");

        let span = text.of_span(&element(&[]), (0, 0)).expect("a span");

        assert_eq!(span.size, 21.0);
    }

    #[test]
    fn a_span_may_state_its_own_size() {
        let text = Style::of_text(&element(&[("font-size", "21")]), (0, 0)).expect("a text");

        let span = text
            .of_span(&element(&[("font-size", "7")]), (0, 0))
            .expect("a span");

        assert_eq!(span.size, 7.0);
    }

    #[test]
    fn a_size_in_a_style_attribute_is_read() {
        let text = Style::of_text(&element(&[("style", "fill: red; font-size: 14px")]), (0, 0))
            .expect("a text");

        assert_eq!(text.size, 14.0);
    }
}
