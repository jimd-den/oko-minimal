//! The keys a panel answers, and what a key does to it.
//!
//! A panel's answer to a key is one of three things: nothing changed, a
//! flag changed, or the program should end. A bool would have to carry
//! the last one in its neighbourhood, so the answer is a value of its
//! own.
//!
//! The bindings are read here, and not in the panel: a binding is a key
//! and what it does, and the panel holds the bindings and asks them.

use super::super::binding::{Action, Binding};
use super::super::event::{Key, KeyCode};
use super::super::term::Term;
use super::flags::{self, Flag};
use super::terms;

/// What a key did to the panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changed: the panel is as it was and needs no repaint.
    Unchanged,
    /// A flag or terminal changed: the panel is different and should repaint.
    Changed,
    /// A binding or terminal prompt asked to run a command.
    Run { term: String, command: String },
    /// The binding asked the program to end.
    Quit,
}

/// What the key does, if it is bound.
///
/// The first binding for a key is the one that answers it: the panel
/// holds its bindings in the order they were written.
pub fn action_for(bindings: &[Binding], key: Key) -> Option<Action> {
    bindings
        .iter()
        .find(|binding| binding.key() == key)
        .map(|binding| binding.action().clone())
}

/// The key two bindings share, if two of them share one.
///
/// A panel with two bindings for one key is a panel whose second binding
/// can never fire: the first one answers the key. The panel refuses to be
/// built rather than keep a binding that is shadowed.
pub fn shadowed(bindings: &[Binding]) -> Option<Key> {
    bindings
        .iter()
        .enumerate()
        .find(|(index, binding)| {
            bindings[..*index]
                .iter()
                .any(|earlier| earlier.key() == binding.key())
        })
        .map(|(_, binding)| binding.key())
}

/// The name a key is called by, for a message that names it.
///
/// The names are the ones a document writes: `on key enter` binds the
/// enter key, and a character is its own name.
pub fn name_of(key: Key) -> String {
    match key.code {
        KeyCode::Char(character) => character.to_string(),
        KeyCode::Enter => "enter".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Backspace => "backspace".to_string(),
        KeyCode::Escape => "escape".to_string(),
        KeyCode::Up => "up".to_string(),
        KeyCode::Down => "down".to_string(),
        KeyCode::Left => "left".to_string(),
        KeyCode::Right => "right".to_string(),
        KeyCode::Function(number) => format!("f{number}"),
    }
}

/// Perform `action` on the panel's state and return the resulting outcome.
pub fn apply(
    action: Action,
    flags: &mut [Flag],
    terms: &[Term],
    focused: &mut Option<String>,
) -> Outcome {
    match action {
        Action::Quit => Outcome::Quit,
        Action::Set { flag, value } => {
            if flags::set(flags, &flag, value) {
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        Action::Toggle { flag } => {
            if flags::toggle(flags, &flag) {
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
        Action::Run { term, command } => Outcome::Run { term, command },
        Action::Focus { term } => {
            if terms::declares(terms, &term) {
                *focused = Some(term);
                Outcome::Changed
            } else {
                Outcome::Unchanged
            }
        }
    }
}
