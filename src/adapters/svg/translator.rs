//! Turning an export's tree into a document and a report of what was left out.
//!
//! The translator walks the tree the scanner built. It draws what it can and
//! records what it cannot, with the line the shape was on, so a person reading
//! the report can find it in the file they exported.

use super::elements;
use super::emit::Writer;
use super::error::SvgError;
use super::outcome::Outcome;
use super::paint::Inherited;
use super::shapes;
use super::transform::{self, Transform};
use super::values;
use super::xml::{Element, Node};
use super::{Skipped, Translation};

/// What an element hands to the elements inside it.
pub(super) struct Scope {
    /// Where the origin of this element sits, after the transforms above it.
    pub(super) offset: (i32, i32),
    /// The paints the ancestors stated.
    pub(super) paint: Inherited,
    /// The style of the text this element is inside, if any.
    pub(super) style: Option<super::text::Style>,
    /// The colour of the text this element is inside, if any.
    pub(super) colour: Option<crate::core::geom::Rgba>,
}

impl Scope {
    /// A scope with no text in it.
    pub(super) fn plain(&self) -> Scope {
        Scope {
            offset: self.offset,
            paint: self.paint.clone(),
            style: None,
            colour: None,
        }
    }
}

/// A translator walks an export and writes a document.
#[derive(Default)]
pub struct Translator {
    pub(super) document: Writer,
    pub(super) skipped: Vec<Skipped>,
}

impl Translator {
    /// Translate an export into a document and a report.
    pub fn run(mut self, root: &Element) -> Result<Translation, SvgError> {
        let scope = Scope {
            offset: (0, 0),
            paint: Inherited::default(),
            style: None,
            colour: None,
        };
        self.element(root, &scope)?;
        Ok(Translation {
            document: self.document.into_text(),
            skipped: self.skipped,
        })
    }

    /// Draw one element and everything inside it.
    ///
    /// The whitelist lives in `elements`: an element this console does not
    /// draw is named in the report rather than refused.
    fn element(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        let name = element.name.as_str();
        if elements::is_structural(name) {
            return Ok(());
        }
        if !elements::is_drawn(name) {
            self.skip(
                element.line,
                format!("`<{name}>` is not an element this console draws"),
            );
            return Ok(());
        }
        match name {
            "svg" | "g" => self.group(element, scope),
            "text" => self.text(element, scope),
            "tspan" => self.span(element, scope),
            _ => self.shape(element, scope),
        }
    }

    /// A group: the paint and the offset it hands down.
    fn group(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        let Some(at) = self.at(element, scope)? else {
            return Ok(());
        };
        self.unapplied(element);
        let mut inner = scope.plain();
        inner.offset = at;
        inner.paint = scope.paint.inherit(element);
        self.children(element, &inner)
    }

    /// One shape: a rectangle, a line, a polyline or a path.
    fn shape(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        let Some(at) = self.at(element, scope)? else {
            return Ok(());
        };
        let fill = scope.paint.outline_fill_of(element);
        let stroke = scope.paint.stroke_of(element);
        let outcome = match element.name.as_str() {
            "rect" => shapes::rect(&mut self.document, element, at, &fill)?,
            "line" => shapes::line(&mut self.document, element, at, &stroke)?,
            "polyline" => shapes::polyline(&mut self.document, element, at, &fill, &stroke)?,
            _ => shapes::path(&mut self.document, element, at, &fill, &stroke)?,
        };
        if let Outcome::Undrawn(reason) = outcome {
            self.skip(element.line, reason);
        }
        Ok(())
    }

    /// The children of an element.
    pub(super) fn children(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        for child in &element.children {
            match child {
                Node::Element(child) => self.element(child, scope)?,
                Node::Text(run) => self.run_text(run, scope)?,
            }
        }
        Ok(())
    }

    /// Where an element's own transform puts its origin.
    pub(super) fn at(
        &mut self,
        element: &Element,
        scope: &Scope,
    ) -> Result<Option<(i32, i32)>, SvgError> {
        let Some(shift) = self.shift(element)? else {
            return Ok(None);
        };
        Ok(Some((scope.offset.0 + shift.0, scope.offset.1 + shift.1)))
    }

    /// The shift an element's own transform makes, if it makes one we can draw.
    pub(super) fn shift(&mut self, element: &Element) -> Result<Option<(i32, i32)>, SvgError> {
        let mut shift = (0, 0);
        if element.name == "svg"
            && let Some(view) = element.attribute("viewBox")
        {
            let numbers = values::numbers(view, element.line)?;
            if numbers.len() != 4 {
                return Err(element.error("a viewBox needs four numbers"));
            }
            shift = (-numbers[0], -numbers[1]);
        }
        let Some(word) = element.attribute("transform") else {
            return Ok(Some(shift));
        };
        match transform::transform(word) {
            Transform::Shift(x, y) => Ok(Some((shift.0 + x, shift.1 + y))),
            Transform::Undrawn(reason) => {
                self.skip(element.line, reason);
                Ok(None)
            }
        }
    }

    /// The references an element carries that this console does not apply.
    fn unapplied(&mut self, element: &Element) {
        for name in ["clip-path", "mask", "filter"] {
            if let Some(word) = element.attribute(name) {
                self.skip(
                    element.line,
                    format!(
                        "the {name} `{word}` is not applied, so this element may show more than the source drew"
                    ),
                );
            }
        }
    }

    /// Record that a shape was left out, and why.
    pub(super) fn skip(&mut self, line: usize, reason: String) {
        self.skipped.push(Skipped { line, reason });
    }
}
