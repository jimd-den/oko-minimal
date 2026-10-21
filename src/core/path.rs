//! A path in the display list: the segments of a shape, and the sampling
//! that turns those segments into a polyline.
//!
//! The document keeps the curve. A cubic stays a cubic in the text, in the
//! scene, and here; only the renderer asks for a polyline. Sampling is
//! therefore a property of this module and not of the document: the document
//! never says `C` and means `line`.
//!
//! Nothing here draws. `sample` answers one question — where are the points
//! of this path — and the renderer decides what to do with them.

use super::curve::{Sample, evaluate};
use super::geom::Point;

/// One step of a path.
///
/// A move starts a shape, the rest draw into it, and a close ends it. The
/// names are the SVG ones, so a document and an export say the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Segment {
    /// Start a new shape at a point.
    Move { to: Point },
    /// A straight edge.
    Line { to: Point },
    /// A cubic curve: two control points and an end.
    Cubic {
        control1: Point,
        control2: Point,
        to: Point,
    },
    /// A quadratic curve: one control point and an end.
    Quadratic { control: Point, to: Point },
    /// Close the current shape back to where it started.
    Close,
}

/// A path: an ordered list of segments.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<Segment>,
}

/// A sampled path: one polyline per shape.
///
/// A closed shape repeats its first point at the end, so the closing edge is
/// drawn by the ordinary rule and an open shape is not closed by accident.
pub type Polyline = Vec<Vec<Sample>>;

/// Sample a path into the polyline the renderer draws.
///
/// Every curve is sampled in proportion to its size: one step per two pixels
/// of the control polygon, at least four steps and at most sixty-four. The
/// control polygon is used because its length bounds the curve's length from
/// above, and measuring the curve itself would cost more than the extra
/// steps are worth.
pub fn sample(path: &Path) -> Polyline {
    let mut shapes: Polyline = Vec::new();
    let mut current = Sample { x: 0.0, y: 0.0 };
    let mut open = false;

    for segment in &path.segments {
        match segment {
            Segment::Move { to } => {
                current = at(*to);
                shapes.push(vec![current]);
                open = true;
            }
            Segment::Line { to } => {
                let to = at(*to);
                current = push(&mut shapes, current, to, &mut open);
            }
            Segment::Cubic {
                control1,
                control2,
                to,
            } => {
                let controls = [current, at(*control1), at(*control2), at(*to)];
                current = curve(&mut shapes, &controls, &mut open);
            }
            Segment::Quadratic { control, to } => {
                let controls = [current, at(*control), at(*to)];
                current = curve(&mut shapes, &controls, &mut open);
            }
            Segment::Close => {
                if let Some(shape) = shapes.last_mut()
                    && let Some(start) = shape.first().copied()
                {
                    shape.push(start);
                    current = start;
                }
                open = false;
            }
        }
    }
    shapes
}

/// A point of a path, in fractions of a pixel.
fn at(point: Point) -> Sample {
    Sample {
        x: point.x as f64,
        y: point.y as f64,
    }
}

/// Add a point to the shape being drawn, starting a new shape when the last
/// one was closed: a path that continues after a close begins again where
/// the closed shape began.
fn push(shapes: &mut Polyline, current: Sample, point: Sample, open: &mut bool) -> Sample {
    if !*open {
        shapes.push(vec![current]);
        *open = true;
    }
    if let Some(shape) = shapes.last_mut() {
        shape.push(point);
    }
    point
}

/// Sample a curve given as its control polygon, from the start to the end.
fn curve(shapes: &mut Polyline, controls: &[Sample], open: &mut bool) -> Sample {
    let polygon: f64 = controls
        .windows(2)
        .map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y))
        .sum();
    let steps = (polygon / 2.0).ceil().clamp(4.0, 64.0) as usize;
    let mut current = controls[0];
    for step in 1..=steps {
        let t = step as f64 / steps as f64;
        let point = evaluate(controls, t);
        current = push(shapes, current, point, open);
    }
    current
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::curve::Sample;

    /// An arc of a given size: a cubic from the origin over to the right.
    fn arc(size: i32) -> Path {
        Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(0, 0),
                },
                Segment::Cubic {
                    control1: Point::new(0, size),
                    control2: Point::new(size, size),
                    to: Point::new(size, 0),
                },
            ],
        }
    }

    fn points_of(path: &Path) -> Vec<Sample> {
        sample(path).first().cloned().unwrap_or_default()
    }

    #[test]
    fn a_cubic_bulges_past_its_chord() {
        let points = points_of(&arc(20));

        assert!(!points.is_empty(), "a cubic is sampled into points");
        assert_eq!(points.first().map(|point| point.y), Some(0.0));
        assert_eq!(points.last().map(|point| point.y), Some(0.0));
        assert!(
            points.iter().any(|point| point.y > 1.0),
            "the curve leaves its chord"
        );
    }

    #[test]
    fn a_large_curve_is_smoother_than_a_small_one() {
        let small = points_of(&arc(4)).len();
        let large = points_of(&arc(40)).len();

        assert!(large > small, "a large curve is sampled more finely");
    }

    #[test]
    fn a_line_is_sampled_as_its_two_ends() {
        let path = Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(1, 1),
                },
                Segment::Line {
                    to: Point::new(4, 1),
                },
            ],
        };

        assert_eq!(points_of(&path).len(), 2);
    }
}
