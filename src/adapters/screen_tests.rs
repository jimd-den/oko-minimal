//! The cells the terminal is showing, and what a new frame changes.

use super::screen::Screen;
use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};

/// A frame of the given size, all black.
fn frame(width: u32, height: u32) -> Frame {
    Frame::new(width, height).expect("a small frame")
}

#[test]
fn a_frame_that_has_not_changed_changes_no_cell() {
    let picture = frame(4, 2);
    let mut screen = Screen::of(&picture);
    screen.reset(&picture);

    let changes = screen.sync(&picture);

    assert!(changes.is_empty(), "{changes:?}");
}

#[test]
fn a_changed_cell_is_reported_with_its_place() {
    let before = frame(4, 2);
    let mut after = before.clone();
    after.set(Point::new(2, 0), Rgba::WHITE);
    let mut screen = Screen::of(&before);
    screen.reset(&before);

    let changes = screen.sync(&after);

    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].row, 0);
    assert_eq!(changes[0].column, 2);
    assert_eq!(changes[0].cell.top, Rgba::WHITE);
    assert_eq!(changes[0].cell.bottom, Rgba::BLACK);
}

#[test]
fn the_screen_is_brought_up_to_date_by_the_change() {
    let before = frame(4, 2);
    let mut after = before.clone();
    after.set(Point::new(2, 0), Rgba::WHITE);
    let mut screen = Screen::of(&before);
    screen.reset(&before);

    screen.sync(&after);
    let changes = screen.sync(&after);

    assert!(changes.is_empty(), "the second look finds nothing new");
}

#[test]
fn a_frame_of_another_size_does_not_fit_the_screen() {
    let picture = frame(4, 2);
    let screen = Screen::of(&picture);

    assert!(screen.fits(&picture));
    assert!(!screen.fits(&frame(6, 4)));
}
