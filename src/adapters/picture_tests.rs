//! The picture on the screen: the frame, and the scene it was painted from.

use super::picture::Picture;
use crate::core::document;
use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};
use crate::core::scene::Scene;

/// The scene of a document, at the start of its timeline.
fn scene(document: &str) -> Scene {
    document::parse(document)
        .expect("a valid document")
        .scene(0)
}

#[test]
fn a_picture_is_the_size_it_was_asked_for() {
    let mut picture = Picture::new((8, 4));
    picture.paint(Scene::new());

    let frame = picture.frame().expect("a frame");

    assert_eq!(frame.color_at(Point::new(7, 3)), Some(Rgba::BLACK));
    assert_eq!(frame.color_at(Point::new(8, 3)), None);
}

#[test]
fn a_picture_that_is_painted_again_keeps_its_buffer() {
    let mut picture = Picture::new((8, 4));
    picture.paint(scene("clear #ff0000"));
    let before = picture.frame().expect("a frame");
    let address = before as *const Frame;

    picture.paint(scene("clear #00ff00"));

    let after = picture.frame().expect("a frame");
    assert_eq!(
        address, after as *const Frame,
        "the same buffer is written into"
    );
    assert_eq!(
        picture.frames_allocated(),
        1,
        "one buffer, however many paints"
    );
}

#[test]
fn a_scene_that_has_not_changed_is_not_painted_again() {
    let mut picture = Picture::new((8, 4));

    assert!(picture.paint(scene("clear #ff0000")));
    assert!(!picture.paint(scene("clear #ff0000")), "the same picture");
}

#[test]
fn a_picture_of_another_size_gets_a_new_buffer() {
    let mut picture = Picture::new((8, 4));
    picture.paint(scene("clear #ff0000"));

    picture.resize((16, 8));
    picture.paint(scene("clear #ff0000"));

    assert_eq!(
        picture.frames_allocated(),
        2,
        "the size changed, so the buffer did"
    );
}
