//! The screen as the terminal is showing it.
//!
//! A character cell holds two pixels, so a frame of pixels is a grid of
//! cells. This module holds the cells the terminal is showing and works
//! out which of them a new frame changes. The presenter turns that list
//! into bytes; the shell decides when to send them.
//!
//! The cells are made once and written into again, so a picture that has
//! not changed costs nothing: no bytes on the wire, and no work per tick
//! beyond the comparison.

use crate::core::frame::Frame;
use crate::core::geom::Rgba;

/// A character cell: the two pixels it shows, top and bottom.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub top: Rgba,
    pub bottom: Rgba,
}

impl Cell {
    /// A cell of the colour a fresh frame starts with.
    const fn plain() -> Self {
        Self {
            top: Rgba::BLACK,
            bottom: Rgba::BLACK,
        }
    }
}

/// A cell that changed, and where it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Changed {
    /// The character row, counting from zero.
    pub row: u32,
    /// The character column, counting from zero.
    pub column: u32,
    /// The colour the cell has now.
    pub cell: Cell,
}

/// The cells the terminal is showing.
pub struct Screen {
    /// The width, in cells.
    width: u32,
    /// The height, in cells.
    height: u32,
    cells: Vec<Cell>,
    changed: Vec<Changed>,
}

impl Screen {
    /// A screen the size of a frame.
    pub fn of(frame: &Frame) -> Self {
        let width = frame.width();
        let height = rows(frame.height());
        Self {
            width,
            height,
            cells: vec![Cell::plain(); count(width, height)],
            changed: Vec::new(),
        }
    }

    /// Whether this screen is the size of the frame.
    ///
    /// A frame of another size is a different screen: the terminal is
    /// showing something else now, and all of it has to be sent.
    pub fn fits(&self, frame: &Frame) -> bool {
        self.width == frame.width() && self.height == rows(frame.height())
    }

    /// Take the frame as it is: the screen shows it, and nothing changed.
    pub fn reset(&mut self, frame: &Frame) {
        let width = frame.width();
        let height = rows(frame.height());
        let mut cells = vec![Cell::plain(); count(width, height)];
        each_cell(frame, |row, column, cell| {
            cells[(row * width + column) as usize] = cell;
        });
        self.width = width;
        self.height = height;
        self.cells = cells;
        self.changed.clear();
    }

    /// The cells of the frame that differ from what the screen shows.
    ///
    /// The screen is brought up to date as it goes, so the next call
    /// compares against the picture that is on the terminal now.
    pub fn sync(&mut self, frame: &Frame) -> &[Changed] {
        self.changed.clear();
        let Self {
            width,
            cells,
            changed,
            ..
        } = self;
        let width = *width;
        each_cell(frame, |row, column, cell| {
            let index = (row * width + column) as usize;
            let Some(slot) = cells.get_mut(index) else {
                return;
            };
            if *slot != cell {
                *slot = cell;
                changed.push(Changed { row, column, cell });
            }
        });
        &self.changed
    }
}

/// How many character rows a frame of pixels has.
fn rows(height: u32) -> u32 {
    height.div_ceil(2)
}

/// How many cells a screen holds.
fn count(width: u32, height: u32) -> usize {
    (u64::from(width) * u64::from(height)) as usize
}

/// Walk the cells of a frame, row by row.
fn each_cell(frame: &Frame, mut visit: impl FnMut(u32, u32, Cell)) {
    for row in 0..rows(frame.height()) {
        let Some(top) = frame.row(row * 2) else {
            continue;
        };
        let bottom = frame.row(row * 2 + 1).unwrap_or(top);
        for (column, (upper, lower)) in top.iter().zip(bottom).enumerate() {
            visit(
                row,
                column as u32,
                Cell {
                    top: *upper,
                    bottom: *lower,
                },
            );
        }
    }
}
