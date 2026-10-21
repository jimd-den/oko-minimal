//! The translator, seen from the outside.
//!
//! These tests are the seam the ADR describes: a whole export goes in, and a
//! document and a report of what was left out come out.

use crate::adapters::svg::{Translation, translate};

/// The document a translation produced.
fn document(svg: &str) -> String {
    translate(svg).expect("a translated export").document
}

/// The reasons a translation left shapes out.
fn reasons(translation: &Translation) -> Vec<&str> {
    translation
        .skipped
        .iter()
        .map(|skipped| skipped.reason.as_str())
        .collect()
}

#[test]
fn a_group_translate_moves_its_children() {
    let svg = r##"<svg viewBox="0 0 20 10"><g transform="translate(10 3)"><rect x="2" y="2" width="4" height="5" fill="#ff9d00"/></g></svg>"##;

    assert_eq!(document(svg), "rect 12 5 4 5 #ff9d00\n");
}

#[test]
fn a_filled_rectangle_path_becomes_one_rect() {
    let svg = r##"<svg viewBox="0 0 10 10"><path d="M2 2 H8 V6 H2 Z" fill="#102030"/></svg>"##;

    assert_eq!(document(svg), "rect 2 2 6 4 #102030\n");
}

#[test]
fn an_svg_text_becomes_a_text_directive() {
    let svg = "<svg viewBox=\"0 0 80 48\"><text x=\"6\" y=\"24\" font-size=\"21\" fill=\"#ff9d00\">OKO</text></svg>";

    assert_eq!(document(svg), "text 6 3 \"OKO\" 3 #ff9d00\n");
}

#[test]
fn a_self_closing_text_leaves_no_style_behind() {
    let error =
        translate("<svg viewBox=\"0 0 20 20\"><text/>stray</svg>").expect_err("a stray label");

    assert!(error.problem.contains("only supported inside"));
}

#[test]
fn a_translated_panel_is_a_document_the_parser_accepts() {
    let svg = "<svg viewBox=\"0 0 80 48\"><rect x=\"2\" y=\"2\" width=\"10\" height=\"6\" fill=\"#ff9d00\"/><polyline points=\"0,0 1,1 2,2\" fill=\"none\" stroke=\"#ffffff\"/><text x=\"6\" y=\"24\" font-size=\"21\" fill=\"#ffffff\">OKO</text></svg>";

    let document = document(svg);

    assert!(crate::core::document::parse(&document).is_ok());
}

#[test]
fn an_unknown_element_is_left_out_and_named() {
    let svg = "<svg viewBox=\"0 0 8 8\">\n<circle cx=\"4\" cy=\"4\" r=\"2\"/>\n</svg>";

    let translation = translate(svg).expect("an export with a hole");

    assert_eq!(document(svg), "");
    assert_eq!(translation.skipped.len(), 1);
    assert_eq!(translation.skipped[0].line, 2);
    assert!(translation.skipped[0].reason.contains("circle"));
}

#[test]
fn a_curve_is_left_out_and_the_rest_of_the_panel_is_drawn() {
    let svg = "<svg viewBox=\"0 0 8 8\">\n<path d=\"M0 0 C 1 1 2 2 3 3\" fill=\"#000000\"/>\n<rect x=\"1\" y=\"1\" width=\"2\" height=\"2\" fill=\"#ff9d00\"/>\n</svg>";

    let translation = translate(svg).expect("a panel with a hole");

    assert_eq!(translation.document, "rect 1 1 2 2 #ff9d00\n");
    assert_eq!(translation.skipped.len(), 1);
    assert_eq!(translation.skipped[0].line, 2);
    assert!(translation.skipped[0].reason.contains("`C`"));
}

#[test]
fn a_panel_too_large_for_the_format_is_refused() {
    let error = translate(&polylines(2000)).expect_err("a panel over the budget");

    let over = crate::core::document::MAX_OPS / 4 + 1;
    assert!(
        error
            .problem
            .contains(&crate::core::document::MAX_OPS.to_string())
    );
    assert_eq!(error.line, over + 1);
}

#[test]
fn a_panel_of_the_whole_budget_is_written() {
    let svg = polylines(crate::core::document::MAX_OPS / 4);

    let document = document(&svg);

    assert_eq!(document.lines().count(), crate::core::document::MAX_OPS);
    assert!(crate::core::document::parse(&document).is_ok());
}

#[test]
fn every_shape_left_out_is_named_in_the_report() {
    let svg = "<svg viewBox=\"0 0 8 8\">\n<circle cx=\"4\" cy=\"4\" r=\"2\"/>\n<path d=\"M0 0 C 1 1 2 2 3 3\" fill=\"#000000\"/>\n</svg>";

    let translation = translate(svg).expect("an export with holes");

    assert_eq!(reasons(&translation).len(), 2);
    assert!(reasons(&translation)[0].contains("circle"));
    assert!(reasons(&translation)[1].contains("`C`"));
}

/// An SVG of polylines, one per line, each writing four operations.
fn polylines(count: usize) -> String {
    let mut svg = String::from("<svg viewBox=\"0 0 80 48\">\n");
    for _ in 0..count {
        svg.push_str(
            "<polyline points=\"0,0 1,1 2,2 3,3 4,4\" fill=\"none\" stroke=\"#ffffff\"/>\n",
        );
    }
    svg.push_str("</svg>");
    svg
}

#[test]
fn a_translucent_shape_is_reported_and_the_panel_still_loads() {
    let svg = r##"<svg viewBox="0 0 10 10"><rect x="0" y="0" width="4" height="4" fill="#ff0000" opacity="0.5"/></svg>"##;

    let translation = translate(svg).expect("a translated export");

    assert_eq!(translation.document, "");
    assert_eq!(translation.skipped.len(), 1);
    assert!(
        translation.skipped[0].reason.contains("translucent"),
        "the reason was {}",
        translation.skipped[0].reason
    );
}
