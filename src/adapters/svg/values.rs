//! Reading numbers, colours and transforms out of SVG attributes.
//!
//! Everything here is total and bounded: a word either becomes a value or a
//! message with a line number. Units (`10px`), percentages and exponents are
//! refused rather than guessed, because a translator that silently rounds a
//! unit it does not understand is worse than one that stops.

use super::error::SvgError;
use crate::core::geom::Rgba;

pub fn integer(word: &str, line: usize) -> Result<i32, SvgError> {
    Ok(scalar(word, line)?.round() as i32)
}

pub fn scalar(word: &str, line: usize) -> Result<f64, SvgError> {
    let value: f64 = word
        .trim()
        .parse()
        .map_err(|_| SvgError::new(line, format!("`{word}` is not a number")))?;
    if !value.is_finite() || value.abs() > 1_000_000.0 {
        return Err(SvgError::new(
            line,
            format!("`{word}` is outside the drawable range"),
        ));
    }
    Ok(value)
}

/// Whitespace or comma separated numbers, as `points` and `viewBox` use.
pub fn numbers(word: &str, line: usize) -> Result<Vec<i32>, SvgError> {
    word.split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .map(|part| integer(part, line))
        .collect()
}

pub fn colour(word: &str, line: usize) -> Result<Rgba, SvgError> {
    if let Some(digits) = word.strip_prefix('#') {
        return hex_colour(digits, word, line);
    }
    if let Some(inner) = word
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() == 3 {
            let mut channels = [0_u8; 3];
            for (channel, part) in channels.iter_mut().zip(parts) {
                *channel = channel_value(part, line)?;
            }
            return Ok(Rgba::rgb(channels[0], channels[1], channels[2]));
        }
    }
    Err(SvgError::new(
        line,
        format!("`{word}` is not a colour this translator understands"),
    ))
}

fn hex_colour(digits: &str, word: &str, line: usize) -> Result<Rgba, SvgError> {
    let bytes = digits.as_bytes();
    match bytes.len() {
        3 => {
            let channel = |at: usize| nibble(bytes[at]).map(|value| value * 17);
            Ok(Rgba::rgb(
                channel(0).ok_or_else(|| bad_colour(word, line))?,
                channel(1).ok_or_else(|| bad_colour(word, line))?,
                channel(2).ok_or_else(|| bad_colour(word, line))?,
            ))
        }
        6 => {
            let pair = |at: usize| Some(nibble(bytes[at])? * 16 + nibble(bytes[at + 1])?);
            Ok(Rgba::rgb(
                pair(0).ok_or_else(|| bad_colour(word, line))?,
                pair(2).ok_or_else(|| bad_colour(word, line))?,
                pair(4).ok_or_else(|| bad_colour(word, line))?,
            ))
        }
        _ => Err(bad_colour(word, line)),
    }
}

fn bad_colour(word: &str, line: usize) -> SvgError {
    SvgError::new(
        line,
        format!("`{word}` is not a colour this translator understands"),
    )
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn channel_value(word: &str, line: usize) -> Result<u8, SvgError> {
    let raw: i64 = word
        .parse()
        .map_err(|_| SvgError::new(line, format!("`{word}` is not a colour channel")))?;
    if !(0..=255).contains(&raw) {
        return Err(SvgError::new(line, format!("`{word}` is outside 0..=255")));
    }
    Ok(raw as u8)
}

/// Only `translate(x y)` is understood. Rotate, scale and matrix change the
/// meaning of every later coordinate, and a translator that guessed at them
/// would produce a panel that looks wrong in a way no one can trace.
pub fn translate(word: &str, line: usize) -> Result<(i32, i32), SvgError> {
    let inner = word
        .trim()
        .strip_prefix("translate(")
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or_else(|| {
            SvgError::new(
                line,
                format!("`{word}` is not supported; only translate(...) is"),
            )
        })?;
    let parts: Vec<&str> = inner
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .collect();
    match parts.as_slice() {
        [x] => Ok((integer(x, line)?, 0)),
        [x, y] => Ok((integer(x, line)?, integer(y, line)?)),
        _ => Err(SvgError::new(
            line,
            format!("`{word}` needs one or two numbers"),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hex_colour_is_read() {
        assert_eq!(colour("#ff9d00", 1), Ok(Rgba::rgb(255, 157, 0)));
    }

    #[test]
    fn a_short_hex_colour_is_expanded() {
        assert_eq!(colour("#0f0", 1), Ok(Rgba::rgb(0, 255, 0)));
    }

    #[test]
    fn an_rgb_colour_is_read() {
        assert_eq!(colour("rgb(12, 34, 56)", 1), Ok(Rgba::rgb(12, 34, 56)));
    }

    #[test]
    fn a_relative_transform_is_refused() {
        let error = translate("rotate(45)", 3).expect_err("a rotation");

        assert_eq!(error.line, 3);
        assert!(error.problem.contains("only translate"));
    }
}
