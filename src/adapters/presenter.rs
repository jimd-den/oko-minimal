//! Presenting a frame to a terminal.
//!
//! A character cell is roughly twice as tall as it is wide, so one cell
//! carries two square pixels: the upper half block paints the top pixel as
//! foreground and the bottom pixel as background. The encoder is pure - it
//! returns the bytes a terminal would receive - so the whole presenter can
//! be tested without a terminal.
//!
//! Two shapes of the same work live here. `paint` writes a frame as a
//! stream of lines, which is what a pipe wants. `screen` places the rows
//! on a screen, which is what a terminal wants. Neither knows about a
//! device; the terminal adapter owns the device.
//!
//! Twenty-four bit colour is assumed. A palette fallback is a separate
//! adapter, because a terminal that cannot show true colour still needs a
//! correct frame, not a different renderer.

use crate::core::document::{self, ParseError};
use crate::core::frame::Frame;
use crate::core::geom::Rgba;
use crate::core::raster;

/// The upper half block: `▀`.
const HALF_BLOCK: char = '\u{2580}';

/// The size of a frame, in pixels, for a window of the given cells.
///
/// A cell is one pixel wide and two pixels tall, so a window of `columns`
/// by `rows` cells is a frame of `columns` by `rows * 2` pixels.
pub fn pixel_size(columns: u32, rows: u32) -> (u32, u32) {
    (columns, rows.saturating_mul(2))
}

/// The bytes of a document, painted at the given size in pixels.
pub fn paint(document: &str, width: u32, height: u32) -> Result<String, PaintError> {
    let panel = document::parse(document)?;
    let frame = raster::render(&panel.scene(0), width, height)
        .ok_or(PaintError::FrameTooLarge { width, height })?;
    Ok(encode(&frame))
}

/// The bytes of a frame, one line per character row.
pub fn encode(frame: &Frame) -> String {
    let mut bytes = String::new();
    for row in rows(frame) {
        bytes.push_str(&row);
        bytes.push_str("\x1b[0m\n");
    }
    bytes
}

/// The bytes that put a frame on a screen: every row at the left margin,
/// in order, and the rest of the screen cleared.
pub fn screen(frame: &Frame) -> String {
    let mut bytes = String::new();
    for (index, row) in rows(frame).into_iter().enumerate() {
        bytes.push_str(&format!("\x1b[{};1H", index + 1));
        bytes.push_str(&row);
        bytes.push_str("\x1b[0m");
    }
    bytes.push_str("\x1b[J");
    bytes
}

/// The character rows of a frame: the colour of every cell, as the
/// characters a terminal shows.
fn rows(frame: &Frame) -> Vec<String> {
    let mut rows = Vec::new();
    let mut y = 0;

    while let Some(top) = frame.row(y) {
        let bottom = frame.row(y + 1).unwrap_or(top);
        let mut previous = None;
        let mut row = String::new();

        for (upper, lower) in top.iter().zip(bottom) {
            let colors = (*upper, *lower);
            if previous != Some(colors) {
                row.push_str(&escape(colors.0, colors.1));
                previous = Some(colors);
            }
            row.push(HALF_BLOCK);
        }

        rows.push(row);
        y += 2;
    }

    rows
}

fn escape(top: Rgba, bottom: Rgba) -> String {
    format!(
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
        top.r, top.g, top.b, bottom.r, bottom.g, bottom.b
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PaintError {
    Document(Vec<ParseError>),
    FrameTooLarge { width: u32, height: u32 },
}

impl std::fmt::Display for PaintError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Document(errors) => {
                for error in errors {
                    writeln!(formatter, "{error}")?;
                }
                Ok(())
            }
            Self::FrameTooLarge { width, height } => {
                write!(formatter, "a {width}x{height} frame is too large")
            }
        }
    }
}

impl From<Vec<ParseError>> for PaintError {
    fn from(errors: Vec<ParseError>) -> Self {
        Self::Document(errors)
    }
}
