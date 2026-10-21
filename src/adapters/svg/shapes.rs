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
use super::path;
use super::rectangle;
use super::values;
use super::xml::Element;
use crate::core::geom::{Point, Rect, Rgba};
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

/// A path: a rectangle when it is one, and a path otherwise.
///
/// A filled path is filled by the core with the non-zero winding rule, which
/// is SVG's rule, so a triangle is a shape and not a hole in the panel. A
/// path that is an axis-aligned rectangle is still written as a rectangle,
/// because the rectangle is the cheaper shape to draw — unless it has a
/// stroke of its own, which a rectangle directive cannot carry.
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
    let shape = match path::segments(word, element.line)? {
        path::Parsed::Segments(shape) => shape,
        path::Parsed::Undrawn(reason) => return Ok(Outcome::Undrawn(reason)),
    };
    let shape = path::translated(&shape, at);
    let fill = match flat_colour(fill) {
        Ok(colour) => colour,
        Err(reason) => return Ok(Outcome::Undrawn(reason)),
    };
    let stroke = match flat_colour(stroke) {
        Ok(colour) => colour,
        Err(reason) => return Ok(Outcome::Undrawn(reason)),
    };
    if fill.is_none() && stroke.is_none() {
        return Ok(Outcome::Unpainted);
    }

    if let Some(colour) = fill
        && stroke.is_none()
        && let Some(outline) = path::outline_of(&shape)
        && let Some(area) = rectangle::from_outline(&outline)
    {
        emit::rect(document, area, colour, element.line)?;
        return Ok(Outcome::Drawn);
    }
    emit::path(document, &shape, fill, stroke, element.line)?;
    Ok(Outcome::Drawn)
}

/// The flat colour of a paint, or the reason there is none.
fn flat_colour(paint: &Paint) -> Result<Option<Rgba>, String> {
    match paint {
        Paint::Colour(colour) => Ok(Some(*colour)),
        Paint::Nothing => Ok(None),
        Paint::Unsupported(reason) => Err(reason.clone()),
    }
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
