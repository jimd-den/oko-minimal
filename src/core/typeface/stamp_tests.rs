//! A stamp, seen from the outside: it paints what the glyph painted.

use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};
use crate::core::typeface::Face;
use crate::core::typeface::stamp::Stamp;

/// The frame an A pressed at (2, 30) leaves behind.
fn pressed_a() -> Frame {
    let face = Face::by_name("dejavu-mono").expect("a face");
    let glyph = face.glyph('A', 16.0, None).expect("an A");
    let mut frame = Frame::new(40, 40).expect("a frame");
    Stamp::of(&glyph, Rgba::WHITE).blit(&mut frame, Point::new(2, 30));
    frame
}

#[test]
fn a_glyph_is_drawn_on_the_frame() {
    let frame = pressed_a();

    let painted =
        (0..40).any(|y| (0..40).any(|x| frame.color_at(Point::new(x, y)) == Some(Rgba::WHITE)));
    assert!(painted, "the A is on the frame");
}

#[test]
fn a_glyph_stands_on_its_baseline() {
    let frame = pressed_a();

    let below =
        (30..40).any(|y| (0..40).any(|x| frame.color_at(Point::new(x, y)) == Some(Rgba::WHITE)));
    assert!(!below, "the A stands on the line, not below it");
}

#[test]
fn a_stamps_pixels_are_the_glyphs_colour() {
    let face = Face::by_name("dejavu-mono").expect("a face");
    let glyph = face.glyph('A', 16.0, None).expect("an A");

    let stamp = Stamp::of(&glyph, Rgba::WHITE);

    assert!(
        stamp.pixels.contains(&Rgba::WHITE),
        "the stamp holds the A's pixels in its colour"
    );
}

#[test]
fn a_stamp_holds_the_advance_of_its_glyph() {
    let face = Face::by_name("dejavu-mono").expect("a face");
    let glyph = face.glyph('A', 16.0, None).expect("an A");

    let stamp = Stamp::of(&glyph, Rgba::WHITE);

    assert_eq!(stamp.advance, glyph.advance, "the pen moves the same");
}
