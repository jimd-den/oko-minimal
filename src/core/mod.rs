//! Pure state: geometry, frames, scenes, the renderer, documents, and the two
//! arbiters of the screen — the display lease and input focus.
//!
//! Nothing here reads a file, opens a device, or writes to a terminal. That is
//! what makes every rule below testable in one line.

pub mod binding;
#[cfg(test)]
mod binding_tests;
pub mod blend;
#[cfg(test)]
mod blend_tests;
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
pub mod panel;
#[cfg(test)]
mod panel_tests;
pub mod path;
pub mod raster;
#[cfg(test)]
mod raster_tests;
pub mod scene;
#[cfg(test)]
mod scene_tests;
pub mod session;
pub mod shape;
#[cfg(test)]
mod shape_tests;
pub mod syntax;
pub mod text;
#[cfg(test)]
mod text_tests;
pub mod timeline;
