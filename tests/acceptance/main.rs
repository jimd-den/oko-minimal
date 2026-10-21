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

    let document = oko::adapters::svg::translate(svg)
        .expect("a supported export")
        .document;
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

/// A panel with a hole is still a panel: the shapes the console can draw are
/// in the document, and the one it cannot is named with the line it was on.
#[test]
fn a_panel_with_a_hole_still_loads_and_reports_the_hole() {
    let svg = "<svg viewBox=\"0 0 8 8\">\n<circle cx=\"4\" cy=\"4\" r=\"2\"/>\n<rect x=\"1\" y=\"1\" width=\"2\" height=\"2\" fill=\"#ff9d00\"/>\n</svg>";

    let translation = oko::adapters::svg::translate(svg).expect("a panel with a hole");

    assert_eq!(translation.document, "rect 1 1 2 2 #ff9d00\n");
    assert!(
        oko::core::document::parse(&translation.document).is_ok(),
        "the panel around the hole is a document"
    );
    assert_eq!(translation.skipped.len(), 1);
    assert_eq!(translation.skipped[0].line, 2);
    assert!(translation.skipped[0].reason.contains("circle"));
}

#[test]
fn an_arc_is_drawn_as_a_path_directive() {
    let svg =
        "<svg viewBox=\"0 0 8 8\">\n<path d=\"M0 0 A 4 4 0 0 1 4 4\" fill=\"#000000\"/>\n</svg>";

    let translation = oko::adapters::svg::translate(svg).expect("a panel with an arc");

    assert!(
        translation.skipped.is_empty(),
        "an arc is drawn, not refused"
    );

    let scene = oko::core::document::parse(&translation.document).expect("a document");
    let oko::core::scene::Op::Path { path, .. } = &scene.ops()[0] else {
        panic!("the arc is a path operation");
    };
    assert_eq!(path.segments.len(), 2, "a move and one cubic");
    assert!(matches!(
        path.segments[1],
        oko::core::path::Segment::Cubic { .. }
    ));
}

#[test]
fn an_svg_label_becomes_a_text_operation() {
    let svg = include_str!("../../examples/text-panel.svg");

    let document = oko::adapters::svg::translate(svg)
        .expect("a supported export")
        .document;
    let scene =
        oko::core::document::parse(&document).expect("the translator writes valid documents");

    let directives: Vec<&str> = document
        .lines()
        .filter(|line| line.starts_with("text "))
        .collect();
    assert_eq!(
        directives,
        [
            "text 6 3 \"OKO\" 3 #ff9d00",
            "text 6 31 \"SYSTEM READY\" 1 #c9d1d9"
        ]
    );

    let labels: Vec<oko::core::scene::Op> = scene
        .ops()
        .iter()
        .filter(|op| matches!(op, oko::core::scene::Op::Text { .. }))
        .cloned()
        .collect();
    assert_eq!(
        labels,
        [
            oko::core::scene::Op::Text {
                at: oko::core::geom::Point::new(6, 3),
                text: "OKO".to_string(),
                scale: 3,
                color: oko::core::geom::Rgba::rgb(255, 157, 0),
            },
            oko::core::scene::Op::Text {
                at: oko::core::geom::Point::new(6, 31),
                text: "SYSTEM READY".to_string(),
                scale: 1,
                color: oko::core::geom::Rgba::rgb(201, 209, 217),
            },
        ]
    );
}

/// The command writes the panel to standard output and the report of what it
/// could not draw to standard error, so a pipe keeps only the panel.
#[test]
fn the_command_writes_the_panel_out_and_the_report_away() {
    let svg = "<svg viewBox=\"0 0 8 8\">\n<circle cx=\"4\" cy=\"4\" r=\"2\"/>\n</svg>";
    let path = std::env::temp_dir().join("oko-svg-hole.svg");
    std::fs::write(&path, svg).expect("the export is written");

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_oko-svg"))
        .arg(&path)
        .output()
        .expect("the command runs");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let report = String::from_utf8_lossy(&output.stderr);
    assert!(
        report.contains("circle"),
        "the report names the shape: {report}"
    );
    assert!(
        report.contains(":2:"),
        "the report names the line: {report}"
    );
}
