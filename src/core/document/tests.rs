//! The document's own rules: what a document describes, and what it
//! refuses.

use super::*;
use crate::core::geom::{Point, Rect, Rgba};
use crate::core::raster;
use crate::core::shape::Shape;

#[test]
fn a_document_describes_a_scene() {
    let text = "\
# a panel
clear #05070a
rect 2 2 10 4 #ff9d00
line 0 0 9 4 #1f6feb";

    let panel = parse(text).expect("a valid document");

    let shapes: Vec<Shape> = panel
        .scene(0)
        .elements()
        .iter()
        .map(|e| e.shape.clone())
        .collect();
    assert_eq!(
        shapes,
        [
            Shape::Clear(Rgba::rgb(5, 7, 10)),
            Shape::Rect {
                area: Rect::new(2, 2, 10, 4),
                color: Rgba::rgb(255, 157, 0),
            },
            Shape::Line {
                from: Point::new(0, 0),
                to: Point::new(9, 4),
                color: Rgba::rgb(31, 111, 235),
            },
        ]
    );
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let panel = parse("\n# a comment\n\nclear #000000\n").expect("a valid document");

    assert_eq!(panel.scene(0).len(), 1);
}

#[test]
fn every_bad_line_is_reported_at_once() {
    let errors = parse("rect 0 0 1 1 #fff\nline 0 0 1 1 #zzzzzz").expect_err("two errors");

    assert_eq!(errors.len(), 2);
    assert_eq!(errors[0].line, 1);
    assert_eq!(errors[1].line, 2);
}

#[test]
fn a_path_in_a_document_reaches_the_frame() {
    let text = "path M 2 2 L 10 2 L 2 10 Z fill #ff9d00";

    let panel = parse(text).expect("a valid document");
    let frame = raster::render(&panel.scene(0), 12, 12).expect("a small frame");

    assert_eq!(
        frame.color_at(Point::new(3, 3)),
        Some(Rgba::rgb(255, 157, 0))
    );
    assert_eq!(frame.color_at(Point::new(9, 9)), Some(Rgba::BLACK));
}

#[test]
fn a_document_may_not_exceed_its_operation_budget() {
    let text = "clear #000000\n".repeat(MAX_OPS + 1);

    let errors = parse(&text).expect_err("a document over budget");

    assert!(errors[0].problem.contains(&MAX_OPS.to_string()));
}
