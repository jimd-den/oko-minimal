//! The outside world: a terminal, its bytes, and the files a person edits.
//!
//! Adapters are thin on purpose. They translate between bytes and the plain
//! data in `core`, and they own every operation that can fail for a reason the
//! program cannot control.

pub mod svg;
pub mod terminal;
