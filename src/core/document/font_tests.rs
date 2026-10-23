//! The font a document names, and the letters it draws.
//!
//! The font belongs to the panel and not to a shape (ADR 0014): a document
//! that names a face changes the letterforms of its labels and nothing
//! else. The built-in font is the default, and a document that says
//! nothing about fonts is drawn in it.

use super::parse;
use crate::core::geom::{Point, Rgba};
use crate::core::raster;

#[test]
fn a_document_that_names_a_face_draws_its_text_in_that_face() {
    let named = parse("font \"inter\"\ntext 0 0 \"I\" 1 #ffffff").expect("a face");
    let plain = parse("text 0 0 \"I\" 1 #ffffff").expect("a document without a face");

    let named_frame = raster::render(&named.scene(0), 20, 12).expect("a frame");
    let plain_frame = raster::render(&plain.scene(0), 20, 12).expect("a frame");

    assert_ne!(named_frame, plain_frame, "the face draws other letters");
}

#[test]
fn the_built_in_font_draws_the_same_text_as_it_always_did() {
    let panel = parse("text 0 0 \"I\" 1 #ffffff").expect("a document");

    let frame = raster::render(&panel.scene(0), 5, 7).expect("a frame");

    assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::WHITE));
    assert_eq!(frame.color_at(Point::new(4, 6)), Some(Rgba::WHITE));
    assert_eq!(frame.color_at(Point::new(2, 3)), Some(Rgba::WHITE));
    assert_eq!(frame.color_at(Point::new(0, 3)), Some(Rgba::BLACK));
}
