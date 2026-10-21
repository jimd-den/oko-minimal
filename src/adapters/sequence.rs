//! What a run of terminal bytes means.
//!
//! A terminal writes escape sequences to say things it has no byte for: an
//! arrow key is `ESC [ A`, a function key is `ESC O P`. This module reads
//! one such sequence and says which key it is. It knows nothing about
//! buffering and nothing about events: it takes the bytes that are there
//! and gives back a key, or says it needs more bytes, or says it does not
//! know the sequence.
//!
//! The escape chord is here too, because a terminal cannot report control
//! and alt apart from escape: it says "alt with escape" by sending escape
//! twice, and the console reserves that run for the way back.

use crate::core::event::{Key, KeyCode, Modifiers};

/// The key that hands the screen back to the console.
///
/// The escape key with the control and alt modifiers, which is what the
/// core compares against.
pub const CHORD: Key = Key::new(KeyCode::Escape, Modifiers::CTRL_ALT);

/// The escape byte, which starts every sequence.
const ESCAPE: u8 = 0x1b;
/// The byte that starts a control sequence.
const CONTROL: u8 = b'[';
/// The byte that starts a single-shift sequence.
const SHIFT: u8 = b'O';

/// What a run of bytes turned out to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    /// The bytes spell this key, and it was this many bytes long.
    Key { key: Key, len: usize },
    /// The bytes are not a key this console knows.
    Unknown,
    /// The bytes are half a sequence; the rest are needed.
    More,
}

/// Read the escape sequence at the front of `bytes`.
pub fn read(bytes: &[u8]) -> Read {
    if bytes.first() != Some(&ESCAPE) {
        return Read::Unknown;
    }
    let run = bytes.iter().take_while(|b| **b == ESCAPE).count();
    if run > 1 {
        return Read::Key {
            key: CHORD,
            len: run,
        };
    }
    match bytes.get(1) {
        Some(&CONTROL) => control(bytes),
        Some(&SHIFT) => single_shift(bytes),
        Some(_) => Read::Key {
            key: Key::new(KeyCode::Char(bytes[1] as char), Modifiers::ALT),
            len: 2,
        },
        None => Read::More,
    }
}

/// A control sequence: `ESC [` and a final byte.
fn control(bytes: &[u8]) -> Read {
    let Some(end) = final_byte(bytes) else {
        return Read::More;
    };
    let code = match bytes[end] {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        _ => return Read::Unknown,
    };
    Read::Key {
        key: Key::new(code, modifiers(&bytes[2..end])),
        len: end + 1,
    }
}

/// A single-shift sequence: `ESC O` and a final byte.
fn single_shift(bytes: &[u8]) -> Read {
    let Some(end) = final_byte(bytes) else {
        return Read::More;
    };
    let code = match bytes[end] {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        b'P' => KeyCode::Function(1),
        b'Q' => KeyCode::Function(2),
        b'R' => KeyCode::Function(3),
        b'S' => KeyCode::Function(4),
        _ => return Read::Unknown,
    };
    Read::Key {
        key: Key::plain(code),
        len: end + 1,
    }
}

/// The index of the final byte of a sequence, once it has arrived.
fn final_byte(bytes: &[u8]) -> Option<usize> {
    bytes
        .iter()
        .skip(2)
        .position(|byte| (0x40..=0x7e).contains(byte))
        .map(|at| at + 2)
}

/// The modifiers the parameters of a control sequence ask for.
///
/// The last parameter counts them the way xterm does: one, plus one for
/// shift, two for alt and four for control.
fn modifiers(parameters: &[u8]) -> Modifiers {
    let Some(last) = parameters
        .split(|byte| *byte == b';')
        .filter_map(|part| std::str::from_utf8(part).ok()?.parse::<u32>().ok())
        .next_back()
    else {
        return Modifiers::NONE;
    };
    let bits = last.saturating_sub(1);
    Modifiers {
        ctrl: bits & 4 != 0,
        alt: bits & 2 != 0,
        shift: bits & 1 != 0,
    }
}
