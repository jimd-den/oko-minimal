//! What a path command means.
//!
//! The reader one level down knows what a number looks like; this module
//! knows what each command does and which segments the path is made of. The
//! curve commands are one level out, in `curve`, because the smooth ones are
//! about reflection and nothing else.
//!
//! A path is read into the segments the core draws. A command this console
//! does not draw is not an error: it is named in the report, and the shapes
//! around it still draw.

use super::curve;
use super::reader::Reader;
use super::state::{Previous, State};
use super::{MAX_POINTS, Parsed};
use crate::adapters::svg::error::SvgError;
use crate::core::path::{Path, Segment};

/// The commands this grammar speaks. Every other letter is named and refused.
const COMMANDS: &[u8] = b"MmLlHhVvCcSsQqTtZz";

/// Read a `d` attribute into the segments the core draws.
pub(super) fn read(source: &str, line: usize) -> Result<Parsed, SvgError> {
    Grammar {
        reader: Reader::new(source, line),
        state: State::new(line),
        segments: Vec::new(),
        command: None,
    }
    .run()
}

/// A path being read.
struct Grammar<'a> {
    reader: Reader<'a>,
    state: State,
    segments: Vec<Segment>,
    /// The command being read, and the one that repeats when it is a number.
    command: Option<u8>,
}

impl Grammar<'_> {
    /// Read every command in the attribute.
    fn run(mut self) -> Result<Parsed, SvgError> {
        while self.reader.peek().is_some() {
            self.reader.separators();
            if self.reader.peek().is_none() {
                break;
            }
            if let Some(byte) = self.reader.command() {
                if !COMMANDS.contains(&byte) {
                    return Ok(Parsed::Undrawn(format!(
                        "the path uses `{letter}`, which this console does not draw",
                        letter = byte as char
                    )));
                }
                self.command = Some(byte);
            } else if self.command.is_none() {
                return Err(SvgError::new(
                    self.state.line,
                    "a path must start with a command",
                ));
            }
            self.step()?;
        }

        if self.segments.is_empty() {
            return Err(SvgError::new(
                self.state.line,
                "a path needs at least one point",
            ));
        }
        Ok(Parsed::Segments(Path {
            segments: self.segments,
        }))
    }

    /// Do what the current command says.
    fn step(&mut self) -> Result<(), SvgError> {
        let step = self.command.expect("a command was read above");
        let relative = step.is_ascii_lowercase();
        match step {
            b'M' | b'm' => self.move_to(relative),
            b'L' | b'l' | b'H' | b'h' | b'V' | b'v' => self.edge(step),
            b'C' | b'c' => self.curve(curve::cubic, relative),
            b'S' | b's' => self.curve(curve::smooth_cubic, relative),
            b'Q' | b'q' => self.curve(curve::quadratic, relative),
            b'T' | b't' => self.curve(curve::smooth_quadratic, relative),
            b'Z' | b'z' => self.close(),
            _ => unreachable!("the command was checked above"),
        }
    }

    /// A command that makes one curve segment.
    fn curve(
        &mut self,
        command: fn(&mut Reader<'_>, &mut State, bool) -> Result<Segment, SvgError>,
        relative: bool,
    ) -> Result<(), SvgError> {
        let segment = command(&mut self.reader, &mut self.state, relative)?;
        self.push(segment)
    }

    /// `M`: start a shape here.
    fn move_to(&mut self, relative: bool) -> Result<(), SvgError> {
        let (x, y) = self.reader.pair_at(self.state.current, relative)?;
        self.state.current = (x, y);
        self.state.origin = self.state.current;
        let to = self.state.at(self.state.current)?;
        self.push(Segment::Move { to })?;
        self.state.previous = Previous::Other;
        // A second pair after a move is a line, as SVG says.
        self.command = Some(if relative { b'l' } else { b'L' });
        Ok(())
    }

    /// `L`, `H` and `V`: a straight edge to here.
    fn edge(&mut self, step: u8) -> Result<(), SvgError> {
        let relative = step.is_ascii_lowercase();
        match step {
            b'L' | b'l' => {
                let (x, y) = self.reader.pair_at(self.state.current, relative)?;
                self.state.current = (x, y);
            }
            b'H' | b'h' => {
                let x = self.reader.number()?;
                self.state.current.0 = if relative {
                    self.state.current.0 + x
                } else {
                    x
                };
            }
            b'V' | b'v' => {
                let y = self.reader.number()?;
                self.state.current.1 = if relative {
                    self.state.current.1 + y
                } else {
                    y
                };
            }
            _ => unreachable!("only L, H and V come here"),
        }
        let to = self.state.at(self.state.current)?;
        self.push(Segment::Line { to })?;
        self.state.previous = Previous::Other;
        Ok(())
    }

    /// `Z`: close the shape here.
    fn close(&mut self) -> Result<(), SvgError> {
        self.push(Segment::Close)?;
        self.state.current = self.state.origin;
        self.state.previous = Previous::Other;
        self.command = None;
        Ok(())
    }

    /// Add a segment, refusing a path that is too long.
    fn push(&mut self, segment: Segment) -> Result<(), SvgError> {
        if self.segments.len() >= MAX_POINTS {
            return Err(SvgError::new(self.state.line, "a path has too many points"));
        }
        self.segments.push(segment);
        Ok(())
    }
}
