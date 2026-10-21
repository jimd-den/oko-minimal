//! The state of one translation: open elements, the current offset, and the
//! document being written.
//!
//! The translator is a small stack machine. An element either writes a
//! directive, opens a frame that offsets its children, or is refused. Nothing
//! here reads the clock or the filesystem, so the whole authoring bridge is a
//! pure function from SVG text to document text.

use super::error::SvgError;
use super::xml::{Element, Event};
use super::{paint, shapes, text, values};
use std::mem;

#[derive(Default)]
pub struct Translator {
    document: String,
    stack: Vec<Frame>,
    offset: (i32, i32),
    /// The styles of the open `<text>` and `<tspan>` elements, outermost first.
    styles: Vec<text::Style>,
}

struct Frame {
    name: String,
    offset: (i32, i32),
    line: usize,
}

impl Translator {
    pub fn run(mut self, events: Vec<Event>) -> Result<String, SvgError> {
        for event in events {
            match event {
                Event::Start(element) => self.start(element)?,
                Event::End { name, line } => self.end(&name, line)?,
                Event::Text { content, line } => self.label(&content, line)?,
            }
        }
        if let Some(frame) = self.stack.last() {
            return Err(SvgError::new(
                frame.line,
                format!("`{}` is not closed", frame.name),
            ));
        }
        Ok(mem::take(&mut self.document))
    }

    fn start(&mut self, element: Element) -> Result<(), SvgError> {
        paint::require_opaque(&element)?;
        let local = self.local_offset(&element)?;
        let at = (self.offset.0 + local.0, self.offset.1 + local.1);

        match element.name.as_str() {
            "svg" | "g" => paint::plain_group(&element)?,
            "rect" => shapes::rect(&mut self.document, &element, at)?,
            "line" => shapes::line(&mut self.document, &element, at)?,
            "polyline" => shapes::polyline(&mut self.document, &element, at)?,
            "path" => shapes::path(&mut self.document, &element, at)?,
            "text" => {
                // A self-closing text has no content, and no style to leave behind.
                if !element.closed {
                    self.styles.push(text::Style::of_text(&element, at)?);
                }
            }
            "tspan" => {
                let Some(parent) = self.styles.last() else {
                    return Err(element.error("a `<tspan>` must be inside a `<text>`"));
                };
                if !element.closed {
                    self.styles.push(parent.of_span(&element, local)?);
                }
            }
            other => {
                return Err(element.error(format!(
                    "`{other}` is not supported; use svg, g, rect, line, polyline, path or text"
                )));
            }
        }

        if !element.closed {
            self.stack.push(Frame {
                name: element.name,
                offset: self.offset,
                line: element.line,
            });
            self.offset = at;
        }
        Ok(())
    }

    fn end(&mut self, name: &str, line: usize) -> Result<(), SvgError> {
        let Some(frame) = self.stack.pop() else {
            return Err(SvgError::new(line, format!("unexpected closing `{name}`")));
        };
        if frame.name != name {
            return Err(SvgError::new(
                line,
                format!("`{name}` closes `{}`", frame.name),
            ));
        }
        if matches!(name, "text" | "tspan") {
            self.styles.pop();
        }
        self.offset = frame.offset;
        Ok(())
    }

    /// A run of characters becomes one label, drawn where its element sits.
    fn label(&mut self, content: &str, line: usize) -> Result<(), SvgError> {
        let Some(style) = self.styles.last() else {
            return Err(SvgError::new(
                line,
                "text is only supported inside a `<text>` or a `<tspan>`",
            ));
        };
        let Some(label) = text::label(content, line)? else {
            return Ok(());
        };
        text::emit(&mut self.document, style, &label, line)
    }

    fn local_offset(&self, element: &Element) -> Result<(i32, i32), SvgError> {
        let mut offset = (0, 0);
        if element.name == "svg"
            && let Some(view) = element.attribute("viewBox")
        {
            let numbers = values::numbers(view, element.line)?;
            if numbers.len() != 4 {
                return Err(element.error("viewBox needs four numbers"));
            }
            offset = (-numbers[0], -numbers[1]);
        }
        if let Some(transform) = element.attribute("transform") {
            let (x, y) = values::translate(transform, element.line)?;
            offset = (offset.0 + x, offset.1 + y);
        }
        Ok(offset)
    }
}
