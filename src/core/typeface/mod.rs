//! A real font: the reading of it, borrowed from skrifa.
//!
//! The project keeps the drawing and borrows the reading (ADR 0011).
//! `skrifa` reads TrueType outlines, CFF outlines, the character map, the
//! advances and a variable font's deltas; it does not shape, hint or draw.
//! A glyph's outline arrives here as a path, and the core's own path
//! painter fills it, so a glyph is a path and not a second renderer.
//!
//! What is not here is shaping: there is no kerning, no ligature and no
//! hinting. A character the face does not have is a hole in the panel, and
//! the panel says so.
//!
//! The faces are compiled into the program, and their licences are beside
//! them in `assets/fonts`. The program carries three: a monospaced face, a
//! proportional face and a variable one.
//!
//! The pieces: `face` reads a font file, `pen` writes an outline into the
//! core's path, `setting` names a face for a panel, `stamp` rasterises a
//! glyph once, and `cache` keeps the stamps for later frames.

pub mod cache;
mod face;
mod pen;
mod setting;
pub mod stamp;

#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod pen_tests;
#[cfg(test)]
mod stamp_tests;

pub use face::{Face, Glyph};
pub use setting::Setting;

/// The grid a glyph is drawn on: four times finer than the panel's.
///
/// The core's path holds whole pixels, and a glyph's outline is not whole
/// pixels. The outline is scaled up by this factor, drawn on the finer
/// grid, and brought down (ADR 0013).
pub const GRID: f32 = 4.0;
