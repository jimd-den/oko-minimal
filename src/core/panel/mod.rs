//! A panel: the scene it draws, the flags it holds, and the keys it answers.
//!
//! The panel is the content's own state. The session is about the screen's
//! authority - the lease and the focus - and the flags are about what is on
//! screen, so the two are not merged (ADR 0009).
//!
//! A key arrives, a binding fires, a flag changes, and the shapes whose
//! condition holds are what the panel draws. A panel can do nothing else: it
//! may not run a command, read a file, or change anything outside itself.
//!
//! The flags are their own part, in `flags`: the panel holds them and asks
//! them, and does not know how a flag is found or turned on.

use std::fmt;

use super::binding::{Action, Binding};
use super::event::Key;
use super::scene::{Condition, Element, Scene};
use super::typeface::Setting;

mod flags;
#[cfg(test)]
mod font_tests;
mod keys;
#[cfg(test)]
mod tests;

pub use flags::Flag;
pub use keys::Outcome;

use flags::{declares, holds, value_of};
use keys::action_for;

/// Why a panel refused to be built.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelError {
    /// A binding or a condition named a flag that was not declared.
    UndeclaredFlag { flag: String },
}

impl fmt::Display for PanelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PanelError::UndeclaredFlag { flag } => {
                write!(f, "the flag `{flag}` is not declared")
            }
        }
    }
}

/// A panel: a scene, the flags it holds, and the bindings it answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Panel {
    scene: Scene,
    flags: Vec<Flag>,
    bindings: Vec<Binding>,
}

impl Panel {
    /// The panel of `scene`, `flags` and `bindings`.
    ///
    /// A flag that is not declared is not a flag, so a binding or a condition
    /// that names one is refused here rather than ignored later.
    pub fn new(scene: Scene, flags: Vec<Flag>, bindings: Vec<Binding>) -> Result<Self, PanelError> {
        for binding in &bindings {
            if let Action::Set { flag, .. } = binding.action()
                && !declares(&flags, flag)
            {
                return Err(PanelError::UndeclaredFlag { flag: flag.clone() });
            }
        }
        for element in scene.elements() {
            if let Condition::Flag(name) = &element.condition
                && !declares(&flags, name)
            {
                return Err(PanelError::UndeclaredFlag { flag: name.clone() });
            }
        }
        Ok(Self {
            scene,
            flags,
            bindings,
        })
    }

    /// The font the panel is set in, and the weight it is set at.
    ///
    /// The font belongs to the panel and not to a shape: a panel that
    /// changes its font changes one thing, and the shapes it draws are the
    /// shapes it had. The letterforms are looked up again when the text is
    /// drawn.
    ///
    /// The font is the scene's, because the scene is what the raster walks
    /// (ADR 0014).
    pub fn set_font(&mut self, font: Option<Setting>) {
        self.scene.set_font(font);
    }

    /// The font the panel is set in, if it is set in one.
    pub fn font(&self) -> Option<&Setting> {
        self.scene.font()
    }

    /// How long the panel's shapes keep changing.
    ///
    /// A panel with no motion has length zero. A runtime that asks for the
    /// panel at a moment must not ask it by zero.
    pub fn duration(&self) -> u64 {
        self.scene.duration()
    }

    /// The elements the panel holds, with their motions and their conditions.
    ///
    /// The scene a panel draws is the elements it holds, filtered by the
    /// flags and sampled at a moment. The elements themselves are what the
    /// panel was written with, and a reader of the document may want them.
    pub fn elements(&self) -> &[Element] {
        self.scene.elements()
    }

    /// The scene to draw at `now` milliseconds.
    ///
    /// The shapes whose flag is false are not in it; the rest are as they are
    /// at the moment asked for.
    pub fn scene(&self, now: u64) -> Scene {
        self.scene
            .satisfying(|condition| holds(&self.flags, condition))
            .at(now)
    }

    /// The value of the named flag, if it is a flag of this panel.
    pub fn flag(&self, name: &str) -> Option<bool> {
        value_of(&self.flags, name)
    }

    /// The flags the panel holds.
    pub fn flags(&self) -> &[Flag] {
        &self.flags
    }

    /// The bindings the panel answers.
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// The key arrives, and the panel says what it did.
    ///
    /// The answer is an outcome and not a bool because there are three
    /// answers, not two: nothing happened, a flag changed, or the program
    /// should end. A bool would have to carry the last one in its
    /// neighbourhood.
    pub fn key(&mut self, key: Key) -> Outcome {
        let Some(action) = action_for(&self.bindings, key) else {
            return Outcome::Unchanged;
        };
        match action {
            Action::Quit => Outcome::Quit,
            Action::Set { flag, value } => {
                if flags::set(&mut self.flags, &flag, value) {
                    Outcome::Changed
                } else {
                    Outcome::Unchanged
                }
            }
        }
    }
}
