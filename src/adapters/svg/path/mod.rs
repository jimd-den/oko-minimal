//! The bounded path grammar: `d` attributes without curves.
//!
//! SVG paths can describe anything. A panel is made of straight edges, so this
//! module understands exactly `M`, `L`, `H`, `V` and `Z`, absolute and
//! relative. A path that uses anything else is not refused: it is reported as
//! undrawable, with the command named, and the shapes around it still draw.
//!
//! One subpath is one outline. Points are quantised to whole pixels here,
//! because the panel format has no subpixel geometry.

mod reader;
#[cfg(test)]
mod tests;

use super::error::SvgError;
use crate::core::geom::Point;
use reader::Reader;

pub const MAX_POINTS: usize = 4096;

/// The largest coordinate a path may name.
const MAX_COORDINATE: f64 = 1_000_000.0;

/// What a path turned out to be.
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

/// Read the edges of a path.
pub fn outline(source: &str, line: usize) -> Result<Path, SvgError> {
    let mut reader = Reader::new(source, line);
    let mut points: Vec<Point> = Vec::new();
    let mut current = (0.0, 0.0);
    let mut origin = (0.0, 0.0);
    let mut command = None;
    let mut closed = false;

    while reader.peek().is_some() {
        reader.separators();
        if reader.peek().is_none() {
            break;
        }

        if let Some(byte) = reader.command() {
            if !matches!(
                byte,
                b'M' | b'm' | b'L' | b'l' | b'H' | b'h' | b'V' | b'v' | b'Z' | b'z'
            ) {
                return Ok(Path::Undrawn(format!(
                    "the path uses `{letter}`, which this console does not draw",
                    letter = byte as char
                )));
            }
            command = Some(byte);
        } else if command.is_none() {
            return Err(SvgError::new(line, "a path must start with a command"));
        }

        let step = command.expect("a command was read above");
        match step {
            b'M' | b'm' => {
                if !points.is_empty() {
                    return Ok(Path::Undrawn(
                        "the path has more than one shape, which this console does not draw"
                            .to_string(),
                    ));
                }
                let (x, y) = reader.pair()?;
                current = if step == b'm' {
                    (current.0 + x, current.1 + y)
                } else {
                    (x, y)
                };
                origin = current;
                push(&mut points, current, line)?;
                command = Some(if step == b'm' { b'l' } else { b'L' });
            }
            b'L' | b'l' => {
                let (x, y) = reader.pair()?;
                current = if step == b'l' {
                    (current.0 + x, current.1 + y)
                } else {
                    (x, y)
                };
                push(&mut points, current, line)?;
            }
            b'H' | b'h' => {
                let x = reader.number()?;
                current.0 = if step == b'h' { current.0 + x } else { x };
                push(&mut points, current, line)?;
            }
            b'V' | b'v' => {
                let y = reader.number()?;
                current.1 = if step == b'v' { current.1 + y } else { y };
                push(&mut points, current, line)?;
            }
            b'Z' | b'z' => {
                closed = true;
                current = origin;
                command = None;
            }
            _ => unreachable!("the command was checked above"),
        }
    }

    if points.is_empty() {
        return Err(SvgError::new(line, "a path needs at least one point"));
    }
    // A closed path that returns to its start says so twice: the `Z` already
    // closes it, and the last edge is the one the writer meant.
    if closed && points.len() > 1 && points[points.len() - 1] == points[0] {
        points.pop();
    }
    Ok(Path::Edges(Outline { points, closed }))
}

fn push(points: &mut Vec<Point>, (x, y): (f64, f64), line: usize) -> Result<(), SvgError> {
    if points.len() >= MAX_POINTS {
        return Err(SvgError::new(line, "a path has too many points"));
    }
    points.push(Point::new(round(x, line)?, round(y, line)?));
    Ok(())
}

fn round(value: f64, line: usize) -> Result<i32, SvgError> {
    if !value.is_finite() || value.abs() > MAX_COORDINATE {
        return Err(SvgError::new(
            line,
            "a path coordinate is outside the drawable range",
        ));
    }
    Ok(value.round() as i32)
}
