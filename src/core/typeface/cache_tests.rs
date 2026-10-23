//! The stamp cache, seen from the outside: what goes in comes back out,
//! and nothing else does.

use crate::core::geom::{Point, Rgba};
use crate::core::typeface::cache::{self, CAP, Key};
use crate::core::typeface::stamp::Stamp;

/// A small stamp worth keeping: one white pixel, some advance.
fn a_stamp() -> Stamp {
    Stamp {
        offset: Point::new(0, 0),
        width: 1,
        height: 1,
        pixels: vec![Rgba::WHITE],
        advance: 24.0,
    }
}

/// The key of an A in dejavu-mono at one size, in white.
fn a_key() -> Key {
    Key::new("dejavu-mono", 'A', 16.0, None, Rgba::WHITE)
}

#[test]
fn a_stamp_that_was_put_in_is_taken_out_again() {
    cache::clear();

    cache::put(a_key(), a_stamp());

    assert_eq!(cache::get(&a_key()), Some(a_stamp()));
}

#[test]
fn a_stamp_that_was_never_put_in_is_not_there() {
    cache::clear();

    assert_eq!(cache::get(&a_key()), None);
}

#[test]
fn another_colour_is_another_stamp() {
    cache::clear();
    cache::put(a_key(), a_stamp());

    let black = Key::new("dejavu-mono", 'A', 16.0, None, Rgba::BLACK);

    assert_eq!(cache::get(&black), None);
}

#[test]
fn another_size_is_another_stamp() {
    cache::clear();
    cache::put(a_key(), a_stamp());

    let bigger = Key::new("dejavu-mono", 'A', 32.0, None, Rgba::WHITE);

    assert_eq!(cache::get(&bigger), None);
}

#[test]
fn a_full_cache_stays_at_or_below_its_cap() {
    cache::clear();

    for size in 0..CAP + 16 {
        let key = Key::new("dejavu-mono", 'A', size as f32, None, Rgba::WHITE);
        cache::put(key, a_stamp());
    }

    assert!(
        cache::len() <= CAP,
        "the cache holds {}, its cap is {CAP}",
        cache::len()
    );
}

#[test]
fn clearing_the_cache_empties_it() {
    cache::put(a_key(), a_stamp());

    cache::clear();

    assert_eq!(cache::len(), 0);
    assert_eq!(cache::get(&a_key()), None);
}
