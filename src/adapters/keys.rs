//! The bytes a terminal sends, turned into events.
//!
//! A terminal sends bytes, not keys: an arrow key is three bytes and one
//! key, and a reader that counted bytes would count an arrow key as three
//! keys. This module is where the counting happens, so nothing else in the
//! program has to know how a key is spelled.
//!
//! What a run of bytes means is in `sequence`; this module holds the bytes
//! a read brought and gives back one key at a time.

use crate::core::event::{Event, Key, KeyCode, Modifiers};

use super::sequence::{self, Read};

/// Turns the bytes a terminal sends into events.
#[derive(Default)]
pub struct Decoder {
    pending: Vec<u8>,
}

impl Decoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Take the bytes a read brought.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
    }

    /// The next event, when the bytes so far spell a whole one.
    ///
    /// The name is not `next`: this is not an iterator, and a reader that
    /// looks like one is a reader somebody will call `collect` on.
    pub fn event(&mut self) -> Option<Event> {
        loop {
            let first = *self.pending.first()?;
            if first == ESCAPE {
                match sequence::read(&self.pending) {
                    Read::Key { key, len } => {
                        self.pending.drain(..len);
                        return Some(Event::Key(key));
                    }
                    Read::Unknown => {
                        self.pending.remove(0);
                    }
                    Read::More => return None,
                }
                continue;
            }
            if first < 0x80 {
                self.pending.remove(0);
                return Some(Event::Key(plain(first)));
            }
            if let Some(event) = self.utf8() {
                return Some(event);
            }
        }
    }

    /// The escape key, for an escape byte that nothing followed.
    ///
    /// A read that timed out ends the wait for a sequence: an escape byte
    /// alone is a key of its own. Half a sequence is not an escape — its
    /// bytes stay, and the key they spell lands whenever the rest arrives,
    /// so a sequence split by the wait is still the one key it always was.
    pub fn flush(&mut self) -> Option<Event> {
        if self.pending == [ESCAPE] {
            self.pending.clear();
            return Some(Event::Key(Key::plain(KeyCode::Escape)));
        }
        None
    }

    /// A character that takes more than one byte.
    fn utf8(&mut self) -> Option<Event> {
        match std::str::from_utf8(&self.pending) {
            Ok(text) => {
                let character = text.chars().next()?;
                self.pending.drain(..character.len_utf8());
                Some(Event::Key(Key::plain(KeyCode::Char(character))))
            }
            Err(problem) => {
                // A character that is not complete yet is not an error: the
                // rest of its bytes are still to come. A character that
                // cannot be read at all is noise, and is dropped.
                problem.error_len()?;
                self.pending.remove(0);
                None
            }
        }
    }
}

/// The escape byte, which starts every sequence.
const ESCAPE: u8 = 0x1b;

/// A key that is not part of an escape sequence.
fn plain(byte: u8) -> Key {
    match byte {
        b'\r' | b'\n' => Key::plain(KeyCode::Enter),
        b'\t' => Key::plain(KeyCode::Tab),
        0x08 | 0x7f => Key::plain(KeyCode::Backspace),
        0x01..=0x1a => {
            let letter = (byte - 1 + b'a') as char;
            Key::new(KeyCode::Char(letter), Modifiers::CTRL)
        }
        _ => Key::plain(KeyCode::Char(byte as char)),
    }
}
