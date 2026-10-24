//! The bytes a terminal sends, turned into events.

use super::keys::Decoder;
use crate::core::event::{Event, Key, KeyCode, Modifiers};

#[test]
fn an_arrow_key_is_one_event_and_not_three() {
    let mut decoder = Decoder::new();
    decoder.feed(b"\x1b[A");

    let event = decoder.event().expect("an event");

    assert_eq!(event, Event::Key(Key::plain(KeyCode::Up)));
    assert_eq!(decoder.event(), None);
}

#[test]
fn the_escape_chord_is_one_event() {
    let mut decoder = Decoder::new();
    decoder.feed(b"\x1b\x1b");

    let event = decoder.event().expect("an event");

    assert_eq!(
        event,
        Event::Key(Key::new(KeyCode::Escape, Modifiers::CTRL_ALT))
    );
}

#[test]
fn a_plain_character_is_a_character() {
    let mut decoder = Decoder::new();
    decoder.feed(b"a");

    let event = decoder.event().expect("an event");

    assert_eq!(event, Event::Key(Key::plain(KeyCode::Char('a'))));
}

#[test]
fn a_character_that_takes_more_than_one_byte_is_one_event() {
    let mut decoder = Decoder::new();
    decoder.feed("é".as_bytes());

    let event = decoder.event().expect("an event");

    assert_eq!(event, Event::Key(Key::plain(KeyCode::Char('é'))));
}

#[test]
fn an_escape_that_nothing_followed_is_a_key_of_its_own() {
    let mut decoder = Decoder::new();
    decoder.feed(b"\x1b");

    assert_eq!(decoder.event(), None);
    assert_eq!(
        decoder.flush(),
        Some(Event::Key(Key::plain(KeyCode::Escape)))
    );
}

#[test]
fn a_sequence_split_by_a_timeout_still_spells_its_key() {
    let mut decoder = Decoder::new();
    decoder.feed(b"\x1b[");

    assert_eq!(decoder.event(), None);
    assert_eq!(decoder.flush(), None, "half a sequence is not an escape");

    decoder.feed(b"A");

    assert_eq!(
        decoder.event(),
        Some(Event::Key(Key::plain(KeyCode::Up))),
        "the rest of the sequence, whenever it arrives, completes the key"
    );
}
