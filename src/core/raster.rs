//! Rendering: walking a display list into a frame.
//!
//! `render` is a pure function of the scene and the frame size. It allocates
//! the frame, paints each operation in order, and returns. No clock, no I/O, no
//! global state — so a test can render a scene in one line and look at a pixel.

use super::frame::Frame;
use super::paint::path as paint_path;
use super::scene::Scene;
use super::shape::Shape;
use super::text;

pub fn render(scene: &Scene, width: u32, height: u32) -> Option<Frame> {
    let mut frame = Frame::new(width, height)?;
    for element in scene.elements() {
        paint(&mut frame, &element.shape);
    }
    Some(frame)
}

fn paint(frame: &mut Frame, shape: &Shape) {
    match shape {
        Shape::Clear(color) => {
            let everything = frame.bounds();
            frame.fill(everything, *color);
        }
        Shape::Rect { area, color } => frame.fill(*area, *color),
        Shape::Line { from, to, color } => frame.line(*from, *to, *color),
        Shape::Text {
            at,
            text: label,
            scale,
            color,
            spacing,
        } => text::paint(
            frame,
            &text::Text {
                at: *at,
                text: label,
                scale: *scale,
                spacing: *spacing,
                colour: *color,
            },
        ),
        Shape::Path { path, fill, stroke } => paint_path(frame, path, *fill, *stroke),
    }
}
