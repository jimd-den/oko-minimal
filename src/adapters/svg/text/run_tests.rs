//! The label a run of text becomes, seen from the outside.

#[cfg(test)]
mod tests {
    use super::super::run::{Label, label, scale};
    use crate::core::font;

    #[test]
    fn a_twenty_one_pixel_label_is_scale_three() {
        assert_eq!(scale(21.0), 3);
    }

    #[test]
    fn a_scale_above_the_limit_is_capped() {
        assert_eq!(scale(200.0), font::MAX_SCALE);
    }

    #[test]
    fn a_label_trims_and_collapses_its_whitespace() {
        assert_eq!(
            label("  A   B ", 1),
            Ok(Some(Label::Text("A B".to_string())))
        );
    }

    #[test]
    fn a_run_of_whitespace_is_no_label() {
        assert_eq!(label("   ", 1), Ok(None));
    }

    #[test]
    fn a_character_outside_the_font_is_left_out() {
        assert_eq!(
            label("A~", 3),
            Ok(Some(Label::Undrawable(
                "the character `~` is not in the built-in font".to_string()
            )))
        );
    }

    #[test]
    fn a_label_with_a_double_quote_is_refused() {
        let error = label("A\"B", 1).expect_err("a quote");

        assert!(error.problem.contains("double quote"));
    }
}
