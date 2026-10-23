//! Tests for terminal viewports in `core::term`.

use super::geom::{Point, Rect, Rgba};
use super::scene::Condition;
use super::shape::Shape;
use super::term::Term;

fn viewport(w: u32, h: u32) -> Term {
    Term::new(
        "bridge",
        Rect::new(10, 20, w, h),
        1,
        Rgba::WHITE,
        Condition::Always,
    )
}

#[test]
fn a_term_wraps_long_lines_to_its_column_budget() {
    let mut term = viewport(30, 24);

    term.write("ABCDEFGHIJ");

    assert_eq!(term.columns(), 5);
    assert_eq!(term.lines(), ["ABCDE", "FGHIJ"]);
}

#[test]
fn a_term_scrolls_to_keep_the_latest_rows_on_screen() {
    let mut term = viewport(60, 16);

    term.write("ONE\nTWO\nTHREE");

    assert_eq!(term.rows(), 2);
    let elements = term.elements(false);
    assert_eq!(elements.len(), 2);
    assert_eq!(
        elements[0].shape,
        Shape::Text {
            at: Point::new(10, 20),
            text: "TWO".to_string(),
            scale: 1,
            color: Rgba::WHITE,
            spacing: 0,
        }
    );
    assert_eq!(
        elements[1].shape,
        Shape::Text {
            at: Point::new(10, 28),
            text: "THREE".to_string(),
            scale: 1,
            color: Rgba::WHITE,
            spacing: 0,
        }
    );
}

#[test]
fn a_focused_term_renders_its_live_prompt_line_at_the_bottom() {
    let mut term = viewport(60, 16);
    term.write("READY");
    assert!(term.push_char('l'));
    assert!(term.push_char('s'));

    let elements = term.elements(true);

    assert_eq!(elements.len(), 2);
    assert_eq!(
        elements[1].shape,
        Shape::Text {
            at: Point::new(10, 28),
            text: "> ls_".to_string(),
            scale: 1,
            color: Rgba::WHITE,
            spacing: 0,
        }
    );
}

#[test]
fn submitting_an_input_line_echoes_and_clears_it() {
    let mut term = viewport(60, 24);
    term.push_char('p');
    term.push_char('s');

    let command = term.submit();

    assert_eq!(command, "ps");
    assert_eq!(term.input(), "");
    assert_eq!(term.lines(), ["> ps"]);
}
