//! How a shape is painted: the fill and the stroke in force.
//!
//! SVG paints with inherited presentation attributes: a fill stated on a group
//! is inherited by the shapes inside it, and a shape that states nothing takes
//! what its ancestors said. Both exporters put a fill on the root `<svg>`, so
//! inheritance is not a corner case; it is how a panel is painted at all.
//!
//! A paint may also name a paint server, `url(#gradient)`. The translator
//! cannot resolve one, so the shape it paints is left out and the reference is
//! named in the report.

mod tests;

use super::colour::{self, Colour};
use super::style;
use super::xml::Element;
use crate::core::geom::Rgba;

/// A paint: what a shape is covered in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Paint {
    /// A flat colour.
    Colour(Rgba),
    /// Nothing is painted: `fill="none"`.
    Nothing,
    /// A paint this console cannot draw, and why.
    Unsupported(String),
}

/// The paints an element and the elements inside it inherit.
///
/// A `None` means nothing has been stated yet, so the default applies. A
/// `Nothing` paint is a statement: `fill="none"` is inherited like any other
/// value, and it is why a group with `fill="none"` hides its children's fill.
#[derive(Clone, Debug, Default)]
pub struct Inherited {
    fill: Option<Paint>,
    stroke: Option<Paint>,
}

impl Inherited {
    /// The paint an element states for itself.
    ///
    /// The attribute wins over the style: the attribute is what an exporter
    /// writes last, and what a person edits by hand. The style is read when the
    /// attribute is absent, which is how a Penpot export states a fill.
    pub fn stated(element: &Element, property: &str) -> Option<Paint> {
        let word = match element.attribute(property) {
            Some(word) => word.to_string(),
            None => style::property(element.attribute("style").unwrap_or_default(), property)?,
        };
        Some(Self::parse(&word, property))
    }

    /// The paints the elements inside an element inherit from it.
    pub fn inherit(&self, element: &Element) -> Inherited {
        Inherited {
            fill: Self::stated(element, "fill").or_else(|| self.fill.clone()),
            stroke: Self::stated(element, "stroke").or_else(|| self.stroke.clone()),
        }
    }

    /// The fill a shape is painted with. A shape nobody filled is black.
    pub fn fill_of(&self, element: &Element) -> Paint {
        match Self::stated(element, "fill") {
            Some(paint) => paint,
            None => self.fill.clone().unwrap_or(Paint::Colour(Rgba::BLACK)),
        }
    }

    /// The fill of a shape whose outline is what matters.
    ///
    /// An outline with no fill of its own is an outline, not a shape: SVG would
    /// fill it black, but a stroked path in a panel is a border.
    pub fn outline_fill_of(&self, element: &Element) -> Paint {
        match Self::stated(element, "fill") {
            Some(paint) => paint,
            None => match self.fill.clone() {
                Some(paint) => paint,
                None => match self.stroke_of(element) {
                    Paint::Colour(_) => Paint::Nothing,
                    _ => Paint::Colour(Rgba::BLACK),
                },
            },
        }
    }

    /// The stroke a shape is drawn with. Nothing is stroked unless stated.
    pub fn stroke_of(&self, element: &Element) -> Paint {
        match Self::stated(element, "stroke") {
            Some(paint) => paint,
            None => self.stroke.clone().unwrap_or(Paint::Nothing),
        }
    }

    /// The paint a word names.
    fn parse(word: &str, property: &str) -> Paint {
        let word = word.trim();
        if word == "none" {
            return Paint::Nothing;
        }
        if let Some(reference) = word.strip_prefix("url(").and_then(|rest| rest.strip_suffix(')')) {
            return Paint::Unsupported(format!(
                "the {property} is the paint server `url({reference})`"
            ));
        }
        match colour::parse(word) {
            Some(Colour::Opaque(colour)) => Paint::Colour(colour),
            Some(Colour::Translucent) => Paint::Unsupported(format!(
                "the {property} `{word}` is translucent, and this document format has no alpha"
            )),
            None => Paint::Unsupported(format!(
                "the {property} `{word}` is not a colour this translator understands"
            )),
        }
    }
}
