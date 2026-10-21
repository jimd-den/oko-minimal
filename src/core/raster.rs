//! Rendering: walking a display list into a frame.
//!
//! `render` is a pure function of the scene and the frame size. It allocates
//! the frame, paints each operation in order, and returns. No clock, no I/O, no
//! global state — so a test can render a scene in one line and look at a pixel.

use super::clip::clip_line;
use super::frame::Frame;
use super::geom::{Point, Rgba};
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
        Op::Line { from, to, color } => {
            if let Some((start, end)) = clip_line(*from, *to, frame.width(), frame.height()) {
                paint_line(frame, start, end, *color);
            }
        }
        Op::Text {
            at,
            text: label,
            scale,
            color,
        } => text::paint(frame, *at, label, *scale, *color),
    }
}

/// Bresenham: one pixel per step, no floating point, no gaps.
fn paint_line(frame: &mut Frame, from: Point, to: Point, color: Rgba) {
    let (mut x, mut y) = (i64::from(from.x), i64::from(from.y));
    let (end_x, end_y) = (i64::from(to.x), i64::from(to.y));
    let dx = (end_x - x).abs();
    let dy = -(end_y - y).abs();
    let step_x = if x < end_x { 1 } else { -1 };
    let step_y = if y < end_y { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        frame.set(Point::new(x as i32, y as i32), color);
        if x == end_x && y == end_y {
            break;
        }
        let twice = 2 * error;
        if twice >= dy {
            error += dy;
            x += step_x;
        }
        if twice <= dx {
            error += dx;
            y += step_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};

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
