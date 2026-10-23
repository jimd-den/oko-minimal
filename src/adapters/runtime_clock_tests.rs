//! The runtime's clock: the picture over time.

use super::runtime::Runtime;
use crate::core::frame::Frame;
use crate::core::geom::Point;

/// A runtime with a frame of the given size, in pixels.
fn runtime(width: u32, height: u32) -> Runtime {
    Runtime::new(width, height)
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

#[test]
fn a_tick_writes_into_the_frame_buffer_that_is_already_there() {
    let mut runtime = runtime(8, 4);
    runtime.document(MOVING).expect("a valid document");
    let before = runtime.frame().expect("a frame");
    let address = before as *const Frame;

    runtime.tick(500);
    runtime.tick(600);

    let after = runtime.frame().expect("a frame");
    assert_eq!(
        address, after as *const Frame,
        "the same buffer is written into"
    );
    assert_eq!(
        runtime.frames_allocated(),
        1,
        "one buffer, however many ticks"
    );
}

#[test]
fn a_looping_panel_never_settles() {
    let mut runtime = runtime(8, 4);
    runtime
        .document(
            "clear #000000\nrect 0 0 4 4 #ff0000 to 4 0 4 4 #ff0000 over 1s to 0 0 4 4 #ff0000 over 1s loop",
        )
        .expect("a valid looping document");

    assert!(runtime.tick(2000));
    assert!(
        runtime.tick(2500),
        "a looping panel keeps moving past its first cycle"
    );
    let frame = runtime.frame().expect("a frame");
    assert_eq!(
        frame.color_at(Point::new(2, 0)),
        Some(crate::core::geom::Rgba::rgb(255, 0, 0))
    );
}
