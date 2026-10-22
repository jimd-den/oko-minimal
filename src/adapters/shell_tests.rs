//! The frame on the screen: what the terminal is told, and when.

use super::runtime::Runtime;
use super::shell::present;
use crate::core::event::{Event, Key, KeyCode, Modifiers};

/// A panel with a shape that moves to the right over a second.
const MOVING: &str = "clear #000000\nrect 0 0 4 4 #ff0000 to 4 0 4 4 #ff0000 over 1s";

#[test]
fn a_frame_that_did_not_change_is_not_sent_again() {
    let mut runtime = Runtime::new(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");
    out.clear();

    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn a_picture_that_moved_is_sent_as_damage() {
    let mut runtime = Runtime::new(8, 4);
    runtime.document(MOVING).expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");
    out.clear();

    runtime.tick(500);
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    let sent = String::from_utf8_lossy(&out);
    assert!(sent.contains("\x1b["), "the cells were addressed: {sent:?}");
    assert!(
        out.len() < 200,
        "the damage is smaller than the frame: {} bytes",
        out.len()
    );
}

#[test]
fn the_escape_chord_hands_the_screen_back_to_the_console() {
    let mut runtime = Runtime::new(8, 4);
    runtime
        .document("on key escape quit")
        .expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    let effect = runtime.event(Event::Key(Key::new(KeyCode::Escape, Modifiers::CTRL_ALT)));

    assert_eq!(effect, Some(crate::core::session::Effect::ReclaimDisplay));
}
