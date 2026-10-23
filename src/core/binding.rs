//! The keys a panel answers, and what they do.
//!
//! A binding is a key and an action. The actions are the whole vocabulary:
//! set a flag true, set a flag false, or quit. A panel may not run a
//! command, read a file, or change anything outside itself, so there is
//! nothing else an action can be (ADR 0009).
//!
//! The reserved escape chord is here because it is a key that may not be
//! bound: it is the way back to the console, and a panel must not be able
//! to take it.

use std::fmt;

use super::event::{Key, KeyCode, Modifiers};

/// The chord that goes back to the console, and that no panel may bind.
pub const RESERVED_ESCAPE: Key = Key::new(KeyCode::Escape, Modifiers::CTRL_ALT);

/// What a binding does when its key arrives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Set the named flag to the value.
    Set { flag: String, value: bool },
    /// Flip the named flag between true and false.
    Toggle { flag: String },
    /// End the program.
    Quit,
}

/// A key and what it does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binding {
    key: Key,
    action: Action,
}

impl Binding {
    /// The binding of `key` to `action`.
    ///
    /// The reserved escape chord is refused here, at the earliest moment a
    /// binding can be refused: an escape binding must never exist.
    pub fn new(key: Key, action: Action) -> Result<Self, BindError> {
        if key == RESERVED_ESCAPE {
            return Err(BindError::ReservedEscape);
        }
        Ok(Self { key, action })
    }

    /// The key the binding waits for.
    pub fn key(&self) -> Key {
        self.key
    }

    /// What the binding does.
    pub fn action(&self) -> &Action {
        &self.action
    }
}

/// Why a binding was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindError {
    /// The reserved escape chord may not be bound.
    ReservedEscape,
}

impl fmt::Display for BindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindError::ReservedEscape => write!(
                f,
                "the reserved escape chord ctrl+alt+escape may not be bound"
            ),
        }
    }
}
