//! Presenting a frame to a terminal.
//!
//! A character cell is roughly twice as tall as it is wide, so one cell carries
//! two square pixels: the upper half block paints the top pixel as foreground
//! and the bottom pixel as background. The encoder is pure — it returns the
//! bytes a terminal would receive — so the whole presenter can be tested
//! without a terminal.
//!
//! Twenty-four bit colour is assumed. A palette fallback is a separate adapter,
//! because a terminal that cannot show true colour still needs a correct frame,
//! not a different renderer.

use crate::core::document::{self, ParseError};
use crate::core::frame::Frame;
use crate::core::geom::Rgba;
use crate::core::raster;

/// The upper half block: `▀`.
const HALF_BLOCK: char = '▀';

pub fn encode(frame: &Frame) -> String {
    let mut bytes = String::new();
    let mut y = 0;

    while let Some(top) = frame.row(y) {
        let bottom = frame.row(y + 1).unwrap_or(top);
        let mut previous = None;

        for (upper, lower) in top.iter().zip(bottom) {
            let colors = (*upper, *lower);
            if previous != Some(colors) {
                bytes.push_str(&escape(colors.0, colors.1));
                previous = Some(colors);
            }
            bytes.push(HALF_BLOCK);
        }

        bytes.push_str("\x1b[0m\n");
        y += 2;
    }

    bytes
}

fn escape(top: Rgba, bottom: Rgba) -> String {
    format!(
        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
        top.r, top.g, top.b, bottom.r, bottom.g, bottom.b
    )
}

pub fn paint(document: &str, width: u32, height: u32) -> Result<String, PaintError> {
    let scene = document::parse(document)?;
    let frame =
        raster::render(&scene, width, height).ok_or(PaintError::FrameTooLarge { width, height })?;
    Ok(encode(&frame))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Point;

    #[test]
    fn one_cell_shows_two_pixels() {
        let mut frame = Frame::new(1, 2).expect("a small frame");
        frame.set(Point::new(0, 1), Rgba::WHITE);

        let bytes = encode(&frame);

        assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;255;255;255m▀\x1b[0m\n");
    }

    #[test]
    fn a_run_of_one_colour_emits_one_escape() {
        let frame = Frame::new(2, 2).expect("a small frame");

        let bytes = encode(&frame);

        assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m▀▀\x1b[0m\n");
    }

    #[test]
    fn an_odd_last_row_repeats_itself() {
        let frame = Frame::new(1, 1).expect("a small frame");

        let bytes = encode(&frame);

        assert_eq!(bytes, "\x1b[38;2;0;0;0m\x1b[48;2;0;0;0m▀\x1b[0m\n");
    }

    #[test]
    fn a_document_paints_into_terminal_blocks() {
        let bytes = paint("clear #ff0000", 2, 2).expect("a valid document");

        assert_eq!(bytes, "\x1b[38;2;255;0;0m\x1b[48;2;255;0;0m▀▀\x1b[0m\n");
    }

    #[test]
    fn a_broken_document_names_its_line() {
        let error = paint("clear #000000\nsparkle", 4, 4).expect_err("a bad document");

        assert_eq!(error.to_string(), "line 2: unknown directive `sparkle`\n");
    }
}
