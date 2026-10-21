//! The arithmetic of moving between two states.
//!
//! This module holds the numbers and nothing else: a point, a size, a
//! colour, a path. The rule it follows is linear - a shape travels evenly
//! between its two states, and a colour's channels are rounded to whole
//! numbers, because a panel has no half a pixel of colour.
//!
//! It is separate from `core::shape` because the arithmetic is the part that
//! has to be right, and the part that is easiest to get wrong by adding a
//! case to it.

use super::geom::{Point, Rect, Rgba};
use super::path::{Path, Segment};

/// A point between two points.
pub fn point(one: Point, other: Point, progress: f64) -> Point {
    Point::new(
        number(f64::from(one.x), f64::from(other.x), progress),
        number(f64::from(one.y), f64::from(other.y), progress),
    )
}

/// A rectangle between two rectangles.
pub fn area(one: Rect, other: Rect, progress: f64) -> Rect {
    Rect::new(
        number(f64::from(one.x), f64::from(other.x), progress),
        number(f64::from(one.y), f64::from(other.y), progress),
        size(one.w, other.w, progress),
        size(one.h, other.h, progress),
    )
}

/// A colour between two colours.
pub fn colour(one: Rgba, other: Rgba, progress: f64) -> Rgba {
    Rgba::new(
        channel(one.r, other.r, progress),
        channel(one.g, other.g, progress),
        channel(one.b, other.b, progress),
        channel(one.a, other.a, progress),
    )
}

/// One channel of a colour, rounded to a whole number.
pub fn channel(one: u8, other: u8, progress: f64) -> u8 {
    number(f64::from(one), f64::from(other), progress) as u8
}

/// A size between two sizes.
pub fn size(one: u32, other: u32, progress: f64) -> u32 {
    number(f64::from(one), f64::from(other), progress) as u32
}

/// A scale between two scales. A scale is never below one: a scale of zero
/// would draw no glyph at all.
pub fn scale(one: u8, other: u8, progress: f64) -> u8 {
    number(f64::from(one), f64::from(other), progress).max(1) as u8
}

/// A number between two numbers, rounded to a whole number.
pub fn number(one: f64, other: f64, progress: f64) -> i32 {
    (one + (other - one) * progress).round() as i32
}

/// A paint between two paints. A paint that is absent stays absent.
pub fn paint(one: Option<Rgba>, other: Option<Rgba>, progress: f64) -> Option<Rgba> {
    match (one, other) {
        (Some(one), Some(other)) => Some(colour(one, other, progress)),
        _ => None,
    }
}

/// A path between two paths of the same structure.
pub fn path(one: &Path, other: &Path, progress: f64) -> Path {
    Path {
        segments: one
            .segments
            .iter()
            .zip(other.segments.iter())
            .map(|(one, other)| segment(one, other, progress))
            .collect(),
    }
}

/// One segment between two segments of the same kind.
pub fn segment(one: &Segment, other: &Segment, progress: f64) -> Segment {
    match (one, other) {
        (Segment::Move { to: one }, Segment::Move { to: other }) => Segment::Move {
            to: point(*one, *other, progress),
        },
        (Segment::Line { to: one }, Segment::Line { to: other }) => Segment::Line {
            to: point(*one, *other, progress),
        },
        (
            Segment::Cubic {
                control1: one_first,
                control2: one_second,
                to: one_to,
            },
            Segment::Cubic {
                control1: other_first,
                control2: other_second,
                to: other_to,
            },
        ) => Segment::Cubic {
            control1: point(*one_first, *other_first, progress),
            control2: point(*one_second, *other_second, progress),
            to: point(*one_to, *other_to, progress),
        },
        (
            Segment::Quadratic {
                control: one_control,
                to: one_to,
            },
            Segment::Quadratic {
                control: other_control,
                to: other_to,
            },
        ) => Segment::Quadratic {
            control: point(*one_control, *other_control, progress),
            to: point(*one_to, *other_to, progress),
        },
        (Segment::Close, Segment::Close) => Segment::Close,
        _ => *one,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::path::Segment;

    #[test]
    fn a_point_is_halfway_between_two_points() {
        let halfway = point(Point::new(0, 0), Point::new(10, 20), 0.5);

        assert_eq!(halfway, Point::new(5, 10));
    }

    #[test]
    fn an_area_is_halfway_between_two_areas() {
        let halfway = area(Rect::new(0, 0, 4, 4), Rect::new(10, 20, 8, 12), 0.5);

        assert_eq!(halfway, Rect::new(5, 10, 6, 8));
    }

    #[test]
    fn a_colour_is_halfway_between_two_colours() {
        let halfway = colour(Rgba::rgb(255, 157, 0), Rgba::rgb(255, 0, 0), 0.5);

        assert_eq!(halfway, Rgba::rgb(255, 79, 0));
    }

    #[test]
    fn a_channel_is_rounded_to_the_nearest_whole_number() {
        // 157 and 0 are 78.5 apart, and the rule is the nearest whole number.
        assert_eq!(channel(157, 0, 0.5), 79);
        assert_eq!(channel(0, 157, 0.5), 79);
    }

    #[test]
    fn a_size_is_halfway_between_two_sizes() {
        let halfway = size(4, 12, 0.5);

        assert_eq!(halfway, 8);
    }

    #[test]
    fn a_paint_that_is_absent_stays_absent() {
        assert_eq!(paint(None, Some(Rgba::WHITE), 0.5), None);
        assert_eq!(paint(Some(Rgba::WHITE), None, 0.5), None);
    }

    #[test]
    fn a_path_moves_to_another_path_of_the_same_shape() {
        let one = Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(0, 0),
                },
                Segment::Line {
                    to: Point::new(10, 0),
                },
            ],
        };
        let other = Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(0, 10),
                },
                Segment::Line {
                    to: Point::new(10, 10),
                },
            ],
        };

        let halfway = path(&one, &other, 0.5);

        assert_eq!(halfway.segments[0], Segment::Move { to: Point::new(0, 5) });
        assert_eq!(halfway.segments[1], Segment::Line { to: Point::new(10, 5) });
    }
}
