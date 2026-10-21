//! What a run of terminal bytes means.

use super::sequence::{CHORD, Read, read};

#[test]
fn an_arrow_key_is_one_key_and_not_three() {
    let read = read(b"\x1b[A");

    assert_eq!(
        read,
        Read::Key {
            key: crate::core::event::Key::plain(crate::core::event::KeyCode::Up),
            len: 3
        }
    );
}

#[test]
fn the_escape_chord_is_the_chord() {
    let read = read(b"\x1b\x1b");

    assert_eq!(read, Read::Key { key: CHORD, len: 2 });
}

#[test]
fn a_half_sequence_needs_more_bytes() {
    let read = read(b"\x1b[");

    assert_eq!(read, Read::More);
}

#[test]
fn a_sequence_this_console_does_not_know_is_unknown() {
    let read = read(b"\x1b[Z");

    assert_eq!(read, Read::Unknown);
}

#[test]
fn a_byte_that_is_not_an_escape_is_not_a_sequence() {
    let read = read(b"a");

    assert_eq!(read, Read::Unknown);
}
