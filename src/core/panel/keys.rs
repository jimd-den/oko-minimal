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
use super::super::event::Key;

/// What a key did to the panel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing changed: the panel is as it was and needs no repaint.
    Unchanged,
    /// A flag changed: the panel is different and the runtime should repaint.
    Changed,
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
