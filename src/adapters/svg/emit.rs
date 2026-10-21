//! Writing document directives, with the format's limits enforced.
//!
//! The translator must not write a document its own parser will reject. The
//! checks here are the document format's checks, shared through
//! `core::syntax`, so the two never drift apart.

use super::error::SvgError;
use crate::core::geom::{Point, Rect, Rgba};
use crate::core::syntax::{MAX_COORDINATE, MAX_SIDE};

pub fn rect(document: &mut String, area: Rect, colour: Rgba, line: usize) -> Result<(), SvgError> {
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
    document.push_str(&format!(
        "rect {} {} {} {} {}\n",
        area.x,
        area.y,
        area.w,
        area.h,
        hex(colour)
    ));
    Ok(())
}

pub fn line(
    document: &mut String,
    from: Point,
    to: Point,
    colour: Rgba,
    line: usize,
) -> Result<(), SvgError> {
    coordinate(from.x, line)?;
    coordinate(from.y, line)?;
    coordinate(to.x, line)?;
    coordinate(to.y, line)?;
    document.push_str(&format!(
        "line {} {} {} {} {}\n",
        from.x,
        from.y,
        to.x,
        to.y,
        hex(colour)
    ));
    Ok(())
}

fn coordinate(value: i32, line: usize) -> Result<(), SvgError> {
    if i64::from(value).abs() > MAX_COORDINATE {
        return Err(SvgError::new(
            line,
            format!("coordinates must be within ±{MAX_COORDINATE}"),
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
        let mut document = String::new();

        rect(
            &mut document,
            Rect::new(2, 3, 4, 5),
            Rgba::rgb(255, 157, 0),
            1,
        )
        .expect("a rectangle");

        assert_eq!(document, "rect 2 3 4 5 #ff9d00\n");
    }

    #[test]
    fn a_coordinate_the_format_would_reject_is_refused() {
        let mut document = String::new();

        let error = rect(&mut document, Rect::new(40_000, 0, 1, 1), Rgba::BLACK, 1)
            .expect_err("a coordinate out of range");

        assert!(error.problem.contains("32767"));
    }
}
