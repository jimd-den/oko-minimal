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
use super::super::scene::Op;
use super::super::syntax;
use std::str::SplitWhitespace;

pub(crate) fn parse(source: &str) -> Result<Op, String> {
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
    Ok(Op::Path {
        path: Path { segments },
        fill,
        stroke,
    })
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rgba};

    #[test]
    fn a_path_directive_becomes_a_path_operation() {
        let op = parse("path M 10 10 C 20 0 30 0 30 10 Z fill #ff9d00").expect("a path");

        assert_eq!(
            op,
            Op::Path {
                path: Path {
                    segments: vec![
                        Segment::Move {
                            to: Point::new(10, 10),
                        },
                        Segment::Cubic {
                            control1: Point::new(20, 0),
                            control2: Point::new(30, 0),
                            to: Point::new(30, 10),
                        },
                        Segment::Close,
                    ],
                },
                fill: Some(Rgba::rgb(255, 157, 0)),
                stroke: None,
            }
        );
    }

    #[test]
    fn a_path_may_state_a_stroke_and_no_fill() {
        let op = parse("path M 0 0 L 10 0 stroke #1f6feb").expect("a path");

        let Op::Path { fill, stroke, .. } = op else {
            panic!("a path is a path operation");
        };
        assert_eq!(fill, None);
        assert_eq!(stroke, Some(Rgba::rgb(31, 111, 235)));
    }

    #[test]
    fn a_path_that_states_a_paint_twice_is_refused() {
        let error = parse("path M 0 0 L 10 0 fill #ff0000 fill #00ff00").expect_err("two fills");

        assert!(error.contains("fill"));
    }

    #[test]
    fn a_quadratic_command_makes_a_quadratic_segment() {
        let op = parse("path M 0 0 Q 5 10 10 0 fill #ff9d00").expect("a path");

        let Op::Path { path, .. } = op else {
            panic!("a path is a path operation");
        };
        assert_eq!(
            path.segments[1],
            Segment::Quadratic {
                control: Point::new(5, 10),
                to: Point::new(10, 0),
            }
        );
    }

    #[test]
    fn an_unknown_path_command_is_refused() {
        let error = parse("path M 0 0 A 5 5 0 0 1 10 10 fill #ff0000").expect_err("an arc");

        assert!(error.contains('A'));
    }

    #[test]
    fn a_path_without_a_paint_is_refused() {
        let error = parse("path M 0 0 L 10 10").expect_err("a path with no paint");

        assert!(error.contains("fill"));
    }

    #[test]
    fn a_path_without_segments_is_refused() {
        let error = parse("path fill #ff0000").expect_err("a path with no shape");

        assert!(error.contains("segment"));
    }

    #[test]
    fn a_path_that_does_not_start_with_a_move_is_refused() {
        let error = parse("path L 10 10 fill #ff0000").expect_err("a path with no start");

        assert!(error.contains('M'));
    }

    #[test]
    fn a_close_without_a_shape_is_refused() {
        let error = parse("path Z fill #ff0000").expect_err("a path that closes nothing");

        assert!(error.contains('Z'));
    }
}
