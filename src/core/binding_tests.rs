//! The keys a panel answers: what a binding keeps, and what it refuses.

mod tests {
    use crate::core::binding::{Action, BindError, Binding, RESERVED_ESCAPE};
    use crate::core::event::{Key, KeyCode, Modifiers};

    #[test]
    fn a_binding_keeps_its_key_and_its_action() {
        let key = Key::plain(KeyCode::Char('x'));
        let action = Action::Set {
            flag: "shown".to_string(),
            value: true,
        };

        let binding = Binding::new(key, action.clone()).expect("a binding");

        assert_eq!(binding.key(), key);
        assert_eq!(binding.action(), &action);
    }

    #[test]
    fn a_binding_for_the_escape_chord_is_refused_and_named() {
        let error = Binding::new(RESERVED_ESCAPE, Action::Quit).expect_err("the reserved chord");

        assert_eq!(error, BindError::ReservedEscape);
        assert!(error.to_string().contains("escape"));
    }

    #[test]
    fn the_reserved_escape_chord_is_the_one_the_session_keeps() {
        assert_eq!(
            RESERVED_ESCAPE,
            Key::new(KeyCode::Escape, Modifiers::CTRL_ALT)
        );
    }
}
