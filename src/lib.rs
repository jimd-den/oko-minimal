//! ỌKỌ is a console that draws itself: it owns the screen, turns panel
//! documents into pixels, and hands a display lease to one session at a time.
//!
//! The crate is split at one line. `core` is pure: the same input returns the
//! same output and nothing outside itself is touched. `adapters` is where a
//! terminal, a file, or a device appears; it is the only layer allowed an
//! opinion about the outside world.
//!
//! The display list is the contract between them. A scene says what to paint,
//! a frame says what was painted, and a document is the human-edited source of
//! a scene. None of the three knows whether the pixels end up in a TTY, a
//! framebuffer, or a test.

pub mod adapters;
pub mod core;
