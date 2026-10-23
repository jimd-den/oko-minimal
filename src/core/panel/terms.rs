//! The terminal viewports and command feeds a panel holds.
//!
//! A panel may declare named terminal viewports (`Term`) and live feeds
//! (`Feed`), and may focus one terminal for interactive typing. This module
//! checks their names against the panel's declarations, routes keys when a
//! terminal has focus, and lowers visible terminals into the scene.

use super::super::binding::{Action, Binding};
use super::super::event::{Key, KeyCode};
use super::super::scene::{Condition, Scene};
use super::super::term::Term;
use super::PanelError;
use super::flags::{self, Flag};
use super::keys::Outcome;

/// A live command or API feed that populates a named terminal viewport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feed {
    term: String,
    command: String,
    every: Option<u64>,
}

impl Feed {
    pub fn new(term: impl Into<String>, command: impl Into<String>, every: Option<u64>) -> Self {
        Self {
            term: term.into(),
            command: command.into(),
            every,
        }
    }

    pub fn term(&self) -> &str {
        &self.term
    }

    pub fn command(&self) -> &str {
        &self.command
    }

    pub fn every(&self) -> Option<u64> {
        self.every
    }

    /// Whether this feed should run at `now` given the `last` moment it ran.
    pub fn is_due(&self, last: Option<u64>, now: u64) -> bool {
        let Some(previous) = last else {
            return true;
        };
        match self.every {
            Some(period) if period > 0 => (now / period) > (previous / period),
            _ => false,
        }
    }
}

/// Whether the terminals declare the named one.
pub fn declares(terms: &[Term], name: &str) -> bool {
    terms.iter().any(|term| term.name() == name)
}

/// Check that every terminal condition, feed, binding, and focus names a
/// declared flag or terminal.
pub fn check(
    flags: &[Flag],
    bindings: &[Binding],
    terms: &[Term],
    feeds: &[Feed],
    focused: Option<&str>,
) -> Result<(), PanelError> {
    for term in terms {
        if let Condition::Flag(flag) = term.condition()
            && !flags::declares(flags, flag)
        {
            return Err(PanelError::UndeclaredFlag { flag: flag.clone() });
        }
    }
    for feed in feeds {
        if !declares(terms, feed.term()) {
            return Err(PanelError::UndeclaredTerm {
                term: feed.term().to_string(),
            });
        }
    }
    for binding in bindings {
        if let Action::Run { term, .. } | Action::Focus { term } = binding.action()
            && !declares(terms, term)
        {
            return Err(PanelError::UndeclaredTerm { term: term.clone() });
        }
    }
    if let Some(name) = focused
        && !declares(terms, name)
    {
        return Err(PanelError::UndeclaredTerm {
            term: name.to_string(),
        });
    }
    Ok(())
}

/// Replace the text in the named terminal, returning whether it was found.
pub fn write(terms: &mut [Term], name: &str, text: &str) -> bool {
    let Some(term) = terms.iter_mut().find(|term| term.name() == name) else {
        return false;
    };
    term.write(text);
    true
}

/// Append text to the named terminal, returning whether it was found.
pub fn append(terms: &mut [Term], name: &str, text: &str) -> bool {
    let Some(term) = terms.iter_mut().find(|term| term.name() == name) else {
        return false;
    };
    term.append(text);
    true
}

/// The `(term, command)` feeds due at `now` given `last`.
pub fn due_feeds(feeds: &[Feed], last: Option<u64>, now: u64) -> Vec<(String, String)> {
    feeds
        .iter()
        .filter(|feed| feed.is_due(last, now))
        .map(|feed| (feed.term().to_string(), feed.command().to_string()))
        .collect()
}

/// Route a key to the focused terminal, if one is focused.
pub fn route_key(terms: &mut [Term], focused: &mut Option<String>, key: Key) -> Option<Outcome> {
    let name = focused.as_ref()?.clone();
    if key.code == KeyCode::Escape && !key.modifiers.ctrl && !key.modifiers.alt {
        *focused = None;
        return Some(Outcome::Changed);
    }
    let term = terms.iter_mut().find(|term| term.name() == name)?;
    match key.code {
        KeyCode::Backspace => Some(if term.pop_char() {
            Outcome::Changed
        } else {
            Outcome::Unchanged
        }),
        KeyCode::Enter => {
            let command = term.submit();
            if command.trim().is_empty() {
                Some(Outcome::Changed)
            } else {
                Some(Outcome::Run {
                    term: name,
                    command,
                })
            }
        }
        KeyCode::Char(ch) if !key.modifiers.ctrl && !key.modifiers.alt => {
            Some(if term.push_char(ch) {
                Outcome::Changed
            } else {
                Outcome::Unchanged
            })
        }
        _ => Some(Outcome::Unchanged),
    }
}

/// Append the visible text elements of each active terminal into `scene`.
pub fn paint_into(scene: &mut Scene, terms: &[Term], flags: &[Flag], focused: Option<&str>) {
    for term in terms {
        if flags::holds(flags, term.condition()) {
            let is_focused = focused == Some(term.name());
            for element in term.elements(is_focused) {
                scene.push(element);
            }
        }
    }
}
