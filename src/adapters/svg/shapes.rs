//! Turning one shape element into document directives.
//!
//! Every function here takes the element and the offset its ancestors impose,
//! and writes zero or more directives. Nothing is stateful: the stack machine
//! in `translator` decides which shape is next, and this module decides what a
//! rectangle, a line, a polyline or a path paints.

use super::emit::Writer;
use super::error::SvgError;
use super::xml::Element;
use super::{emit, paint, path, rectangle, values};
use crate::core::geom::{Point, Rect};
use crate::core::syntax::MAX_SIDE;

pub fn rect(document: &mut Writer, element: &Element, at: (i32, i32)) -> Result<(), SvgError> {
    let width = size(element, "width")?;
    let height = size(element, "height")?;
    let Some(colour) = paint::fill(element)? else {
        return Ok(());
    };
    let area = Rect::new(
        integer(element, "x", 0)? + at.0,
        integer(element, "y", 0)? + at.1,
        width,
        height,
    );
    emit::rect(document, area, colour, element.line)
}

pub fn line(document: &mut Writer, element: &Element, at: (i32, i32)) -> Result<(), SvgError> {
    let Some(colour) = paint::stroke(element)? else {
        return Ok(());
    };
    let from = Point::new(
        integer(element, "x1", 0)? + at.0,
        integer(element, "y1", 0)? + at.1,
    );
    let to = Point::new(
        integer(element, "x2", 0)? + at.0,
        integer(element, "y2", 0)? + at.1,
    );
    emit::line(document, from, to, colour, element.line)
}

pub fn polyline(document: &mut Writer, element: &Element, at: (i32, i32)) -> Result<(), SvgError> {
    let Some(word) = element.attribute("points") else {
        return Err(element.error("a polyline needs points"));
    };
    let numbers = values::numbers(word, element.line)?;
    if numbers.len() < 4 || numbers.len() % 2 != 0 {
        return Err(element.error("a polyline needs at least two points"));
    }
    if paint::outline_fill(element)?.is_some() {
        return Err(
            element.error("a filled polyline is not supported; use fill=\"none\" and a stroke")
        );
    }
    let Some(colour) = paint::stroke(element)? else {
        return Ok(());
    };
    let points: Vec<Point> = numbers
        .chunks(2)
        .map(|pair| Point::new(pair[0] + at.0, pair[1] + at.1))
        .collect();
    for pair in points.windows(2) {
        emit::line(document, pair[0], pair[1], colour, element.line)?;
    }
    Ok(())
}

pub fn path(document: &mut Writer, element: &Element, at: (i32, i32)) -> Result<(), SvgError> {
    let Some(word) = element.attribute("d") else {
        return Err(element.error("a path needs a `d` attribute"));
    };
    let outline = path::outline(word, element.line)?;
    let points: Vec<Point> = outline
        .points
        .iter()
        .map(|point| Point::new(point.x + at.0, point.y + at.1))
        .collect();
    let outline = path::Outline {
        points,
        closed: outline.closed,
    };

    if let Some(colour) = paint::outline_fill(element)? {
        let Some(area) = rectangle::from_outline(&outline) else {
            return Err(element.error(
                "a filled path must be an axis-aligned rectangle; use fill=\"none\" for strokes",
            ));
        };
        emit::rect(document, area, colour, element.line)?;
    }
    if let Some(colour) = paint::stroke(element)? {
        for pair in outline.points.windows(2) {
            emit::line(document, pair[0], pair[1], colour, element.line)?;
        }
        if outline.closed {
            let last = outline.points[outline.points.len() - 1];
            emit::line(document, last, outline.points[0], colour, element.line)?;
        }
    }
    Ok(())
}

fn integer(element: &Element, name: &str, fallback: i32) -> Result<i32, SvgError> {
    match element.attribute(name) {
        Some(word) => values::integer(word, element.line),
        None => Ok(fallback),
    }
}

fn size(element: &Element, name: &str) -> Result<u32, SvgError> {
    let value = integer(element, name, 0)?;
    if value <= 0 {
        return Ok(0);
    }
    if i64::from(value) > MAX_SIDE {
        return Err(element.error(format!("sizes above {MAX_SIDE} are not supported")));
    }
    Ok(value as u32)
}
