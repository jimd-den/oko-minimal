//! The shape between two shapes.
//!
//! The interpolation is linear: a shape travels evenly between its two
//! states, and a colour's channels are rounded to whole numbers, because a
//! panel has no half a pixel of colour. An easing curve is a later decision;
//! the shape of that decision is already here, because a leg would carry one.
//!
//! This module knows nothing about time. It is asked for the shape between
//! two states and a progress, and it answers.

use super::blend;
use super::shape::Shape;

/// The shape at `progress` of the way from `from` to `to`.
///
/// A pair that may not be blended gives the shape it started from: the
/// refusal has already been made, and this function is not where it is made.
pub fn between(from: &Shape, to: &Shape, progress: f64) -> Shape {
    match (from, to) {
        (Shape::Clear(one), Shape::Clear(other)) => {
            Shape::Clear(blend::colour(*one, *other, progress))
        }
        (
            Shape::Rect {
                area: one_area,
                color: one_color,
            },
            Shape::Rect {
                area: other_area,
                color: other_color,
            },
        ) => Shape::Rect {
            area: blend::area(*one_area, *other_area, progress),
            color: blend::colour(*one_color, *other_color, progress),
        },
        (
            Shape::Line {
                from: one_from,
                to: one_to,
                color: one_color,
            },
            Shape::Line {
                from: other_from,
                to: other_to,
                color: other_color,
            },
        ) => Shape::Line {
            from: blend::point(*one_from, *other_from, progress),
            to: blend::point(*one_to, *other_to, progress),
            color: blend::colour(*one_color, *other_color, progress),
        },
        (
            Shape::Text {
                at: one_at,
                text,
                scale: one_scale,
                color: one_color,
            },
            Shape::Text {
                at: other_at,
                scale: other_scale,
                color: other_color,
                ..
            },
        ) => Shape::Text {
            at: blend::point(*one_at, *other_at, progress),
            text: text.clone(),
            scale: blend::scale(*one_scale, *other_scale, progress),
            color: blend::colour(*one_color, *other_color, progress),
        },
        (
            Shape::Path {
                path: one_path,
                fill: one_fill,
                stroke: one_stroke,
            },
            Shape::Path {
                path: other_path,
                fill: other_fill,
                stroke: other_stroke,
            },
        ) => Shape::Path {
            path: blend::path(one_path, other_path, progress),
            fill: blend::paint(*one_fill, *other_fill, progress),
            stroke: blend::paint(*one_stroke, *other_stroke, progress),
        },
        _ => from.clone(),
    }
}
