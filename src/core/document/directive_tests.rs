//! One line of a document, and the refusals it can earn.
//!
//! These are the document's refusals: a directive it does not know, a
//! colour with too few digits, a coordinate out of range, a word where the
//! line should end, and a character the built-in font does not have.

mod tests {
    use super::super::directive::parse;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::shape::Shape;

    #[test]
    fn an_unknown_directive_is_reported() {
        let error = parse("sparkle 1 2").expect_err("an unknown directive");

        assert!(error.contains("sparkle"));
    }

    #[test]
    fn a_colour_with_three_digits_is_reported() {
        let error = parse("clear #fff").expect_err("a short colour");

        assert!(error.contains("#fff"));
    }

    #[test]
    fn a_coordinate_beyond_the_limit_is_reported() {
        let error = parse("rect 40000 0 1 1 #000000").expect_err("a coordinate out of range");

        assert!(error.contains("32767"));
    }

    #[test]
    fn a_trailing_word_is_reported() {
        let error = parse("clear #000000 please").expect_err("a trailing word");

        assert!(error.contains("please"));
    }

    #[test]
    fn a_text_directive_becomes_a_text_operation() {
        let op = parse("text 2 3 \"A B\" 2 #ff9d00").expect("a label");

        assert_eq!(
            op.shape,
            Shape::Text {
                at: Point::new(2, 3),
                text: "A B".to_string(),
                scale: 2,
                color: Rgba::rgb(255, 157, 0),
            }
        );
    }

    #[test]
    fn an_unclosed_label_is_reported() {
        let error = parse("text 0 0 \"OPEN 1 #000000").expect_err("an unclosed label");

        assert!(error.contains("closing quote"));
    }

    #[test]
    fn a_character_outside_the_font_is_reported() {
        let error = parse("text 0 0 \"A~\" 1 #000000").expect_err("an unknown character");

        assert!(error.contains("`~`"));
    }

    #[test]
    fn a_scale_of_zero_is_reported() {
        let error = parse("text 0 0 \"A\" 0 #000000").expect_err("a zero scale");

        assert!(error.contains("1..="));
    }

    #[test]
    fn a_document_describes_a_scene() {
        let op = parse("rect 2 2 10 4 #ff9d00").expect("a rectangle");

        assert_eq!(
            op.shape,
            Shape::Rect {
                area: Rect::new(2, 2, 10, 4),
                color: Rgba::rgb(255, 157, 0),
            }
        );
    }
}
