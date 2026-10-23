//! A panel: the scene it draws, the flags it holds, and the keys it answers.
//!
//! The panel is the content's own state. The session is about the screen's
//! authority - the lease and the focus - and the flags and terminals are about
//! what is on screen, so the two are not merged (ADR 0009, ADR 0017).

use std::fmt;

use super::binding::Binding;
use super::event::Key;
use super::scene::{Element, Scene};
use super::term::Term;
use super::typeface::Setting;

mod flags;
#[cfg(test)]
mod font_tests;
mod keys;
#[cfg(test)]
mod keys_tests;
mod terms;
#[cfg(test)]
mod terms_tests;
#[cfg(test)]
mod tests;

pub use flags::Flag;
pub use keys::Outcome;
pub use terms::Feed;

use flags::{holds, value_of};
use keys::action_for;

/// Why a panel refused to be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelError {
    /// A binding or a condition named a flag that was not declared.
    UndeclaredFlag { flag: String },
    /// A feed, binding, or input directive named a terminal that was not declared.
    UndeclaredTerm { term: String },
    /// Two bindings were for the same key, and the second is shadowed.
    DuplicateKey { key: Key },
}

impl fmt::Display for PanelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelError::UndeclaredFlag { flag } => {
                write!(f, "the flag `{flag}` is not declared")
            }
            PanelError::UndeclaredTerm { term } => {
                write!(f, "the terminal `{term}` is not declared")
            }
            PanelError::DuplicateKey { key } => {
                write!(f, "the key `{}` is bound twice", keys::name_of(*key))
            }
        }
    }
}

/// A panel: a scene, the flags it holds, the terminals it updates, and the
/// bindings it answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    scene: Scene,
    flags: Vec<Flag>,
    bindings: Vec<Binding>,
    terms: Vec<Term>,
    feeds: Vec<Feed>,
    focused: Option<String>,
}

impl Panel {
    /// The panel of `scene`, `flags` and `bindings`.
    pub fn new(scene: Scene, flags: Vec<Flag>, bindings: Vec<Binding>) -> Result<Self, PanelError> {
        flags::check(&flags, &bindings, scene.elements())?;
        if let Some(key) = keys::shadowed(&bindings) {
            return Err(PanelError::DuplicateKey { key });
        }
        Ok(Self {
            scene,
            flags,
            bindings,
            terms: Vec::new(),
            feeds: Vec::new(),
            focused: None,
        })
    }

    /// Attach terminal viewports, live feeds, and optional initial terminal focus.
    pub fn with_terminals(
        mut self,
        terms: Vec<Term>,
        feeds: Vec<Feed>,
        focused: Option<String>,
    ) -> Result<Self, PanelError> {
        terms::check(
            &self.flags,
            &self.bindings,
            &terms,
            &feeds,
            focused.as_deref(),
        )?;
        self.terms = terms;
        self.feeds = feeds;
        self.focused = focused;
        Ok(self)
    }

    pub fn set_font(&mut self, font: Option<Setting>) {
        self.scene.set_font(font);
    }

    pub fn font(&self) -> Option<&Setting> {
        self.scene.font()
    }

    pub fn duration(&self) -> u64 {
        self.scene.duration()
    }

    pub fn is_looping(&self) -> bool {
        self.scene
            .satisfying(|condition| holds(&self.flags, condition))
            .is_looping()
    }

    pub fn elements(&self) -> &[Element] {
        self.scene.elements()
    }

    /// The scene to draw at `now` milliseconds, including active terminals.
    pub fn scene(&self, now: u64) -> Scene {
        let mut scene = self
            .scene
            .satisfying(|condition| holds(&self.flags, condition))
            .at(now);
        terms::paint_into(
            &mut scene,
            &self.terms,
            &self.flags,
            self.focused.as_deref(),
        );
        scene
    }

    pub fn flag(&self, name: &str) -> Option<bool> {
        value_of(&self.flags, name)
    }

    pub fn flags(&self) -> &[Flag] {
        &self.flags
    }

    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    pub fn terms(&self) -> &[Term] {
        &self.terms
    }

    pub fn feeds(&self) -> &[Feed] {
        &self.feeds
    }

    pub fn focused_term(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    /// Replace the text in the named terminal.
    pub fn write(&mut self, term: &str, text: &str) -> bool {
        terms::write(&mut self.terms, term, text)
    }

    /// Append text to the named terminal.
    pub fn append(&mut self, term: &str, text: &str) -> bool {
        terms::append(&mut self.terms, term, text)
    }

    /// The `(term, command)` feeds due to run at `now`.
    pub fn due_feeds(&self, last: Option<u64>, now: u64) -> Vec<(String, String)> {
        terms::due_feeds(&self.feeds, last, now)
    }

    /// The key arrives, and the panel says what it did.
    pub fn key(&mut self, key: Key) -> Outcome {
        if let Some(outcome) = terms::route_key(&mut self.terms, &mut self.focused, key) {
            return outcome;
        }
        let Some(action) = action_for(&self.bindings, key) else {
            return Outcome::Unchanged;
        };
        keys::apply(action, &mut self.flags, &self.terms, &mut self.focused)
    }
}
