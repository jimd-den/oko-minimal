//! Painting into a frame that is already there.

use super::painter;
use crate::core::document;
use crate::core::frame::Frame;
use crate::core::geom::Rgba;
use crate::core::raster;
use crate::core::scene::Scene;

/// The scene of a document, at the start of its timeline.
fn scene(document: &str) -> Scene {
    document::parse(document)
        .expect("a valid document")
        .scene(0)
}

/// The frame the core's renderer makes for a scene.
fn fresh(scene: &Scene, width: u32, height: u32) -> Frame {
    raster::render(scene, width, height).expect("a frame")
}

#[test]
fn painting_into_a_used_frame_gives_the_same_picture_as_a_new_one() {
    let picture = scene("clear #102030\nrect 1 1 3 2 #ff0000");
    let mut frame = fresh(&picture, 8, 4);
    frame.fill(frame.bounds(), Rgba::WHITE);

    painter::paint_into(&picture, &mut frame);

    assert_eq!(frame, fresh(&picture, 8, 4));
}

#[test]
fn a_scene_without_a_clear_wipes_the_old_picture() {
    let picture = scene("rect 0 0 2 2 #ff0000");
    let mut frame = fresh(&picture, 4, 2);
    frame.fill(frame.bounds(), Rgba::WHITE);

    painter::paint_into(&picture, &mut frame);

    assert_eq!(frame, fresh(&picture, 4, 2));
}
