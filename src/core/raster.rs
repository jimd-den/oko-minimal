//! Rendering: walking a display list into a frame.
//!
//! `render` is a pure function of the scene and the frame size. It allocates
//! the frame, paints each operation in order, and returns. No clock, no I/O, no
//! global state — so a test can render a scene in one line and look at a pixel.

use super::frame::Frame;
use super::paint::path as paint_path;
use super::scene::{Op, Scene};
use super::text;

pub fn render(scene: &Scene, width: u32, height: u32) -> Option<Frame> {
    let mut frame = Frame::new(width, height)?;
    for op in scene.ops() {
        paint(&mut frame, op);
    }
    Some(frame)
}

fn paint(frame: &mut Frame, op: &Op) {
    match op {
        Op::Clear(color) => {
            let everything = frame.bounds();
            frame.fill(everything, *color);
        }
        Op::Rect { area, color } => frame.fill(*area, *color),
        Op::Line { from, to, color } => frame.line(*from, *to, *color),
        Op::Text {
            at,
            text: label,
            scale,
            color,
        } => text::paint(frame, *at, label, *scale, *color),
        Op::Path { path, fill, stroke } => paint_path(frame, path, *fill, *stroke),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::path::{Path, Segment};

    #[test]
    fn a_rectangle_paints_its_area() {
        let mut scene = Scene::new();
        scene.push(Op::Rect {
            area: Rect::new(1, 1, 2, 2),
            color: Rgba::WHITE,
        });

        let frame = render(&scene, 4, 3).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(1, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 1)), Some(Rgba::BLACK));
        assert_eq!(frame.color_at(Point::new(3, 3)), None);
    }

    #[test]
    fn a_clear_paints_over_what_came_before() {
        let mut scene = Scene::new();
        scene.push(Op::Rect {
            area: Rect::new(0, 0, 2, 2),
            color: Rgba::WHITE,
        });
        scene.push(Op::Clear(Rgba::BLACK));

        let frame = render(&scene, 2, 2).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_reaches_both_ends() {
        let mut scene = Scene::new();
        scene.push(Op::Line {
            from: Point::new(0, 0),
            to: Point::new(3, 3),
            color: Rgba::WHITE,
        });

        let frame = render(&scene, 4, 4).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(1, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 3)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 0)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_from_far_outside_is_clipped_and_does_not_hang() {
        let mut scene = Scene::new();
        scene.push(Op::Line {
            from: Point::new(-1_000_000, 2),
            to: Point::new(1_000_000, 2),
            color: Rgba::WHITE,
        });

        let frame = render(&scene, 8, 5).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(7, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 1)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_text_operation_paints_its_glyph() {
        let mut scene = Scene::new();
        scene.push(Op::Text {
            at: Point::new(0, 0),
            text: "I".to_string(),
            scale: 1,
            color: Rgba::WHITE,
        });

        let frame = render(&scene, 5, 7).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(2, 3)), Some(Rgba::WHITE));
    }

    #[test]
    fn a_path_paints_its_interior() {
        let mut scene = Scene::new();
        scene.push(Op::Path {
            path: Path {
                segments: vec![
                    Segment::Move {
                        to: Point::new(1, 1),
                    },
                    Segment::Line {
                        to: Point::new(9, 1),
                    },
                    Segment::Line {
                        to: Point::new(1, 9),
                    },
                    Segment::Close,
                ],
            },
            fill: Some(Rgba::WHITE),
            stroke: None,
        });

        let frame = render(&scene, 12, 12).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(9, 9)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_stroked_path_paints_its_edge_and_not_its_inside() {
        let mut scene = Scene::new();
        scene.push(Op::Path {
            path: Path {
                segments: vec![
                    Segment::Move {
                        to: Point::new(1, 1),
                    },
                    Segment::Line {
                        to: Point::new(9, 1),
                    },
                    Segment::Line {
                        to: Point::new(1, 9),
                    },
                    Segment::Close,
                ],
            },
            fill: None,
            stroke: Some(Rgba::WHITE),
        });

        let frame = render(&scene, 12, 12).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(5, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_entirely_outside_paints_nothing() {
        let mut scene = Scene::new();
        scene.push(Op::Line {
            from: Point::new(-10, -10),
            to: Point::new(-10, 10),
            color: Rgba::WHITE,
        });

        let frame = render(&scene, 4, 4).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }
}
