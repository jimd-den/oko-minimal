//! Turning one shape element into document directives.
//!
//! Every function here takes the element, where it sits, and the paints in
//! force, and writes zero or more directives. Nothing is stateful: the
//! translator decides which shape is next, and this module decides what a
//! rectangle, a line, a polyline or a path paints.

use super::emit::{self, Writer};
use super::error::SvgError;
use super::outcome::Outcome;
use super::paint::Paint;
use super::path::{self, Path};
use super::rectangle;
use super::values;
use super::xml::Element;
use crate::core::geom::{Point, Rect};
use crate::core::syntax::MAX_SIDE;

/// A rectangle, painted with its fill.
pub fn rect(
    document: &mut Writer,
    element: &Element,
    at: (i32, i32),
    fill: &Paint,
) -> Result<Outcome, SvgError> {
    if element.attribute("stroke").is_some() {
        return Ok(Outcome::Undrawn(
            "the stroke of this rectangle is not drawn".to_string(),
        ));
    }
    let colour = match fill {
        Paint::Colour(colour) => *colour,
        Paint::Nothing => return Ok(Outcome::Unpainted),
        Paint::Unsupported(reason) => return Ok(Outcome::Undrawn(reason.clone())),
    };
    let area = Rect::new(
        integer(element, "x", 0)? + at.0,
        integer(element, "y", 0)? + at.1,
        size(element, "width")?,
        size(element, "height")?,
    );
    emit::rect(document, area, colour, element.line)?;
    Ok(Outcome::Drawn)
}

/// A line, stroked.
pub fn line(
    document: &mut Writer,
    element: &Element,
    at: (i32, i32),
    stroke: &Paint,
) -> Result<Outcome, SvgError> {
    let colour = match stroke {
        Paint::Colour(colour) => *colour,
        Paint::Nothing => return Ok(Outcome::Unpainted),
        Paint::Unsupported(reason) => return Ok(Outcome::Undrawn(reason.clone())),
    };
    let from = Point::new(
        integer(element, "x1", 0)? + at.0,
        integer(element, "y1", 0)? + at.1,
    );
    let to = Point::new(
        integer(element, "x2", 0)? + at.0,
        integer(element, "y2", 0)? + at.1,
    );
    emit::line(document, from, to, colour, element.line)?;
    Ok(Outcome::Drawn)
}

/// A polyline, stroked. A polyline has no interior, so a fill is not drawn.
pub fn polyline(
    document: &mut Writer,
    element: &Element,
    at: (i32, i32),
    fill: &Paint,
    stroke: &Paint,
) -> Result<Outcome, SvgError> {
    if matches!(fill, Paint::Colour(_)) {
        return Ok(Outcome::Undrawn(
            "the fill of this polyline is not drawn".to_string(),
        ));
    }
    let colour = match stroke {
        Paint::Colour(colour) => *colour,
        Paint::Nothing => return Ok(Outcome::Unpainted),
        Paint::Unsupported(reason) => return Ok(Outcome::Undrawn(reason.clone())),
    };
    let Some(word) = element.attribute("points") else {
        return Err(element.error("a polyline needs points"));
    };
    let numbers = values::numbers(word, element.line)?;
    if numbers.len() < 4 || numbers.len() % 2 != 0 {
        return Err(element.error("a polyline needs at least two points"));
    }
    let points: Vec<Point> = numbers
        .chunks(2)
        .map(|pair| Point::new(pair[0] + at.0, pair[1] + at.1))
        .collect();
    for pair in points.windows(2) {
        emit::line(document, pair[0], pair[1], colour, element.line)?;
    }
    Ok(Outcome::Drawn)
}

/// A path: filled as a rectangle when it is one, stroked as an outline.
pub fn path(
    document: &mut Writer,
    element: &Element,
    at: (i32, i32),
    fill: &Paint,
    stroke: &Paint,
) -> Result<Outcome, SvgError> {
    let Some(word) = element.attribute("d") else {
        return Err(element.error("a path needs a `d` attribute"));
    };
    let outline = match path::outline(word, element.line)? {
        Path::Edges(outline) => outline,
        Path::Undrawn(reason) => return Ok(Outcome::Undrawn(reason)),
    };
    let points: Vec<Point> = outline
        .points
        .iter()
        .map(|point| Point::new(point.x + at.0, point.y + at.1))
        .collect();

    if let Paint::Colour(colour) = fill {
        let closed = path::Outline {
            points: points.clone(),
            closed: outline.closed,
        };
        match rectangle::from_outline(&closed) {
            Some(area) => emit::rect(document, area, *colour, element.line)?,
            None => {
                return Ok(Outcome::Undrawn(
                    "the fill of this path is not an axis-aligned rectangle".to_string(),
                ))
            }
        }
    } else if let Paint::Unsupported(reason) = fill {
        return Ok(Outcome::Undrawn(reason.clone()));
    }

    if let Paint::Colour(colour) = stroke {
        for pair in points.windows(2) {
            emit::line(document, pair[0], pair[1], *colour, element.line)?;
        }
        if outline.closed && points.len() > 1 {
            let last = points[points.len() - 1];
            emit::line(document, last, points[0], *colour, element.line)?;
        }
    } else if let Paint::Unsupported(reason) = stroke {
        return Ok(Outcome::Undrawn(reason.clone()));
    }
    Ok(Outcome::Drawn)
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
