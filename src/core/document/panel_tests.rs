//! A panel's flags and bindings, as a document writes them.
//!
//! The document is the panel's own text: it declares the flags, it binds the
//! keys, and it says which shape is drawn under which flag.

mod tests {
    use super::super::parse;
    use crate::core::event::{Key, KeyCode};
    use crate::core::panel::Outcome;

    /// The enter key: a key a panel may bind.
    fn enter() -> Key {
        Key::plain(KeyCode::Enter)
    }

    #[test]
    fn a_flag_is_declared_and_starts_false() {
        let panel = parse("flag cursor").expect("a valid document");

        assert_eq!(panel.flag("cursor"), Some(false));
    }

    #[test]
    fn a_flag_may_start_true() {
        let panel = parse("flag running true").expect("a valid document");

        assert_eq!(panel.flag("running"), Some(true));
    }

    #[test]
    fn a_flag_is_true_or_false() {
        let error = parse("flag cursor maybe").expect_err("a flag that is neither");

        assert!(error[0].problem.contains("maybe"), "{:?}", error[0]);
    }

    #[test]
    fn a_shape_is_drawn_only_while_its_flag_is_true() {
        let panel =
            parse("flag cursor\nrect 0 0 2 2 #ff0000 when cursor").expect("a valid document");

        assert_eq!(panel.scene(0).len(), 0, "the flag is false");
    }

    #[test]
    fn a_key_sets_a_flag() {
        let mut panel =
            parse("flag cursor\nrect 0 0 2 2 #ff0000 when cursor\non key enter set cursor true")
                .expect("a valid document");

        let outcome = panel.key(enter());

        assert_eq!(outcome, Outcome::Changed);
        assert_eq!(panel.flag("cursor"), Some(true));
        assert_eq!(panel.scene(0).len(), 1, "the shape is drawn now");
    }

    #[test]
    fn a_key_may_end_the_program() {
        let mut panel = parse("on key escape quit").expect("a valid document");

        let outcome = panel.key(Key::plain(KeyCode::Escape));

        assert_eq!(outcome, Outcome::Quit);
    }

    #[test]
    fn a_key_no_panel_binds_changes_nothing() {
        let mut panel = parse("flag cursor").expect("a valid document");

        let outcome = panel.key(enter());

        assert_eq!(outcome, Outcome::Unchanged);
    }

    #[test]
    fn a_binding_to_an_undeclared_flag_is_refused_with_its_line() {
        let error = parse("on key enter set missing true").expect_err("an undeclared flag");

        assert_eq!(error[0].line, 1);
        assert!(error[0].problem.contains("missing"), "{:?}", error[0]);
    }

    #[test]
    fn a_condition_on_an_undeclared_flag_is_refused_with_its_line() {
        let error = parse("clear #000000\nrect 0 0 2 2 #ff0000 when missing")
            .expect_err("an undeclared flag");

        assert_eq!(error[0].line, 2);
        assert!(error[0].problem.contains("missing"), "{:?}", error[0]);
    }

    #[test]
    fn a_flag_may_not_be_declared_twice() {
        let error = parse("flag cursor\nflag cursor").expect_err("a flag twice");

        assert_eq!(error[0].line, 2);
        assert!(error[0].problem.contains("twice"), "{:?}", error[0]);
    }

    #[test]
    fn a_key_may_be_one_character() {
        let mut panel = parse("flag cursor\non key a set cursor true").expect("a valid document");

        let outcome = panel.key(Key::plain(KeyCode::Char('a')));

        assert_eq!(outcome, Outcome::Changed);
        assert_eq!(panel.flag("cursor"), Some(true));
    }

    #[test]
    fn an_unknown_key_name_is_refused() {
        let error =
            parse("on key flibbertigibbet set cursor true").expect_err("a key that is not named");

        assert!(
            error[0].problem.contains("flibbertigibbet"),
            "{:?}",
            error[0]
        );
    }

    #[test]
    fn the_escape_chord_may_not_be_bound() {
        let error = parse("on key ctrl+alt+escape quit").expect_err("the reserved chord");

        assert!(
            error[0].problem.contains("flibbertigibbet")
                || error[0].problem.contains("ctrl+alt+escape"),
            "{:?}",
            error[0]
        );
    }

    #[test]
    fn a_document_may_name_a_face() {
        let panel = parse("font \"inter\"").expect("a face the program carries");

        assert_eq!(panel.font().map(|font| font.face.name()), Some("inter"));
    }

    #[test]
    fn a_face_the_program_does_not_carry_is_refused_by_name() {
        let error = parse("font \"nonesuch\"").expect_err("a face that is not carried");

        assert!(error[0].problem.contains("nonesuch"), "{:?}", error[0]);
        assert!(
            !error[0].problem.contains('"'),
            "the refusal names the face itself: {:?}",
            error[0]
        );
    }

    #[test]
    fn two_bindings_for_one_key_are_refused() {
        let error = parse("on key enter quit\non key enter quit").expect_err("a key bound twice");

        assert_eq!(error[0].line, 2);
        assert!(error[0].problem.contains("enter"), "{:?}", error[0]);
    }
}
