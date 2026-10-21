//! The runtime, seen from the outside: it holds the panel, and it answers.

use super::runtime::Runtime;
use crate::core::event::{Event, Key, KeyCode, Modifiers};
use crate::core::frame::Frame;
use crate::core::geom::Point;
use crate::core::session::Effect;

/// A runtime with a frame of the given size, in pixels.
fn runtime(width: u32, height: u32) -> Runtime {
    Runtime::new(width, height)
}

/// The escape chord: the way back to the console.
fn chord() -> Key {
    Key::new(KeyCode::Escape, Modifiers::CTRL_ALT)
}

#[test]
fn the_runtime_presents_the_document_it_was_given() {
    let mut runtime = runtime(8, 4);

    runtime.document("clear #ff0000").expect("a valid document");

    let frame = runtime.frame().expect("a frame");
    assert_eq!(
        frame.color_at(Point::new(0, 0)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn a_document_that_does_not_parse_leaves_the_last_good_panel_in_place() {
    let mut runtime = runtime(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");

    let problem = runtime.document("sparkle").expect_err("a broken document");

    assert!(problem.contains("sparkle"), "{problem}");
    let frame = runtime.frame().expect("the last good frame");
    assert_eq!(
        frame.color_at(Point::new(0, 0)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn a_later_document_replaces_the_panel() {
    let mut runtime = runtime(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");

    runtime.document("clear #00ff00").expect("a valid document");

    let frame = runtime.frame().expect("a frame");
    assert_eq!(
        frame.color_at(Point::new(0, 0)),
        Some(crate::core::geom::Rgba::rgb(0, 255, 0))
    );
}

#[test]
fn the_escape_chord_hands_the_screen_back() {
    let mut runtime = runtime(8, 4);
    runtime.take_display().expect("the display");

    let effect = runtime.event(Event::Key(chord()));

    assert_eq!(effect, Some(Effect::ReclaimDisplay));
    assert_eq!(runtime.lease().holder(), None);
}

#[test]
fn a_resize_repaints_at_the_new_size() {
    let mut runtime = runtime(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");

    let effect = runtime.event(Event::Resize {
        width: 16,
        height: 8,
    });

    assert_eq!(effect, Some(Effect::Repaint));
    let frame = runtime.frame().expect("a frame");
    assert_eq!(
        frame.color_at(Point::new(15, 7)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn a_key_with_nothing_focused_is_dropped() {
    let mut runtime = runtime(8, 4);

    let effect = runtime.event(Event::Key(Key::plain(KeyCode::Char('a'))));

    assert_eq!(effect, None);
}

#[test]
fn a_frame_is_the_size_it_was_asked_for() {
    let mut runtime = runtime(8, 4);
    runtime.document("clear #000000").expect("a valid document");

    let frame: &Frame = runtime.frame().expect("a frame");

    assert_eq!(
        frame.color_at(Point::new(7, 3)),
        Some(crate::core::geom::Rgba::BLACK)
    );
    assert_eq!(frame.color_at(Point::new(8, 3)), None);
}

/// A panel with a shape that moves to the right over a second.
const MOVING: &str = "clear #000000\nrect 0 0 4 4 #ff0000 to 4 0 4 4 #ff0000 over 1s";

#[test]
fn a_moving_shape_is_painted_again_at_a_later_moment() {
    let mut runtime = runtime(8, 4);
    runtime.document(MOVING).expect("a valid document");

    let painted = runtime.tick(500);

    assert!(painted, "the moment brought a new picture");
    let frame = runtime.frame().expect("a frame");
    assert_eq!(
        frame.color_at(Point::new(2, 0)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0)),
        "the shape has moved halfway"
    );
}

#[test]
fn the_same_moment_is_painted_once() {
    let mut runtime = runtime(8, 4);
    runtime.document(MOVING).expect("a valid document");

    assert!(runtime.tick(500));
    assert!(!runtime.tick(500), "the same moment brings no new picture");
}

#[test]
fn a_panel_with_no_motion_is_painted_once_and_never_again() {
    let mut runtime = runtime(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");

    assert!(!runtime.tick(100), "a still panel is not painted again");
}

#[test]
fn the_picture_settles_when_the_timeline_has_ended() {
    let mut runtime = runtime(8, 4);
    runtime.document(MOVING).expect("a valid document");

    assert!(runtime.tick(1000), "the end of the timeline is painted");
    assert!(!runtime.tick(1100), "the picture has come to rest");
}
