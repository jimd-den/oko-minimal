//! The use cases, named in a person's words.
//!
//! These tests sit at the outermost honest boundary: a document goes in and the
//! bytes a terminal receives come out; an event goes in and the console gets an
//! answer. They do not know how the pixels are made.

use oko::adapters::terminal::{PaintError, paint};
use oko::core::event::{Event, Key, KeyCode, Modifiers};
use oko::core::focus::SurfaceId;
use oko::core::lease::OwnerId;
use oko::core::session::{Effect, Session};

#[test]
fn a_panel_document_becomes_terminal_blocks() {
    let document = "clear #05070a\nrect 0 0 2 2 #ff9d00";

    let bytes = paint(document, 2, 2).expect("a valid panel");

    assert_eq!(bytes, "\x1b[38;2;255;157;0m\x1b[48;2;255;157;0m▀▀\x1b[0m\n");
}

#[test]
fn a_broken_panel_document_names_every_bad_line() {
    let error = paint("rect 0 0 1 1 #fff\nsparkle\n", 4, 4).expect_err("two bad lines");

    let PaintError::Document(errors) = error else {
        panic!("a document error was expected, got {error}");
    };
    let lines: Vec<usize> = errors.iter().map(|error| error.line).collect();
    assert_eq!(lines, [1, 2]);
}

#[test]
fn the_escape_chord_hands_the_screen_back_to_the_console() {
    let escape = Key::new(KeyCode::Escape, Modifiers::CTRL_ALT);
    let mut session = Session::new(escape);
    session.grant_display(OwnerId(1)).expect("a grant");
    session.focus(SurfaceId(7));

    let effect = session.dispatch(Event::Key(escape));

    assert_eq!(effect, Some(Effect::ReclaimDisplay));
    assert_eq!(session.lease().holder(), None);
}

#[test]
fn an_svg_export_becomes_a_panel_document() {
    let svg = include_str!("../../examples/panel.svg");

    let document = oko::adapters::svg::translate(svg).expect("a supported export");
    let scene =
        oko::core::document::parse(&document).expect("the translator writes valid documents");

    assert_eq!(
        scene.ops()[0],
        oko::core::scene::Op::Rect {
            area: oko::core::geom::Rect::new(0, 0, 80, 48),
            color: oko::core::geom::Rgba::rgb(5, 7, 10),
        }
    );
    assert!(oko::adapters::terminal::paint(&document, 80, 48).is_ok());
}

#[test]
fn an_svg_curve_is_refused_with_its_line() {
    let svg =
        "<svg viewBox=\"0 0 8 8\">\n<path d=\"M0 0 C 1 1 2 2 3 3\" fill=\"#000000\"/>\n</svg>";

    let error = oko::adapters::svg::translate(svg).expect_err("a curve");

    assert_eq!(error.line, 2);
    assert!(error.problem.contains("`C`"));
}
