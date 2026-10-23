//! The flags a panel holds: what is on screen, and what a key may turn on.
//!
//! The flags are the panel's own content state. The session's lease and
//! focus are about the screen's authority, and the two are not merged
//! (ADR 0009), so the flags live here and not among the panel's keys.
//!
//! A flag that is not declared is not a flag: the panel refuses a binding
//! or a condition that names one, and this module is where the names are
//! known. The rules are here so that the panel holds the flags without
//! knowing how a flag is found or turned on.

use super::super::scene::Condition;

/// A named flag and its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    name: String,
    value: bool,
}

impl Flag {
    pub fn new(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }

    /// The name the flag is known by.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The value the flag holds.
    pub fn value(&self) -> bool {
        self.value
    }
}

/// Whether the flags declare the named one.
pub fn declares(flags: &[Flag], name: &str) -> bool {
    flags.iter().any(|flag| flag.name == name)
}

/// The value of the named flag, if it is a flag of these.
pub fn value_of(flags: &[Flag], name: &str) -> Option<bool> {
    flags
        .iter()
        .find(|flag| flag.name == name)
        .map(|flag| flag.value)
}

/// The named flag takes the value, and whether that changed it is said.
///
/// A flag that was not declared is left alone: a panel that has no such
/// flag has nothing to change.
pub fn set(flags: &mut [Flag], name: &str, value: bool) -> bool {
    let Some(flag) = flags.iter_mut().find(|flag| flag.name == name) else {
        return false;
    };
    if flag.value == value {
        return false;
    }
    flag.value = value;
    true
}

/// Flip the named flag between true and false.
pub fn toggle(flags: &mut [Flag], name: &str) -> bool {
    let Some(flag) = flags.iter_mut().find(|flag| flag.name == name) else {
        return false;
    };
    flag.value = !flag.value;
    true
}

/// Whether the condition holds in these flags.
///
/// A condition on a flag that was not declared does not hold: the panel
/// refuses such a condition when it is built, so this is the answer for a
/// panel that was made by hand.
pub fn holds(flags: &[Flag], condition: &Condition) -> bool {
    match condition {
        Condition::Always => true,
        Condition::Flag(name) => value_of(flags, name) == Some(true),
    }
}
