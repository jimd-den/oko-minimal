//! Writing document directives, with the format's limits enforced.
//!
//! The translator must not write a document its own parser will reject. The
//! checks here are the document format's checks, shared through
//! `core::syntax`, so the two never drift apart.
//!
//! The writer also counts the operations it writes, because the format caps
//! how many a document may hold. The count is of operations, not of bytes or
//! of elements: a polyline of five points writes four `line` directives and
//! counts four.

use super::error::SvgError;
use crate::core::document::MAX_OPS;
use crate::core::geom::{Point, Rect, Rgba};
use crate::core::syntax::{MAX_COORDINATE, MAX_SIDE};

/// The document being written, and how many operations it holds.
///
/// Contract: every operation written is counted, and the write that would
/// pass the format's budget is refused with the line of the element that
/// caused it.
#[derive(Default)]
pub struct Writer {
    text: String,
    ops: usize,
}

impl Writer {
    /// Write one operation, refusing when the budget is spent.
    pub fn write(&mut self, operation: &str, line: usize) -> Result<(), SvgError> {
        if self.ops >= MAX_OPS {
            return Err(SvgError::new(
                line,
                format!("a document may hold at most {MAX_OPS} operations"),
            ));
        }
        self.ops += 1;
        self.text.push_str(operation);
        Ok(())
    }

    /// The document text, once every operation has been written.
    pub fn into_text(self) -> String {
        self.text
    }
}

pub fn rect(document: &mut Writer, area: Rect, colour: Rgba, line: usize) -> Result<(), SvgError> {
    if area.w == 0 || area.h == 0 {
        return Ok(());
    }
    coordinate(area.x, line)?;
    coordinate(area.y, line)?;
    if i64::from(area.w) > MAX_SIDE || i64::from(area.h) > MAX_SIDE {
        return Err(SvgError::new(
            line,
            format!("sizes above {MAX_SIDE} are not supported"),
        ));
    }
    document.write(
        &format!(
            "rect {} {} {} {} {}\n",
            area.x,
            area.y,
            area.w,
            area.h,
            hex(colour)
        ),
        line,
    )
}

pub fn line(
    document: &mut Writer,
    from: Point,
    to: Point,
    colour: Rgba,
    line: usize,
) -> Result<(), SvgError> {
    coordinate(from.x, line)?;
    coordinate(from.y, line)?;
    coordinate(to.x, line)?;
    coordinate(to.y, line)?;
    document.write(
        &format!(
            "line {} {} {} {} {}\n",
            from.x,
            from.y,
            to.x,
            to.y,
            hex(colour)
        ),
        line,
    )
}

fn coordinate(value: i32, line: usize) -> Result<(), SvgError> {
    if i64::from(value).abs() > MAX_COORDINATE {
        return Err(SvgError::new(
            line,
            format!("coordinates must be within \u{b1}{MAX_COORDINATE}"),
        ));
    }
    Ok(())
}

fn hex(colour: Rgba) -> String {
    format!("#{:02x}{:02x}{:02x}", colour.r, colour.g, colour.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_becomes_one_document_line() {
        let mut document = Writer::default();

        rect(
            &mut document,
            Rect::new(2, 3, 4, 5),
            Rgba::rgb(255, 157, 0),
            1,
        )
        .expect("a rectangle");

        assert_eq!(document.into_text(), "rect 2 3 4 5 #ff9d00\n");
    }

    #[test]
    fn a_coordinate_the_format_would_reject_is_refused() {
        let mut document = Writer::default();

        let error = rect(&mut document, Rect::new(40_000, 0, 1, 1), Rgba::BLACK, 1)
            .expect_err("a coordinate out of range");

        assert!(error.problem.contains("32767"));
    }

    #[test]
    fn the_write_that_passes_the_budget_is_refused() {
        let mut document = Writer::default();

        for _ in 0..MAX_OPS {
            document.write("clear #000000\n", 1).expect("an operation");
        }
        let error = document
            .write("clear #000000\n", 9)
            .expect_err("an operation over the budget");

        assert_eq!(error.line, 9);
        assert!(error.problem.contains(&MAX_OPS.to_string()));
    }
}
