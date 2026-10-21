//! Pure state: geometry, frames, scenes, the renderer, documents, and the two
//! arbiters of the screen — the display lease and input focus.
//!
//! Nothing here reads a file, opens a device, or writes to a terminal. That is
//! what makes every rule below testable in one line.

pub mod blend;
pub mod clip;
pub mod curve;
pub mod document;
pub mod event;
pub mod focus;
pub mod font;
pub mod frame;
pub mod geom;
pub mod interpolate;
pub mod lease;
pub mod motion;
pub mod paint;
pub mod path;
pub mod raster;
#[cfg(test)]
mod raster_tests;
pub mod scene;
pub mod session;
pub mod shape;
pub mod syntax;
pub mod text;
pub mod timeline;
