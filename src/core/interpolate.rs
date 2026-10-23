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
                spacing: one_spacing,
            },
            Shape::Text {
                at: other_at,
                scale: other_scale,
                color: other_color,
                spacing: _other_spacing,
                ..
            },
        ) => Shape::Text {
            at: blend::point(*one_at, *other_at, progress),
            text: text.clone(),
            scale: blend::scale(*one_scale, *other_scale, progress),
            color: blend::colour(*one_color, *other_color, progress),
            spacing: *one_spacing,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::path::{Path, Segment};

    fn rect(x: i32) -> Shape {
        Shape::Rect {
            area: Rect::new(x, 0, 4, 4),
            color: Rgba::WHITE,
        }
    }

    #[test]
    fn a_shape_is_halfway_between_two_shapes() {
        let halfway = between(&rect(0), &rect(10), 0.5);

        assert_eq!(
            halfway,
            Shape::Rect {
                area: Rect::new(5, 0, 4, 4),
                color: Rgba::WHITE
            }
        );
    }

    #[test]
    fn a_texts_label_does_not_change() {
        let one = Shape::Text {
            at: Point::new(0, 0),
            text: "OK".to_string(),
            spacing: 0,
            scale: 1,
            color: Rgba::WHITE,
        };
        let other = Shape::Text {
            at: Point::new(10, 0),
            text: "OK".to_string(),
            spacing: 0,
            scale: 1,
            color: Rgba::BLACK,
        };

        let halfway = between(&one, &other, 0.5);

        assert_eq!(
            halfway,
            Shape::Text {
                at: Point::new(5, 0),
                text: "OK".to_string(),
                spacing: 0,
                scale: 1,
                color: Rgba::rgb(128, 128, 128)
            }
        );
    }

    #[test]
    fn a_paths_segments_move() {
        let one = Shape::Path {
            path: Path {
                segments: vec![Segment::Move {
                    to: Point::new(0, 0),
                }],
            },
            fill: Some(Rgba::WHITE),
            stroke: None,
        };
        let other = Shape::Path {
            path: Path {
                segments: vec![Segment::Move {
                    to: Point::new(0, 10),
                }],
            },
            fill: Some(Rgba::BLACK),
            stroke: None,
        };

        let halfway = between(&one, &other, 0.5);

        let Shape::Path { path, .. } = halfway else {
            panic!("a path is a path");
        };
        assert_eq!(
            path.segments[0],
            Segment::Move {
                to: Point::new(0, 5)
            }
        );
    }
}
