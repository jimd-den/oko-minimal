//! Rendering: walking a display list into a frame.
//!
//! `render` is a pure function of the scene and the frame size. It allocates
//! the frame, paints each operation in order, and returns. No clock, no I/O, no
//! global state — so a test can render a scene in one line and look at a pixel.
//!
//! `render_into` paints into a frame the caller already has. A runtime
//! that keeps its frame between ticks paints into it again, and an adapter
//! that draws a panel's scene does not have to copy the shape dispatch to
//! do it: a second renderer is a second thing to rot.

#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod tests;

use super::frame::Frame;
use super::paint::path as paint_path;
use super::scene::Scene;
use super::shape::Shape;
use super::text;
use super::typeface::Setting;

/// The scene drawn into a frame of `width` by `height` pixels.
pub fn render(scene: &Scene, width: u32, height: u32) -> Option<Frame> {
    let mut frame = Frame::new(width, height)?;
    render_into(&mut frame, scene);
    Some(frame)
}

/// Paint the scene into a frame the caller already has.
///
/// The frame is not emptied first: what the caller had is the caller's, and
/// `Frame::clear` is there for a caller that wants a clean frame.
pub fn render_into(frame: &mut Frame, scene: &Scene) {
    for element in scene.elements() {
        paint(frame, &element.shape, scene.font());
    }
}

/// Paint one shape, in the font the scene is drawn in.
fn paint(frame: &mut Frame, shape: &Shape, face: Option<&Setting>) {
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
            face,
        ),
        Shape::Path { path, fill, stroke } => paint_path(frame, path, *fill, *stroke),
    }
}
