//! The bytes a frame is made of.

use super::presenter;
use super::screen::Screen;
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
fn a_frame_that_did_not_change_costs_no_bytes() {
    let picture = Frame::new(4, 2).expect("a small frame");
    let mut screen = Screen::of(&picture);
    screen.reset(&picture);

    let bytes = presenter::damage(screen.sync(&picture));

    assert_eq!(bytes, "", "nothing changed, so nothing is sent");
}

#[test]
fn one_changed_cell_moves_the_cursor_to_it_and_paints_its_colour() {
    let before = Frame::new(4, 2).expect("a small frame");
    let mut after = before.clone();
    after.set(Point::new(2, 0), Rgba::WHITE);
    let mut screen = Screen::of(&before);
    screen.reset(&before);

    let bytes = presenter::damage(screen.sync(&after));

    assert_eq!(
        bytes,
        "\x1b[1;3H\x1b[38;2;255;255;255m\x1b[48;2;0;0;0m\u{2580}"
    );
}

#[test]
fn a_changed_cell_in_the_middle_of_a_row_moves_the_cursor_to_its_column() {
    let before = Frame::new(6, 2).expect("a small frame");
    let mut after = before.clone();
    after.set(Point::new(3, 0), Rgba::WHITE);
    let mut screen = Screen::of(&before);
    screen.reset(&before);

    let bytes = presenter::damage(screen.sync(&after));

    assert!(bytes.contains("\x1b[1;4H"), "{bytes:?}");
    assert!(!bytes.contains("\x1b[1;1H"), "{bytes:?}");
}

#[test]
fn the_damage_of_a_panel_with_one_percent_change_is_far_smaller_than_the_frame() {
    let before = busy_frame(80, 48);
    let mut after = before.clone();
    for cell in 0..80u32 * 24 {
        if cell % 100 == 0 {
            let x = (cell % 80) as i32;
            let y = (cell / 80 * 2) as i32;
            after.set(Point::new(x, y), Rgba::WHITE);
        }
    }
    let mut screen = Screen::of(&before);
    screen.reset(&before);

    let damage = presenter::damage(screen.sync(&after));
    let full = presenter::screen(&after);

    assert!(
        damage.len() * 20 < full.len(),
        "damage {} bytes against a full frame of {} bytes",
        damage.len(),
        full.len()
    );
}

/// A frame of many colours, so that a full paint costs an escape per cell.
fn busy_frame(width: u32, height: u32) -> Frame {
    let mut frame = Frame::new(width, height).expect("a frame");
    for y in 0..height {
        for x in 0..width {
            let index = x + y * width;
            frame.set(
                Point::new(x as i32, y as i32),
                Rgba::rgb(
                    (index * 37 % 251) as u8,
                    (index * 91 % 253) as u8,
                    (index * 17 % 247) as u8,
                ),
            );
        }
    }
    frame
}
