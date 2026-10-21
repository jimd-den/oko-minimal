//! The bounded path grammar: `d` attributes without curves.
//!
//! SVG paths can describe anything. A panel is made of straight edges, so this
//! module understands exactly `M`, `L`, `H`, `V` and `Z`, absolute and relative,
//! and refuses every curve by name. A refusal with a line number is a design
//! constraint the author can see; a guessed curve is a pixel nobody can explain.
//!
//! One subpath is one outline. Points are quantised to whole pixels here,
//! because the panel format has no subpixel geometry.

mod reader;

use super::error::SvgError;
use crate::core::geom::Point;
use reader::Reader;

pub const MAX_POINTS: usize = 4096;
const MAX_COORDINATE: f64 = 1_000_000.0;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outline {
    pub points: Vec<Point>,
    pub closed: bool,
}

pub fn outline(source: &str, line: usize) -> Result<Outline, SvgError> {
    let mut reader = Reader::new(source, line);
    let mut points = Vec::new();
    let mut current = (0.0, 0.0);
    let mut origin = (0.0, 0.0);
    let mut command = None;
    let mut closed = false;
    let mut started = false;

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
                return Err(SvgError::new(
                    line,
                    format!(
                        "path command `{}` is not supported; use M, L, H, V, Z",
                        byte as char
                    ),
                ));
            }
            command = Some(byte);
        } else if command.is_none() {
            return Err(SvgError::new(line, "a path must start with a command"));
        }

        let Some(step) = command else {
            return Err(SvgError::new(line, "a path must start with a command"));
        };
        match step {
            b'M' | b'm' => {
                if started {
                    return Err(SvgError::new(
                        line,
                        "multiple subpaths are not supported; use one path per shape",
                    ));
                }
                started = true;
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
    Ok(Outline { points, closed })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_commands_become_points() {
        let outline = outline("M2 2 H8 V6 H2 Z", 1).expect("a rectangle");

        assert_eq!(
            outline.points,
            [
                Point::new(2, 2),
                Point::new(8, 2),
                Point::new(8, 6),
                Point::new(2, 6),
            ]
        );
        assert!(outline.closed);
    }

    #[test]
    fn relative_commands_accumulate() {
        let outline = outline("m 2 2 l 6 0 v 4 h -6 z", 1).expect("a rectangle");

        assert_eq!(outline.points[3], Point::new(2, 6));
    }

    #[test]
    fn a_curve_is_refused_by_name() {
        let error = outline("M0 0 C 1 1 2 2 3 3", 4).expect_err("a curve");

        assert_eq!(error.line, 4);
        assert!(error.problem.contains("`C`"));
    }
}
