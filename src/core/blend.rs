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
