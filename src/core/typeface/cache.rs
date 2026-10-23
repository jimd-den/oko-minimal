//! The stamps the program has already rasterised, kept for later frames.
//!
//! A stamp is dear to make and cheap to keep, and a panel's frames are
//! mostly the same glyphs pressed again, so the stamps live in one place
//! and are shared by every label. The cache is deliberately simple: when
//! it is full it clears itself rather than growing without bound, and a
//! panel re-stamps its glyphs over the next frames.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use super::stamp::Stamp;
use crate::core::geom::Rgba;

/// How many stamps are kept before the cache clears itself.
pub const CAP: usize = 4096;

/// What makes one stamp different from another: the face, the character,
/// the size it was drawn at, the weight, and the colour it was pressed
/// in. Two glyphs that differ in any of these are different stamps.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    /// The face's name.
    pub face: &'static str,
    /// The character drawn.
    pub character: char,
    /// The em size in pixels, as bits: the same size is the same stamp.
    pub em: u32,
    /// The weight the face was set at, if any.
    pub weight: Option<u16>,
    /// The colour the glyph was pressed in.
    pub colour: (u8, u8, u8, u8),
}

impl Key {
    /// The key of a glyph: its face, character, size, weight and colour.
    pub fn new(
        face: &'static str,
        character: char,
        em: f32,
        weight: Option<u16>,
        colour: Rgba,
    ) -> Self {
        Self {
            face,
            character,
            em: em.to_bits(),
            weight,
            colour: (colour.r, colour.g, colour.b, colour.a),
        }
    }
}

/// The stamps, kept once for the whole program.
static STAMPS: LazyLock<Mutex<HashMap<Key, Stamp>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

/// The stamps themselves, behind the one lock the program takes for them.
fn store() -> std::sync::MutexGuard<'static, HashMap<Key, Stamp>> {
    STAMPS
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The stamp under a key, if one has been made.
pub fn get(key: &Key) -> Option<Stamp> {
    store().get(key).cloned()
}

/// Keep a stamp under its key. A full cache clears itself first, so it
/// never holds more than its cap.
pub fn put(key: Key, stamp: Stamp) {
    let mut stamps = store();
    if stamps.len() >= CAP {
        stamps.clear();
    }
    stamps.insert(key, stamp);
}

/// How many stamps are being kept.
pub fn len() -> usize {
    store().len()
}

/// Drop every stamp being kept.
pub fn clear() {
    store().clear();
}
