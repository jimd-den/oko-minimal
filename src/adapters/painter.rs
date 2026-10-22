//! Painting a scene into a frame that is already there.
//!
//! The core's renderer makes a new frame for every scene. A console that
//! presents sixty pictures a second would make sixty frames and throw
//! fifty-nine of them away; this paints into a buffer that was made once,
//! so the picture is written into the same memory every time.
//!
//! What a scene looks like is the core's business, and the painting here
//! is copied from `core::raster`, because the core has no way to render
//! into a frame that already exists. When it grows one, this file
//! becomes a call to it.

use crate::core::frame::Frame;
use crate::core::geom::Rgba;
use crate::core::paint::path as paint_path;
use crate::core::scene::Scene;
use crate::core::shape::Shape;
use crate::core::text;

/// Paint the scene into the frame, over what is in it.
///
/// The frame is wiped first unless the scene clears it itself: a scene
/// with a clear in it paints the whole frame at some point, and the old
/// picture does not survive that.
pub fn paint_into(scene: &Scene, frame: &mut Frame) {
    if !clears_the_frame(scene) {
        frame.fill(frame.bounds(), Rgba::BLACK);
    }
    for element in scene.elements() {
        paint(frame, &element.shape);
    }
}

/// Whether the scene paints the whole frame at some point.
fn clears_the_frame(scene: &Scene) -> bool {
    scene
        .elements()
        .iter()
        .any(|element| matches!(element.shape, Shape::Clear(_)))
}

/// Paint one shape, as `core::raster` paints it.
fn paint(frame: &mut Frame, shape: &Shape) {
    match shape {
        Shape::Clear(color) => frame.fill(frame.bounds(), *color),
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
                text: label.as_str(),
                scale: *scale,
                spacing: *spacing,
                colour: *color,
            },
        ),
        Shape::Path { path, fill, stroke } => paint_path(frame, path, *fill, *stroke),
    }
}
