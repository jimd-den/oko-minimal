//! The keys a panel answers: what a key does, and what a panel refuses.

use crate::core::binding::{Action, Binding};
use crate::core::event::{Key, KeyCode};
use crate::core::panel::Panel;
use crate::core::scene::Scene;

/// A binding of a key to the end of the program.
fn binding(key: char) -> Binding {
    Binding::new(Key::plain(KeyCode::Char(key)), Action::Quit).expect("a binding")
}

#[test]
fn two_bindings_for_one_key_are_refused() {
    let error = Panel::new(Scene::new(), vec![], vec![binding('x'), binding('x')])
        .expect_err("two bindings for one key");

    assert!(
        error.to_string().contains('x'),
        "the refusal names the key: {error}"
    );
}

#[test]
fn two_bindings_for_two_keys_are_kept() {
    let panel = Panel::new(Scene::new(), vec![], vec![binding('x'), binding('y')])
        .expect("two bindings for two keys");

    assert_eq!(panel.bindings().len(), 2);
}
