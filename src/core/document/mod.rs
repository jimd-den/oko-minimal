//! Panel documents: the text a person edits to change the screen.
//!
//! A document is a sequence of directives, one per line, with `#` starting a
//! comment line and blank lines ignored. It is the hot-edited half of the
//! system: read the file, parse it, render it. A good document replaces the last
//! good scene atomically; a bad one changes nothing and reports every bad line
//! at once.
//!
//! This module owns the loop, the operation budget, and the error list. What a
//! single directive means lives in `directive`; what a word means lives in
//! `core::syntax`. The parser is total: no input makes it panic, and this is the
//! only text that arrives from outside the program.

mod directive;
#[cfg(test)]
mod directive_tests;
mod error;
mod fields;
#[cfg(test)]
mod font_tests;
mod motion;
#[cfg(test)]
mod motion_tests;
mod panel;
#[cfg(test)]
mod panel_tests;
mod path;
#[cfg(test)]
mod path_tests;
#[cfg(test)]
mod tests;
mod words;
#[cfg(test)]
mod words_tests;

pub use error::ParseError;

use super::binding::Action;
use super::event::Key;
use super::panel::{Flag, Panel, PanelError};
use super::scene::{Condition, Scene};
use directive::Addition;

/// A document larger than this is a mistake, not a panel. Four thousand
/// operations is roughly a hundred times the richest screen in the SRD.
pub const MAX_OPS: usize = 4096;

pub fn parse(text: &str) -> Result<Panel, Vec<ParseError>> {
    let mut scene = Scene::new();
    let mut flags: Vec<Flag> = Vec::new();
    let mut bindings = Vec::new();
    let mut errors = Vec::new();

    // Where each flag was used, so that a refusal can name the line.
    let mut uses: Vec<(String, usize)> = Vec::new();
    // The keys that are bound, and the lines they were bound on, so that
    // a refusal can name the line of the second binding.
    let mut bound: Vec<(Key, usize)> = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        let source = line.trim();
        if source.is_empty() || source.starts_with('#') {
            continue;
        }

        match directive::parse(source) {
            Ok(Addition::Shape(element)) => {
                if scene.len() < MAX_OPS {
                    if let Condition::Flag(name) = &element.condition {
                        uses.push((name.clone(), number));
                    }
                    scene.push(element);
                } else {
                    errors.push(ParseError::new(
                        number,
                        format!("a document may hold at most {MAX_OPS} operations"),
                    ));
                    break;
                }
            }
            Ok(Addition::Flag(flag)) => {
                if flags.iter().any(|declared| declared.name() == flag.name()) {
                    errors.push(ParseError::new(
                        number,
                        format!("the flag `{}` is declared twice", flag.name()),
                    ));
                } else {
                    flags.push(flag);
                }
            }
            Ok(Addition::Font(setting)) => {
                scene.set_font(Some(setting));
            }
            Ok(Addition::Binding(binding)) => {
                if let Action::Set { flag, .. } | Action::Toggle { flag } = binding.action() {
                    uses.push((flag.clone(), number));
                }
                bound.push((binding.key(), number));
                bindings.push(binding);
            }
            Err(problem) => errors.push(ParseError::new(number, problem)),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    match Panel::new(scene, flags, bindings) {
        Ok(panel) => Ok(panel),
        Err(problem) => {
            let line = line_of(&problem, &uses, &bound);
            Err(vec![ParseError::new(line, problem.to_string())])
        }
    }
}

/// The line a refusal is about.
///
/// A flag that was not declared is refused at the line that used it, and a
/// key that is bound twice is refused at the line that bound it again.
fn line_of(problem: &PanelError, uses: &[(String, usize)], bound: &[(Key, usize)]) -> usize {
    match problem {
        PanelError::UndeclaredFlag { flag } => uses
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, line)| *line)
            .unwrap_or(1),
        PanelError::DuplicateKey { key } => bound
            .iter()
            .rev()
            .find(|(bound, _)| bound == key)
            .map(|(_, line)| *line)
            .unwrap_or(1),
    }
}
