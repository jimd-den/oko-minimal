//! One line of a document, and what it adds to the scene.
//!
//! The grammar is small on purpose: a directive name, then its arguments. Text
//! is the only directive whose arguments are not whitespace-separated, because
//! a label keeps its spaces.

use super::super::font;
use super::super::geom::Point;
use super::super::scene::Op;
use super::super::syntax;

pub(crate) fn parse(source: &str) -> Result<Op, String> {
    let mut words = source.split_whitespace();
    let name = words.next().unwrap_or_default();

    if name == "text" {
        return text(source);
    }

    let op = match name {
        "clear" => Op::Clear(syntax::colour(&mut words)?),
        "rect" => Op::Rect {
            area: syntax::area(&mut words)?,
            color: syntax::colour(&mut words)?,
        },
        "line" => Op::Line {
            from: syntax::point(&mut words)?,
            to: syntax::point(&mut words)?,
            color: syntax::colour(&mut words)?,
        },
        other => return Err(format!("unknown directive `{other}`")),
    };

    syntax::end(&mut words)?;
    Ok(op)
}

fn text(source: &str) -> Result<Op, String> {
    let rest = source.strip_prefix("text").unwrap_or(source);
    let (x, rest) = syntax::coordinate_prefix(rest)?;
    let (y, rest) = syntax::coordinate_prefix(rest)?;
    let (label, rest) = syntax::quoted(rest)?;
    for character in label.chars() {
        if !font::supports(character) {
            return Err(format!("`{character}` is not in the built-in font"));
        }
    }
    let (scale, rest) = syntax::scale_prefix(rest)?;
    let colour = syntax::colour_rest(rest)?;
    Ok(Op::Text {
        at: Point::new(x, y),
        text: label,
        scale,
        color: colour,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};

    #[test]
    fn an_unknown_directive_is_reported() {
        let error = parse("sparkle 1 2").expect_err("an unknown directive");

        assert!(error.contains("sparkle"));
    }

    #[test]
    fn a_colour_with_three_digits_is_reported() {
        let error = parse("clear #fff").expect_err("a short colour");

        assert!(error.contains("#fff"));
    }

    #[test]
    fn a_coordinate_beyond_the_limit_is_reported() {
        let error = parse("rect 40000 0 1 1 #000000").expect_err("a coordinate out of range");

        assert!(error.contains("32767"));
    }

    #[test]
    fn a_trailing_word_is_reported() {
        let error = parse("clear #000000 please").expect_err("a trailing word");

        assert!(error.contains("please"));
    }

    #[test]
    fn a_text_directive_becomes_a_text_operation() {
        let op = parse("text 2 3 \"A B\" 2 #ff9d00").expect("a label");

        assert_eq!(
            op,
            Op::Text {
                at: Point::new(2, 3),
                text: "A B".to_string(),
                scale: 2,
                color: Rgba::rgb(255, 157, 0),
            }
        );
    }

    #[test]
    fn an_unclosed_label_is_reported() {
        let error = parse("text 0 0 \"OPEN 1 #000000").expect_err("an unclosed label");

        assert!(error.contains("closing quote"));
    }

    #[test]
    fn a_character_outside_the_font_is_reported() {
        let error = parse("text 0 0 \"A~\" 1 #000000").expect_err("an unknown character");

        assert!(error.contains("`~`"));
    }

    #[test]
    fn a_scale_of_zero_is_reported() {
        let error = parse("text 0 0 \"A\" 0 #000000").expect_err("a zero scale");

        assert!(error.contains("1..="));
    }

    #[test]
    fn a_document_describes_a_scene() {
        let op = parse("rect 2 2 10 4 #ff9d00").expect("a rectangle");

        assert_eq!(
            op,
            Op::Rect {
                area: Rect::new(2, 2, 10, 4),
                color: Rgba::rgb(255, 157, 0),
            }
        );
    }
}
