//! Reading a colour out of an export.
//!
//! An export writes a colour as hexadecimal digits, as `rgb()` or `rgba()`,
//! or by name. The names matter because Figma writes `fill="white"` and
//! `stroke="black"`, and a translator that knows only hexadecimal digits
//! refuses a whole file over two words.
//!
//! The document format has no alpha, so a translucent colour is reported
//! rather than quietly flattened to its opaque self.

#[cfg(test)]
mod tests;

use super::names;
use crate::core::geom::Rgba;

/// A colour an export writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    /// An opaque colour.
    Opaque(Rgba),
    /// A colour with an alpha channel.
    Translucent,
}

/// The colour a word names, if it names one this translator knows.
pub fn parse(word: &str) -> Option<Colour> {
    let word = word.trim();
    if let Some(digits) = word.strip_prefix('#') {
        return hex(digits);
    }
    if let Some(inner) = word
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return channels(inner, false);
    }
    if let Some(inner) = word
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        return channels(inner, true);
    }
    match names::colour(word) {
        // A name may carry alpha: `transparent` is black with no alpha left.
        Some(colour) if colour.a == 255 => Some(Colour::Opaque(colour)),
        Some(_) => Some(Colour::Translucent),
        None => None,
    }
}

/// A colour written as hexadecimal digits.
fn hex(digits: &str) -> Option<Colour> {
    let bytes = digits.as_bytes();
    let pair = |at: usize| -> Option<u8> {
        let high = nibble(bytes[at])?;
        let low = nibble(bytes[at + 1])?;
        Some(high * 16 + low)
    };
    match bytes.len() {
        3 => {
            let channel = |at: usize| nibble(bytes[at]).map(|value| value * 17);
            Some(Colour::Opaque(Rgba::rgb(
                channel(0)?,
                channel(1)?,
                channel(2)?,
            )))
        }
        6 => Some(Colour::Opaque(Rgba::rgb(pair(0)?, pair(2)?, pair(4)?))),
        _ => None,
    }
}

/// A colour written as `rgb()` or `rgba()`.
fn channels(inner: &str, alpha: bool) -> Option<Colour> {
    let parts: Vec<&str> = inner
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() != if alpha { 4 } else { 3 } {
        return None;
    }
    let mut channels = [0_u8; 3];
    for (slot, part) in channels.iter_mut().zip(parts.iter()) {
        let value: i64 = part.parse().ok()?;
        if !(0..=255).contains(&value) {
            return None;
        }
        *slot = value as u8;
    }
    if !alpha {
        return Some(Colour::Opaque(Rgba::rgb(
            channels[0],
            channels[1],
            channels[2],
        )));
    }
    let opacity: f64 = parts[3].parse().ok()?;
    if opacity >= 1.0 {
        return Some(Colour::Opaque(Rgba::rgb(
            channels[0],
            channels[1],
            channels[2],
        )));
    }
    Some(Colour::Translucent)
}

fn nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
