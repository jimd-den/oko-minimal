//! Reading numbers, colours and transforms out of SVG attributes.
//!
//! Everything here is total and bounded: a word either becomes a value or a
//! message with a line number. Units (`10px`), percentages and exponents are
//! refused rather than guessed, because a translator that silently rounds a
//! unit it does not understand is worse than one that stops.

use super::error::SvgError;
use super::names;
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
    split(word).iter().map(|part| integer(part, line)).collect()
}

/// The colour an attribute names.
pub fn colour(word: &str, line: usize) -> Result<Rgba, SvgError> {
    if let Some(digits) = word.strip_prefix('#') {
        return hex_colour(digits, word, line);
    }
    if word.starts_with("rgba(") {
        return Err(translucent(word, line));
    }
    if let Some(inner) = word
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return channels(&split(inner), word, line);
    }
    if let Some(colour) = names::colour(word) {
        return if colour.a == 255 {
            Ok(colour)
        } else {
            Err(translucent(word, line))
        };
    }
    Err(bad_colour(word, line))
}

/// The colour three channels name.
fn channels(parts: &[&str], word: &str, line: usize) -> Result<Rgba, SvgError> {
    if parts.len() != 3 {
        return Err(bad_colour(word, line));
    }
    let mut channels = [0_u8; 3];
    for (channel, part) in channels.iter_mut().zip(parts) {
        // A channel is one byte: a number outside 0..=255 is not a channel.
        *channel = part
            .parse::<u8>()
            .map_err(|_| SvgError::new(line, format!("`{word}` has no colour channel `{part}`")))?;
    }
    Ok(Rgba::rgb(channels[0], channels[1], channels[2]))
}

/// The parts of a list of numbers.
fn split(inner: &str) -> Vec<&str> {
    inner
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .collect()
}

/// A colour written as hexadecimal digits: three of them or six.
///
/// Three digits repeat one nibble; six digits pair two.
fn hex_colour(digits: &str, word: &str, line: usize) -> Result<Rgba, SvgError> {
    let nibbles: Option<Vec<u8>> = digits.bytes().map(nibble).collect();
    let nibbles = nibbles.ok_or_else(|| bad_colour(word, line))?;
    let channels: Vec<u8> = match nibbles.len() {
        3 => nibbles.iter().map(|nibble| nibble * 17).collect(),
        6 => nibbles
            .chunks(2)
            .map(|pair| pair[0] * 16 + pair[1])
            .collect(),
        _ => return Err(bad_colour(word, line)),
    };
    let [red, green, blue] = channels[..] else {
        return Err(bad_colour(word, line));
    };
    Ok(Rgba::rgb(red, green, blue))
}

fn bad_colour(word: &str, line: usize) -> SvgError {
    SvgError::new(
        line,
        format!("`{word}` is not a colour this translator understands"),
    )
}

/// A colour with an alpha channel, which this document format cannot hold:
/// the document holds one opaque colour, and the console draws no transparency.
fn translucent(word: &str, line: usize) -> SvgError {
    SvgError::new(
        line,
        format!("`{word}` has an alpha channel; this document holds opaque colours only"),
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
    let parts = split(inner);
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
    fn a_colour_name_is_read() {
        assert_eq!(colour("black", 1), Ok(Rgba::rgb(0, 0, 0)));
        assert_eq!(colour("white", 1), Ok(Rgba::rgb(255, 255, 255)));
    }

    #[test]
    fn a_translucent_colour_is_refused() {
        let error = colour("rgba(0, 0, 0, 0.5)", 3).expect_err("a translucent colour");

        assert_eq!(error.line, 3);
        assert!(error.problem.contains("alpha"));
    }

    #[test]
    fn a_relative_transform_is_refused() {
        let error = translate("rotate(45)", 3).expect_err("a rotation");

        assert_eq!(error.line, 3);
        assert!(error.problem.contains("only translate"));
    }
}
