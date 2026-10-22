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
mod motion;
#[cfg(test)]
mod motion_tests;
mod panel;
#[cfg(test)]
mod panel_tests;
mod path;
#[cfg(test)]
mod path_tests;
mod words;
#[cfg(test)]
mod words_tests;

pub use error::ParseError;

use super::binding::Action;
use super::panel::{Flag, Panel, PanelError};
use super::scene::{Condition, Scene};
use super::typeface::Setting;
use directive::Addition;

/// A document larger than this is a mistake, not a panel. Four thousand
/// operations is roughly a hundred times the richest screen in the SRD.
pub const MAX_OPS: usize = 4096;

pub fn parse(text: &str) -> Result<Panel, Vec<ParseError>> {
    let mut scene = Scene::new();
    let mut flags: Vec<Flag> = Vec::new();
    let mut bindings = Vec::new();
    let mut font: Option<Setting> = None;
    let mut errors = Vec::new();

    // Where each flag was used, so that a refusal can name the line.
    let mut uses: Vec<(String, usize)> = Vec::new();

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
                font = Some(setting);
            }
            Ok(Addition::Binding(binding)) => {
                if let Action::Set { flag, .. } = binding.action() {
                    uses.push((flag.clone(), number));
                }
                bindings.push(binding);
            }
            Err(problem) => errors.push(ParseError::new(number, problem)),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    match Panel::new(scene, flags, bindings) {
        Ok(mut panel) => {
            panel.set_font(font);
            Ok(panel)
        }
        Err(problem) => {
            let PanelError::UndeclaredFlag { flag } = &problem;
            let line = uses
                .iter()
                .find(|(name, _)| name == flag)
                .map(|(_, line)| *line)
                .unwrap_or(1);
            Err(vec![ParseError::new(line, problem.to_string())])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::raster;
    use crate::core::shape::Shape;

    #[test]
    fn a_document_describes_a_scene() {
        let text = "\
# a panel
clear #05070a
rect 2 2 10 4 #ff9d00
line 0 0 9 4 #1f6feb";

        let panel = parse(text).expect("a valid document");

        let shapes: Vec<Shape> = panel
            .scene(0)
            .elements()
            .iter()
            .map(|e| e.shape.clone())
            .collect();
        assert_eq!(
            shapes,
            [
                Shape::Clear(Rgba::rgb(5, 7, 10)),
                Shape::Rect {
                    area: Rect::new(2, 2, 10, 4),
                    color: Rgba::rgb(255, 157, 0),
                },
                Shape::Line {
                    from: Point::new(0, 0),
                    to: Point::new(9, 4),
                    color: Rgba::rgb(31, 111, 235),
                },
            ]
        );
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let panel = parse("\n# a comment\n\nclear #000000\n").expect("a valid document");

        assert_eq!(panel.scene(0).len(), 1);
    }

    #[test]
    fn every_bad_line_is_reported_at_once() {
        let errors = parse("rect 0 0 1 1 #fff\nline 0 0 1 1 #zzzzzz").expect_err("two errors");

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line, 1);
        assert_eq!(errors[1].line, 2);
    }

    #[test]
    fn a_path_in_a_document_reaches_the_frame() {
        let text = "path M 2 2 L 10 2 L 2 10 Z fill #ff9d00";

        let panel = parse(text).expect("a valid document");
        let frame = raster::render(&panel.scene(0), 12, 12).expect("a small frame");

        assert_eq!(
            frame.color_at(Point::new(3, 3)),
            Some(Rgba::rgb(255, 157, 0))
        );
        assert_eq!(frame.color_at(Point::new(9, 9)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_document_may_not_exceed_its_operation_budget() {
        let text = "clear #000000\n".repeat(MAX_OPS + 1);

        let errors = parse(&text).expect_err("a document over budget");

        assert!(errors[0].problem.contains(&MAX_OPS.to_string()));
    }
}
