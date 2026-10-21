//! The path directive: commands, then paints.
//!
//! A path is written the way SVG writes one, in absolute coordinates: `M`
//! starts a shape, `L`, `C` and `Q` draw into it, and `Z` closes it. The
//! paints come at the end: `fill` and `stroke` name a colour each.
//!
//! A path must state at least one paint. The document has no current colour
//! for a path to inherit, and a shape nobody can see is a shape nobody can
//! fix.

use super::super::motion::Motion;
use super::super::path::{Path, Segment};
use super::super::scene::Element;
use super::super::shape::Shape;
use super::super::syntax;
use std::str::SplitWhitespace;

pub(crate) fn parse(source: &str) -> Result<Element, String> {
    let rest = source.strip_prefix("path").unwrap_or(source);
    let mut words = rest.split_whitespace();
    let mut segments: Vec<Segment> = Vec::new();
    let mut fill = None;
    let mut stroke = None;
    let mut open = false;

    while let Some(word) = words.next() {
        match word {
            "M" => {
                segments.push(Segment::Move {
                    to: syntax::point(&mut words)?,
                });
                open = true;
            }
            "L" | "C" | "Q" => {
                if !open {
                    return Err("a path must start with `M`".to_string());
                }
                segments.push(drawing(word, &mut words)?);
            }
            "Z" => {
                if !open {
                    return Err("a `Z` must follow a shape".to_string());
                }
                segments.push(Segment::Close);
                open = false;
            }
            "fill" | "stroke" => {
                let colour = syntax::colour(&mut words)?;
                let slot = if word == "fill" {
                    &mut fill
                } else {
                    &mut stroke
                };
                if slot.replace(colour).is_some() {
                    return Err(format!("a path states `{word}` once"));
                }
            }
            other => return Err(format!("unknown path command `{other}`")),
        }
    }

    if segments.is_empty() {
        return Err("a path needs at least one segment".to_string());
    }
    if fill.is_none() && stroke.is_none() {
        return Err("a path must state a fill or a stroke".to_string());
    }
    Element::new(
        Shape::Path {
            path: Path { segments },
            fill,
            stroke,
        },
        Motion::none(),
    )
}

/// The segment one drawing command makes.
fn drawing(command: &str, words: &mut SplitWhitespace<'_>) -> Result<Segment, String> {
    match command {
        "L" => Ok(Segment::Line {
            to: syntax::point(words)?,
        }),
        "C" => {
            let control1 = syntax::point(words)?;
            let control2 = syntax::point(words)?;
            let to = syntax::point(words)?;
            Ok(Segment::Cubic {
                control1,
                control2,
                to,
            })
        }
        _ => {
            let control = syntax::point(words)?;
            let to = syntax::point(words)?;
            Ok(Segment::Quadratic { control, to })
        }
    }
}
