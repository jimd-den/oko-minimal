//! The runtime, seen from the outside: it holds the panel, and it answers.

use super::runtime::Runtime;
use crate::core::event::{Event, Key, KeyCode, Modifiers};
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
fn a_key_reaches_the_panel_and_the_frame_changes() {
    let mut runtime = runtime(8, 4);
    runtime
        .document(
            "flag shown\nclear #000000\nrect 0 0 2 2 #ff0000 when shown\non key enter set shown true",
        )
        .expect("a valid document");

    let before = runtime.frame().expect("a frame");
    assert_eq!(
        before.color_at(Point::new(0, 0)),
        Some(crate::core::geom::Rgba::rgb(0, 0, 0)),
        "the shape is hidden while the flag is false"
    );

    let effect = runtime.event(Event::Key(Key::plain(KeyCode::Enter)));

    assert_eq!(effect, Some(Effect::Repaint));
    let after = runtime.frame().expect("a frame");
    assert_eq!(
        after.color_at(Point::new(0, 0)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0)),
        "the shape is drawn now"
    );
}

#[test]
fn a_quit_binding_hands_the_screen_back() {
    let mut runtime = runtime(8, 4);
    runtime
        .document("on key escape quit")
        .expect("a valid document");

    let effect = runtime.event(Event::Key(Key::plain(KeyCode::Escape)));

    assert_eq!(effect, Some(Effect::ReclaimDisplay));
}
