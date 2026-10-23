//! Panel documents: the text a person edits to change the screen.
//!
//! A document is a sequence of directives, one per line, with `#` starting a
//! comment line and blank lines ignored. It is the hot-edited half of the
//! system: read the file, parse it, render it. A good document replaces the last
//! good scene atomically; a bad one changes nothing and reports every bad line
//! at once.

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
mod terms;
#[cfg(test)]
mod terms_tests;
#[cfg(test)]
mod tests;
mod words;
#[cfg(test)]
mod words_tests;

pub use error::ParseError;

use super::binding::Action;
use super::event::Key;
use super::panel::{Feed, Flag, Panel, PanelError};
use super::scene::{Condition, Scene};
use super::term::Term;
use directive::Addition;

/// A document larger than this is a mistake, not a panel. Four thousand
/// operations is roughly a hundred times the richest screen in the SRD.
pub const MAX_OPS: usize = 4096;

pub fn parse(text: &str) -> Result<Panel, Vec<ParseError>> {
    let mut scene = Scene::new();
    let mut flags: Vec<Flag> = Vec::new();
    let mut bindings = Vec::new();
    let mut terms: Vec<Term> = Vec::new();
    let mut feeds: Vec<Feed> = Vec::new();
    let mut focused: Option<String> = None;
    let mut errors = Vec::new();

    let mut uses: Vec<(String, usize)> = Vec::new();
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
                match binding.action() {
                    Action::Set { flag, .. } | Action::Toggle { flag } => {
                        uses.push((flag.clone(), number));
                    }
                    Action::Run { term, .. } | Action::Focus { term } => {
                        uses.push((term.clone(), number));
                    }
                    Action::Quit => {}
                }
                bound.push((binding.key(), number));
                bindings.push(binding);
            }
            Ok(Addition::Term(term)) => {
                if terms.iter().any(|declared| declared.name() == term.name()) {
                    errors.push(ParseError::new(
                        number,
                        format!("the terminal `{}` is declared twice", term.name()),
                    ));
                } else {
                    if let Condition::Flag(name) = term.condition() {
                        uses.push((name.clone(), number));
                    }
                    terms.push(term);
                }
            }
            Ok(Addition::Feed(feed)) => {
                uses.push((feed.term().to_string(), number));
                feeds.push(feed);
            }
            Ok(Addition::Input(name)) => {
                uses.push((name.clone(), number));
                focused = Some(name);
            }
            Err(problem) => errors.push(ParseError::new(number, problem)),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    Panel::new(scene, flags, bindings)
        .and_then(|panel| panel.with_terminals(terms, feeds, focused))
        .map_err(|problem| {
            let line = line_of(&problem, &uses, &bound);
            vec![ParseError::new(line, problem.to_string())]
        })
}

fn line_of(problem: &PanelError, uses: &[(String, usize)], bound: &[(Key, usize)]) -> usize {
    match problem {
        PanelError::UndeclaredFlag { flag } => uses
            .iter()
            .find(|(name, _)| name == flag)
            .map(|(_, line)| *line)
            .unwrap_or(1),
        PanelError::UndeclaredTerm { term } => uses
            .iter()
            .find(|(name, _)| name == term)
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
