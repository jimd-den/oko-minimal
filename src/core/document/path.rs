//! The path directive: commands, then paints.
//!
//! A path is written the way SVG writes one, in absolute coordinates: `M`
//! starts a shape, `L`, `C` and `Q` draw into it, and `Z` closes it. The
//! paints come at the end: `fill` and `stroke` name a colour each.
//!
//! A path must state at least one paint. The document has no current colour
//! for a path to inherit, and a shape nobody can see is a shape nobody can
//! fix.

use super::super::path::{Path, Segment};
use super::super::shape::Shape;
use super::words::Words;

/// The segments and paints a path states, read from the next words.
pub(crate) fn read(words: &mut Words) -> Result<Shape, String> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut fill = None;
    let mut stroke = None;
    let mut open = false;

    while let Some(word) = words.peek(0) {
        match word {
            "M" => {
                words.next();
                segments.push(Segment::Move { to: words.point()? });
                open = true;
            }
            "L" | "C" | "Q" => {
                if !open {
                    return Err("a path must start with `M`".to_string());
                }
                words.next();
                segments.push(drawing(word, words)?);
            }
            "Z" => {
                if !open {
                    return Err("a `Z` must follow a shape".to_string());
                }
                words.next();
                segments.push(Segment::Close);
                open = false;
            }
            "fill" | "stroke" => {
                words.next();
                let colour = words.colour()?;
                let slot = if word == "fill" {
                    &mut fill
                } else {
                    &mut stroke
                };
                if slot.replace(colour).is_some() {
                    return Err(format!("a path states `{word}` once"));
                }
            }
            other if super::words::is_keyword(other) => break,
            other => return Err(format!("unknown path command `{other}`")),
        }
    }

    if segments.is_empty() {
        return Err("a path needs at least one segment".to_string());
    }
    if fill.is_none() && stroke.is_none() {
        return Err("a path must state a fill or a stroke".to_string());
    }

    Ok(Shape::Path {
        path: Path { segments },
        fill,
        stroke,
    })
}

/// The segment one drawing command makes.
fn drawing(command: &str, words: &mut Words) -> Result<Segment, String> {
    match command {
        "L" => Ok(Segment::Line { to: words.point()? }),
        "C" => {
            let control1 = words.point()?;
            let control2 = words.point()?;
            let to = words.point()?;
            Ok(Segment::Cubic {
                control1,
                control2,
                to,
            })
        }
        _ => {
            let control = words.point()?;
            let to = words.point()?;
            Ok(Segment::Quadratic { control, to })
        }
    }
}
