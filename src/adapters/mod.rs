//! The outside world: a terminal, its bytes, and the files a person edits.
//!
//! Adapters are thin on purpose. They translate between bytes and the plain
//! data in `core`, and they own every operation that can fail for a reason the
//! program cannot control.

pub mod keys;
#[cfg(test)]
mod keys_tests;
pub mod presenter;
#[cfg(test)]
mod presenter_tests;
pub mod runtime;
#[cfg(test)]
mod runtime_tests;
pub mod screen;
#[cfg(test)]
mod screen_tests;
pub mod sequence;
#[cfg(test)]
mod sequence_tests;
pub mod shell;
pub mod svg;
pub mod terminal;
#[cfg(test)]
mod terminal_tests;
