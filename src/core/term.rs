//! A rectangular terminal viewport inside a panel.
//!
//! A panel may place named viewports (`Term`) into its display list. Nothing
//! here spawns a process or touches a device: `Term` buffers and wraps text
//! purely in memory and lowers its visible rows into `Shape::Text` elements
//! when the panel builds its scene.

use super::font;
use super::geom::{Point, Rect, Rgba};
use super::motion::Motion;
use super::scene::{Condition, Element};
use super::shape::Shape;

/// The maximum number of scrollback lines kept in a terminal viewport.
pub const MAX_LINES: usize = 64;
/// The maximum length of an interactive command line.
const MAX_INPUT_CHARS: usize = 120;
/// Seven rows of glyph plus one row of leading between terminal lines.
const ROW_STRIDE: u32 = 8;

/// A rectangular terminal viewport placed on a panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term {
    name: String,
    area: Rect,
    scale: u8,
    color: Rgba,
    condition: Condition,
    lines: Vec<String>,
    input: String,
}

impl Term {
    /// A new, empty terminal viewport.
    pub fn new(
        name: impl Into<String>,
        area: Rect,
        scale: u8,
        color: Rgba,
        condition: Condition,
    ) -> Self {
        Self {
            name: name.into(),
            area,
            scale: scale.clamp(1, font::MAX_SCALE),
            color,
            condition,
            lines: Vec::new(),
            input: String::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn scale(&self) -> u8 {
        self.scale
    }

    pub fn color(&self) -> Rgba {
        self.color
    }

    pub fn condition(&self) -> &Condition {
        &self.condition
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    /// How many characters fit across one row of this viewport.
    pub fn columns(&self) -> usize {
        let cell_w = (font::ADVANCE as u32).saturating_mul(u32::from(self.scale));
        (self.area.w / cell_w.max(1)).max(1) as usize
    }

    /// How many text rows fit vertically in this viewport.
    pub fn rows(&self) -> usize {
        let cell_h = ROW_STRIDE.saturating_mul(u32::from(self.scale));
        (self.area.h / cell_h.max(1)).max(1) as usize
    }

    /// Replace the viewport's buffer with `text`, wrapped to the viewport width.
    pub fn write(&mut self, text: &str) {
        self.lines.clear();
        self.append(text);
    }

    /// Append `text` to the viewport's scrollback buffer.
    pub fn append(&mut self, text: &str) {
        let width = self.columns();
        for raw in text.lines() {
            let clean = sanitize(raw);
            if clean.is_empty() {
                self.lines.push(String::new());
                continue;
            }
            let chars: Vec<char> = clean.chars().collect();
            for chunk in chars.chunks(width) {
                self.lines.push(chunk.iter().collect());
            }
        }
        if self.lines.len() > MAX_LINES {
            let excess = self.lines.len() - MAX_LINES;
            self.lines.drain(..excess);
        }
    }

    /// Type a character into the interactive command line.
    pub fn push_char(&mut self, ch: char) -> bool {
        if !font::supports(ch) || self.input.chars().count() >= MAX_INPUT_CHARS {
            return false;
        }
        self.input.push(ch);
        true
    }

    /// Delete the last character from the interactive command line.
    pub fn pop_char(&mut self) -> bool {
        self.input.pop().is_some()
    }

    /// Submit the current command line, echoing it to scrollback.
    pub fn submit(&mut self) -> String {
        let command = std::mem::take(&mut self.input);
        self.append(&format!("> {command}"));
        command
    }

    /// Lower the visible rows of this terminal into display-list elements.
    pub fn elements(&self, focused: bool) -> Vec<Element> {
        let capacity = self.rows();
        let history_budget = if focused {
            capacity.saturating_sub(1)
        } else {
            capacity
        };
        let start = self.lines.len().saturating_sub(history_budget);
        let mut visible: Vec<String> = self.lines[start..].to_vec();
        if focused {
            let prompt = sanitize(&format!("> {}_", self.input));
            let chars: Vec<char> = prompt.chars().collect();
            let tail_start = chars.len().saturating_sub(self.columns());
            visible.push(chars[tail_start..].iter().collect());
        }
        let step = (ROW_STRIDE as i32) * i32::from(self.scale);
        visible
            .into_iter()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
            .map(|(row, text)| Element {
                shape: Shape::Text {
                    at: Point::new(self.area.x, self.area.y + (row as i32) * step),
                    text,
                    scale: self.scale,
                    color: self.color,
                    spacing: 0,
                },
                motion: Motion::none(),
                condition: Condition::Always,
            })
            .collect()
    }
}

/// Sanitize arbitrary command output so every character is in the built-in font.
fn sanitize(raw: &str) -> String {
    raw.chars()
        .map(|ch| if font::supports(ch) { ch } else { ' ' })
        .collect()
}
