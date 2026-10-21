//! One line of a document, and the refusals it can earn.
//!
//! These are the document's refusals: a directive it does not know, a
//! colour with too few digits, a coordinate out of range, a word where the
//! line should end, and a character the built-in font does not have.

mod tests {
    use super::super::parse;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::shape::Shape;

    #[test]
    fn an_unknown_directive_is_reported() {
        let error = parse("sparkle 1 2").expect_err("an unknown directive");

        assert!(error[0].problem.contains("sparkle"));
    }

    #[test]
    fn a_colour_with_three_digits_is_reported() {
        let error = parse("clear #fff").expect_err("a short colour");

        assert!(error[0].problem.contains("#fff"));
    }

    #[test]
    fn a_coordinate_beyond_the_limit_is_reported() {
        let error = parse("rect 40000 0 1 1 #000000").expect_err("a coordinate out of range");

        assert!(error[0].problem.contains("32767"));
    }

    #[test]
    fn a_trailing_word_is_reported() {
        let error = parse("clear #000000 please").expect_err("a trailing word");

        assert!(error[0].problem.contains("please"));
    }

    #[test]
    fn a_text_directive_becomes_a_text_operation() {
        let panel = parse("text 2 3 \"A B\" 2 #ff9d00").expect("a label");

        assert_eq!(
            panel.scene(0).elements()[0].shape,
            Shape::Text {
                at: Point::new(2, 3),
                text: "A B".to_string(),
                scale: 2,
                color: Rgba::rgb(255, 157, 0),
                spacing: 0,
            }
        );
    }

    #[test]
    fn a_text_may_state_the_space_between_its_letters() {
        let panel = parse("text 0 0 \"A B\" 1 #ff9d00 spacing 2").expect("a spaced label");

        let spacing = match &panel.scene(0).elements()[0].shape {
            Shape::Text { spacing, .. } => *spacing,
            other => panic!("a text is a text: {other:?}"),
        };
        assert_eq!(spacing, 2);
    }

    #[test]
    fn a_text_without_a_spacing_uses_the_font_own_spacing() {
        let panel = parse("text 0 0 \"A B\" 1 #ff9d00").expect("a label");

        let spacing = match &panel.scene(0).elements()[0].shape {
            Shape::Text { spacing, .. } => *spacing,
            other => panic!("a text is a text: {other:?}"),
        };
        assert_eq!(spacing, 0);
    }

    #[test]
    fn an_unclosed_label_is_reported() {
        let error = parse("text 0 0 \"OPEN 1 #000000").expect_err("an unclosed label");

        assert!(error[0].problem.contains("closing quote"));
    }

    #[test]
    fn a_character_outside_the_font_is_reported() {
        let error = parse("text 0 0 \"A~\" 1 #000000").expect_err("an unknown character");

        assert!(error[0].problem.contains("`~`"));
    }

    #[test]
    fn a_scale_of_zero_is_reported() {
        let error = parse("text 0 0 \"A\" 0 #000000").expect_err("a zero scale");

        assert!(error[0].problem.contains("1..="));
    }

    #[test]
    fn a_document_describes_a_scene() {
        let panel = parse("rect 2 2 10 4 #ff9d00").expect("a rectangle");

        assert_eq!(
            panel.scene(0).elements()[0].shape,
            Shape::Rect {
                area: Rect::new(2, 2, 10, 4),
                color: Rgba::rgb(255, 157, 0),
            }
        );
    }
}
