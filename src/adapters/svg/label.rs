//! Drawing the text of an export.
//!
//! A `<text>` element holds runs of characters, and a `<tspan>` inside it holds
//! more. The translator walks that nesting itself, because a label's place and
//! size come from the text around it: a span keeps what it does not say, and
//! the colour comes from the fill in force where the text sits.

use super::error::SvgError;
use super::paint::Paint;
use super::scope::Scope;
use super::text;
use super::translator::Translator;
use super::xml::{Element, Text};

impl Translator {
    /// A text element: the style and the colour its runs are drawn with.
    pub(super) fn text(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        let Some(at) = self.at(element, scope)? else {
            return Ok(());
        };
        let colour = match scope.paint.fill_of(element) {
            Paint::Colour(colour) => Some(colour),
            Paint::Nothing => None,
            Paint::Unsupported(reason) => {
                self.skip(element.line, reason);
                return Ok(());
            }
        };
        let style = text::Style::of_text(element, at)?;
        let mut inner = scope.plain();
        inner.offset = at;
        inner.paint = scope.paint.inherit(element);
        inner.style = Some(style);
        inner.colour = colour;
        self.children(element, &inner)
    }

    /// A span inside a text: the style it takes from the text.
    pub(super) fn span(&mut self, element: &Element, scope: &Scope) -> Result<(), SvgError> {
        let Some(parent) = scope.style.clone() else {
            return Err(element.error("a `<tspan>` must be inside a `<text>`"));
        };
        let Some(shift) = self.shift(element)? else {
            return Ok(());
        };
        let style = parent.of_span(element, shift)?;
        let colour = match scope.paint.fill_of(element) {
            Paint::Colour(colour) => Some(colour),
            Paint::Nothing => None,
            Paint::Unsupported(reason) => {
                self.skip(element.line, reason);
                return Ok(());
            }
        };
        let mut inner = scope.plain();
        inner.offset = (scope.offset.0 + shift.0, scope.offset.1 + shift.1);
        inner.paint = scope.paint.inherit(element);
        inner.style = Some(style);
        inner.colour = colour;
        self.children(element, &inner)
    }

    /// One run of text: a label in the document.
    pub(super) fn run_text(&mut self, run: &Text, scope: &Scope) -> Result<(), SvgError> {
        let Some(style) = scope.style.clone() else {
            return Err(SvgError::new(
                run.line,
                "text is only supported inside a `<text>` or a `<tspan>`",
            ));
        };
        let Some(colour) = scope.colour else {
            return Ok(());
        };
        match text::label(&run.content, run.line)? {
            None => Ok(()),
            Some(text::Label::Text(label)) => {
                if let Some(note) =
                    text::emit(&mut self.document, &style, &label, colour, run.line)?
                {
                    self.skip(run.line, note);
                }
                Ok(())
            }
            Some(text::Label::Undrawable(reason)) => {
                self.skip(run.line, reason);
                Ok(())
            }
        }
    }
}
