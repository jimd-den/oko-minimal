//! The text rules, seen from the outside.
//!
//! Each test here is a whole SVG in and a whole document out, because that is
//! the seam the ADR describes: an `<text>` element becomes one text directive.

use crate::adapters::svg::translate;

/// The document a translation produced.
fn document(svg: &str) -> String {
    translate(svg).expect("a translated export").document
}

#[test]
fn a_span_keeps_the_position_of_its_text_and_adds_its_baseline() {
    let svg = "<svg><text x=\"6\" y=\"10\" font-size=\"14\"><tspan x=\"6\" dy=\"14\">SECOND</tspan></text></svg>";

    assert_eq!(document(svg), "text 6 10 \"SECOND\" 2 #000000\n");
}

#[test]
fn a_span_without_a_position_inherits_it() {
    let svg = "<svg><text x=\"6\" y=\"10\"><tspan dy=\"14\">SECOND</tspan></text></svg>";

    assert_eq!(document(svg), "text 6 17 \"SECOND\" 1 #000000\n");
}

#[test]
fn a_span_keeps_the_colour_of_its_text() {
    let svg = "<svg><text x=\"6\" y=\"10\" fill=\"#ff9d00\"><tspan>A</tspan></text></svg>";

    assert_eq!(document(svg), "text 6 3 \"A\" 1 #ff9d00\n");
}

#[test]
fn a_size_in_a_style_attribute_is_read() {
    let svg = "<svg><text x=\"1\" y=\"16\" style=\"font-size: 14px\">A</text></svg>";

    assert_eq!(document(svg), "text 1 2 \"A\" 2 #000000\n");
}

#[test]
fn a_text_without_a_size_is_scale_one() {
    let svg = "<svg><text x=\"1\" y=\"9\">A</text></svg>";

    assert_eq!(document(svg), "text 1 2 \"A\" 1 #000000\n");
}

#[test]
fn a_size_above_the_scale_limit_is_capped() {
    let svg = "<svg><text x=\"1\" y=\"60\" font-size=\"200\">A</text></svg>";

    assert_eq!(document(svg), "text 1 4 \"A\" 8 #000000\n");
}

#[test]
fn a_text_with_no_fill_is_not_drawn() {
    let svg = "<svg><text x=\"1\" y=\"2\" fill=\"none\">A</text></svg>";

    assert_eq!(document(svg), "");
}

/// A character the built-in font does not have is a shape left out, not a
/// file refused: the panel around it still loads and the report names it.
#[test]
fn a_character_outside_the_font_is_left_out_and_named() {
    let translation =
        translate("<svg><text x=\"1\" y=\"2\">A~</text></svg>").expect("a panel with a hole");

    assert_eq!(translation.document, "");
    assert_eq!(translation.skipped.len(), 1);
    assert!(translation.skipped[0].reason.contains("`~`"));
}

#[test]
fn a_label_with_a_double_quote_is_refused() {
    let error = translate("<svg><text x=\"1\" y=\"2\">A\"B</text></svg>").expect_err("a quote");

    assert!(error.problem.contains("double quote"));
}

#[test]
fn a_label_collapses_its_whitespace() {
    let svg = "<svg><text x=\"1\" y=\"9\">  A   B  </text></svg>";

    assert_eq!(document(svg), "text 1 2 \"A B\" 1 #000000\n");
}

#[test]
fn a_run_of_whitespace_is_ignored() {
    let svg = "<svg><text x=\"1\" y=\"2\">   </text></svg>";

    assert_eq!(document(svg), "");
}

#[test]
fn lowercase_is_left_for_the_font_to_draw() {
    let svg = "<svg><text x=\"1\" y=\"9\">oko</text></svg>";

    assert_eq!(document(svg), "text 1 2 \"oko\" 1 #000000\n");
}

#[test]
fn text_outside_a_text_element_is_refused() {
    let error = translate("<svg><g>loose</g></svg>").expect_err("loose text");

    assert!(error.problem.contains("`<text>`"));
}

#[test]
fn a_group_translation_moves_its_label() {
    let svg = "<svg><g transform=\"translate(10 3)\"><text x=\"2\" y=\"9\">A</text></g></svg>";

    assert_eq!(document(svg), "text 12 5 \"A\" 1 #000000\n");
}
