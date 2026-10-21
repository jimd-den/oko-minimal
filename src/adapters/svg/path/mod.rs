//! The bounded path grammar: `d` attributes, curves and all.
//!
//! SVG paths can describe anything. A panel is made of shapes, so this
//! module understands exactly the commands a panel can draw: `M`, `L`, `H`,
//! `V`, `C`, `S`, `Q`, `T` and `Z`, absolute and relative, and it produces
//! the segments the core draws. A path that uses anything else is not
//! refused: it is reported as undrawable, with the command named, and the
//! shapes around it still draw.
//!
//! Points are quantised to whole pixels here, because the panel format has
//! no subpixel geometry. That is a property of the format, like the 5x7
//! font, and it is not reported per shape.

mod curve;
#[cfg(test)]
mod curve_tests;
mod grammar;
mod reader;
mod state;
#[cfg(test)]
mod tests;

use super::error::SvgError;
use crate::core::geom::Point;
use crate::core::path::{Path as CorePath, Segment};

pub const MAX_POINTS: usize = 4096;

/// The straight edges of a path, for the rectangle check.
///
/// This is the narrow answer the rectangle check asks for. The path a panel
/// draws is `Parsed`: the segments the core draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Path {
    /// The straight edges of one shape.
    Edges(Outline),
    /// A path this console cannot draw, and why.
    Undrawn(String),
}

/// The straight edges of one shape.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outline {
    pub points: Vec<Point>,
    pub closed: bool,
}

/// A path read out of an export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parsed {
    /// The segments of a path this console draws.
    Segments(CorePath),
    /// A path this console cannot draw, and why.
    Undrawn(String),
}

/// Read a path into the segments the core draws.
pub fn segments(source: &str, line: usize) -> Result<Parsed, SvgError> {
    grammar::read(source, line)
}

/// The straight edges of a path, if it is one shape of straight edges.
///
/// A path with a curve, or with more than one shape, has no straight
/// outline: the rectangle check needs one, and a path that is not a
/// rectangle is drawn as a path.
pub fn outline_of(shape: &CorePath) -> Option<Outline> {
    let mut points: Vec<Point> = Vec::new();
    let mut closed = false;
    for segment in &shape.segments {
        match segment {
            Segment::Move { to } => {
                if !points.is_empty() {
                    return None;
                }
                points.push(*to);
            }
            Segment::Line { to } => points.push(*to),
            Segment::Close => closed = true,
            Segment::Cubic { .. } | Segment::Quadratic { .. } => return None,
        }
    }
    if points.is_empty() {
        return None;
    }
    // A closed path that returns to its start says so twice: the `Z` already
    // closes it, and the last edge is the one the writer meant.
    if closed && points.len() > 1 && points[points.len() - 1] == points[0] {
        points.pop();
    }
    Some(Outline { points, closed })
}

/// Read the straight edges of a path.
///
/// This is the outline the rectangle check asks for. A path with a curve is
/// not one, and says so.
pub fn outline(source: &str, line: usize) -> Result<Path, SvgError> {
    match segments(source, line)? {
        Parsed::Segments(shape) => Ok(match outline_of(&shape) {
            Some(outline) => Path::Edges(outline),
            None => Path::Undrawn("the path is not one shape of straight edges".to_string()),
        }),
        Parsed::Undrawn(reason) => Ok(Path::Undrawn(reason)),
    }
}

/// A path moved by an offset.
///
/// Every point moves: the ends, and the control points of the curves, so a
/// curve is moved whole and not bent.
pub fn translated(shape: &CorePath, at: (i32, i32)) -> CorePath {
    let moved = |point: &Point| Point::new(point.x + at.0, point.y + at.1);
    let segments = shape
        .segments
        .iter()
        .map(|segment| match segment {
            Segment::Move { to } => Segment::Move { to: moved(to) },
            Segment::Line { to } => Segment::Line { to: moved(to) },
            Segment::Cubic {
                control1,
                control2,
                to,
            } => Segment::Cubic {
                control1: moved(control1),
                control2: moved(control2),
                to: moved(to),
            },
            Segment::Quadratic { control, to } => Segment::Quadratic {
                control: moved(control),
                to: moved(to),
            },
            Segment::Close => Segment::Close,
        })
        .collect();
    CorePath { segments }
}
