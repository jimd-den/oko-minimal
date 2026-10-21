//! The bytes a frame is made of.

use super::presenter;
use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};

#[test]
fn one_cell_shows_two_pixels() {
    let mut frame = Frame::new(1, 2).expect("a small frame");
    frame.set(Point::new(0, 1), Rgba::WHITE);

    let bytes = presenter::encode(&frame);

    assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;255;255;255m▀\x1b[0m\n");
}

#[test]
fn a_run_of_one_colour_emits_one_escape() {
    let frame = Frame::new(2, 2).expect("a small frame");

    let bytes = presenter::encode(&frame);

    assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m▀▀\x1b[0m\n");
}

#[test]
fn an_odd_last_row_repeats_itself() {
    let frame = Frame::new(1, 1).expect("a small frame");

    let bytes = presenter::encode(&frame);

    assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m▀\x1b[0m\n");
}

#[test]
fn a_document_paints_into_terminal_blocks() {
    let bytes = presenter::paint("clear #ff0000", 2, 2).expect("a valid document");

    assert_eq!(bytes, "\x1b[38;2;255;0;0m\x1b[48;2;255;0;0m▀▀\x1b[0m\n");
}

#[test]
fn a_broken_document_names_its_line() {
    let error = presenter::paint("clear #000000\nsparkle", 4, 4).expect_err("a bad document");

    assert_eq!(error.to_string(), "line 2: unknown directive `sparkle`\n");
}

#[test]
fn a_screen_places_every_row_at_the_left_margin() {
    let mut frame = Frame::new(2, 4).expect("a small frame");
    frame.set(Point::new(0, 0), Rgba::WHITE);

    let bytes = presenter::screen(&frame);

    assert!(bytes.starts_with("\x1b[1;1H"), "{bytes:?}");
    assert!(bytes.contains("\x1b[2;1H"), "{bytes:?}");
    assert!(bytes.ends_with("\x1b[J"), "{bytes:?}");
}

#[test]
fn a_window_of_cells_is_a_frame_of_pixels() {
    assert_eq!(presenter::pixel_size(80, 24), (80, 48));
}

#[test]
fn an_update_between_identical_frames_is_empty() {
    let frame = Frame::new(2, 2).expect("a small frame");

    let bytes = presenter::screen_update(&frame, &frame);

    assert_eq!(bytes, "");
}

#[test]
fn an_update_paints_only_the_changed_cell() {
    let previous = Frame::new(2, 2).expect("a small frame");
    let mut frame = Frame::new(2, 2).expect("a small frame");
    frame.set(Point::new(1, 1), Rgba::WHITE);

    let bytes = presenter::screen_update(&previous, &frame);

    assert_eq!(
        bytes,
        "\x1b[1;2H\x1b[38;2;0;0;0m\x1b[48;2;255;255;255m▀\x1b[0m"
    );
}

#[test]
fn an_update_skips_rows_that_did_not_change() {
    let previous = Frame::new(1, 4).expect("a small frame");
    let mut frame = Frame::new(1, 4).expect("a small frame");
    frame.set(Point::new(0, 2), Rgba::WHITE);

    let bytes = presenter::screen_update(&previous, &frame);

    assert!(bytes.contains("\x1b[2;1H"), "{bytes:?}");
    assert!(!bytes.contains("\x1b[1;1H"), "{bytes:?}");
}

#[test]
fn an_update_of_a_different_size_is_the_full_screen() {
    let previous = Frame::new(2, 2).expect("a small frame");
    let frame = Frame::new(4, 2).expect("a wider frame");

    let bytes = presenter::screen_update(&previous, &frame);

    assert_eq!(bytes, presenter::screen(&frame));
}
