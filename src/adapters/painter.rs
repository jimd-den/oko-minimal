//! Painting a scene into a frame that is already there.
//!
//! The core's renderer makes a new frame for every scene. A console that
//! presents sixty pictures a second would make sixty frames and throw
//! fifty-nine of them away; this paints into a buffer that was made once,
//! so the picture is written into the same memory every time.
//!
//! What a scene looks like is the core's business. The core can render into
//! a frame that already exists, so this is a call to it and not a second
//! copy of the shape dispatch: a second renderer is a second thing to rot,
//! and a copy of the dispatch is how a label ends up drawn in the built-in
//! font while the core draws it in the face the panel asked for.

use crate::core::frame::Frame;
use crate::core::geom::Rgba;
use crate::core::raster;
use crate::core::scene::Scene;
use crate::core::shape::Shape;

/// Paint the scene into the frame, over what is in it.
///
/// The frame is wiped first unless the scene clears it itself: a scene
/// with a clear in it paints the whole frame at some point, and the old
/// picture does not survive that.
pub fn paint_into(scene: &Scene, frame: &mut Frame) {
    if !clears_the_frame(scene) {
        frame.fill(frame.bounds(), Rgba::BLACK);
    }
    raster::render_into(frame, scene);
}

/// Whether the scene paints the whole frame at some point.
fn clears_the_frame(scene: &Scene) -> bool {
    scene
        .elements()
        .iter()
        .any(|element| matches!(element.shape, Shape::Clear(_)))
}
