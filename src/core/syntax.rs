//! Reading typed values out of a line's words.
//!
//! The grammar lives in `document`; this module is the layer below it. It
//! knows a coordinate has a range, a size is positive, and a colour is six
//! hexadecimal digits after a `#`. It knows nothing about scenes, directives,
//! or files, which is what makes each rule testable with a string literal.
//!
//! Every function is total: no input makes it panic, and a refusal is a
//! message a person can act on.

use super::font;
use super::geom::{Point, Rect, Rgba};
use std::str::SplitWhitespace;

pub const MAX_COORDINATE: i64 = 32_767;
pub const MAX_SIDE: i64 = 16_384;
pub const MAX_TEXT_CHARS: usize = 128;

pub(crate) fn point(words: &mut SplitWhitespace<'_>) -> Result<Point, String> {
    Ok(Point::new(coordinate(words)?, coordinate(words)?))
}

pub(crate) fn area(words: &mut SplitWhitespace<'_>) -> Result<Rect, String> {
    Ok(Rect::new(
        coordinate(words)?,
        coordinate(words)?,
        extent(words)?,
        extent(words)?,
    ))
}

fn coordinate(words: &mut SplitWhitespace<'_>) -> Result<i32, String> {
    let word = words.next().ok_or("expected a coordinate")?;
    let value: i64 = word
        .parse()
        .map_err(|_| format!("`{word}` is not a number"))?;
    if !(-MAX_COORDINATE..=MAX_COORDINATE).contains(&value) {
        return Err(format!("coordinate {value} is outside ±{MAX_COORDINATE}"));
    }
    Ok(value as i32)
}

fn extent(words: &mut SplitWhitespace<'_>) -> Result<u32, String> {
    let word = words.next().ok_or("expected a width or height")?;
    let value: i64 = word
        .parse()
        .map_err(|_| format!("`{word}` is not a number"))?;
    if !(1..=MAX_SIDE).contains(&value) {
        return Err(format!("size {value} is outside 1..={MAX_SIDE}"));
    }
    Ok(value as u32)
}

pub(crate) fn colour(words: &mut SplitWhitespace<'_>) -> Result<Rgba, String> {
    let word = words.next().ok_or("expected a colour like #ff9d00")?;
    let digits = word
        .strip_prefix('#')
        .ok_or_else(|| format!("`{word}` is not a colour; write #rrggbb"))?;
    let bytes = digits.as_bytes();

    if bytes.len() != 6 {
        return Err(format!("`{word}` needs exactly six hexadecimal digits"));
    }
    let channel = |at: usize| {
        hex_pair(bytes[at], bytes[at + 1])
            .ok_or_else(|| format!("`{word}` needs exactly six hexadecimal digits"))
    };

    Ok(Rgba::rgb(channel(0)?, channel(2)?, channel(4)?))
}

fn hex_pair(high: u8, low: u8) -> Option<u8> {
    Some(nibble(high)? * 16 + nibble(low)?)
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

pub(crate) fn coordinate_prefix(source: &str) -> Result<(i32, &str), String> {
    let source = source.trim_start();
    let end = source.find(char::is_whitespace).unwrap_or(source.len());
    let (word, rest) = source.split_at(end);
    let value: i64 = word
        .parse()
        .map_err(|_| format!("`{word}` is not a number"))?;
    if !(-MAX_COORDINATE..=MAX_COORDINATE).contains(&value) {
        return Err(format!("coordinate {value} is outside ±{MAX_COORDINATE}"));
    }
    Ok((value as i32, rest))
}

pub(crate) fn scale_prefix(source: &str) -> Result<(u8, &str), String> {
    let source = source.trim_start();
    let end = source.find(char::is_whitespace).unwrap_or(source.len());
    let (word, rest) = source.split_at(end);
    let value: i64 = word
        .parse()
        .map_err(|_| format!("`{word}` is not a number"))?;
    if !(1..=i64::from(font::MAX_SCALE)).contains(&value) {
        return Err(format!("scale {value} is outside 1..={}", font::MAX_SCALE));
    }
    Ok((value as u8, rest))
}

pub(crate) fn quoted(source: &str) -> Result<(String, &str), String> {
    let source = source.trim_start();
    let Some(rest) = source.strip_prefix('"') else {
        return Err("expected a label in double quotes".to_string());
    };
    let Some(end) = rest.find('"') else {
        return Err("a label needs a closing quote".to_string());
    };
    let label = &rest[..end];
    if label.chars().count() > MAX_TEXT_CHARS {
        return Err(format!(
            "a label may hold at most {MAX_TEXT_CHARS} characters"
        ));
    }
    Ok((label.to_string(), &rest[end + 1..]))
}

pub(crate) fn colour_rest(source: &str) -> Result<Rgba, String> {
    let mut words = source.split_whitespace();
    let colour = colour(&mut words)?;
    end(&mut words)?;
    Ok(colour)
}

pub(crate) fn end(words: &mut SplitWhitespace<'_>) -> Result<(), String> {
    match words.next() {
        Some(extra) => Err(format!("unexpected `{extra}`")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_coordinate_is_read_as_a_number() {
        let mut words = "12 -7".split_whitespace();

        assert_eq!(point(&mut words), Ok(Point::new(12, -7)));
    }

    #[test]
    fn a_size_must_be_positive() {
        let mut words = "0".split_whitespace();

        assert!(
            extent(&mut words)
                .expect_err("a zero size")
                .contains("1..=")
        );
    }

    #[test]
    fn a_colour_is_read_from_six_hexadecimal_digits() {
        let mut words = "#ff9d00".split_whitespace();

        assert_eq!(colour(&mut words), Ok(Rgba::rgb(255, 157, 0)));
    }

    #[test]
    fn a_colour_of_the_wrong_length_is_refused() {
        let mut words = "#fff".split_whitespace();

        assert!(colour(&mut words).unwrap_err().contains("#fff"));
    }

    #[test]
    fn a_trailing_word_is_refused() {
        let mut words = "please".split_whitespace();

        assert!(end(&mut words).unwrap_err().contains("please"));
    }
}
