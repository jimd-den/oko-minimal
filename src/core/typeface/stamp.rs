//! A glyph, rasterised once: the stamp a named face presses onto a frame.
//!
//! Drawing a glyph the slow way is dear: the outline is read from the
//! face, drawn on a grid four times finer than the panel's, and brought
//! down (ADR 0013). A panel paints the same glyphs on every frame, so a
//! glyph is rasterised once into a stamp — the pixels it paints, where
//! they stand relative to the baseline point it is pressed at, and the
//! room the pen takes after it — and later frames press the stamp. The
//! stamp holds exactly the pixels the slow drawing set, so a pressed
//! stamp and a drawn glyph are the same picture.

use super::Glyph;
use crate::core::frame::Frame;
use crate::core::geom::{Point, Rect, Rgba};
use crate::core::paint;
use crate::core::path::{Path, Segment};

/// The grid a glyph is drawn on: four times finer than the panel's.
///
/// The count comes from the module's own grid, so the outline and the
/// drawing are on the same grid by construction.
const GRID: i32 = super::GRID as i32;

/// Nothing is here: the frame a glyph is drawn on starts empty.
const EMPTY: Rgba = Rgba::new(0, 0, 0, 0);

/// A rasterised glyph, ready to press onto a frame.
///
/// Two stamps are the same stamp when every field agrees: the tests lean
/// on that to say the cache hands back what it was given.
#[derive(Clone, Debug, PartialEq)]
pub struct Stamp {
    /// Where the first pixel stands, relative to the baseline point.
    pub offset: Point,
    /// The stamp's width and height, in panel pixels.
    pub width: u32,
    pub height: u32,
    /// The pixels the glyph paints, row by row. A pixel with no alpha
    /// paints nothing, exactly as the slow drawing skipped it.
    pub pixels: Vec<Rgba>,
    /// How far the pen moves after the glyph, on the fine grid.
    pub advance: f32,
}

impl Stamp {
    /// The stamp of a glyph in a colour: the glyph drawn on its fine
    /// grid, brought down to the panel's grid, and kept.
    pub fn of(glyph: &Glyph, colour: Rgba) -> Self {
        let empty = Self {
            offset: Point::new(0, 0),
            width: 0,
            height: 0,
            pixels: Vec::new(),
            advance: glyph.advance,
        };
        let Some(area) = area_of(&glyph.path) else {
            return empty;
        };
        // The frame begins on the grid, so that the pixels taken from it
        // are the ones the grid names.
        let left = area.x.div_euclid(GRID) * GRID;
        let top = area.y.div_euclid(GRID) * GRID;
        let width = area.x + area.w as i32 - left;
        let height = area.y + area.h as i32 - top;
        let mut fine = match Frame::new(width as u32, height as u32) {
            Some(fine) => fine,
            None => return empty,
        };
        fine.fill(fine.bounds(), EMPTY);
        let placed = moved(&glyph.path, left, top);
        paint::path(&mut fine, &placed, Some(colour), None);

        let cols = fine.width().div_ceil(GRID as u32);
        let rows = fine.height().div_ceil(GRID as u32);
        let mut pixels = vec![EMPTY; (cols * rows) as usize];
        for y in 0..fine.height() {
            for x in 0..fine.width() {
                if x % GRID as u32 != 0 || y % GRID as u32 != 0 {
                    continue;
                }
                let Some(pixel) = fine.color_at(Point::new(x as i32, y as i32)) else {
                    continue;
                };
                let index = (y / GRID as u32 * cols + x / GRID as u32) as usize;
                pixels[index] = pixel;
            }
        }
        Self {
            offset: Point::new(left / GRID, top / GRID),
            width: cols,
            height: rows,
            pixels,
            advance: glyph.advance,
        }
    }

    /// Press the stamp onto a frame at a baseline point.
    pub fn blit(&self, frame: &mut Frame, at: Point) {
        for (index, pixel) in self.pixels.iter().enumerate() {
            if pixel.a == 0 {
                continue;
            }
            let x = (index as u32 % self.width) as i32;
            let y = (index as u32 / self.width) as i32;
            frame.set(
                Point::new(at.x + self.offset.x + x, at.y + self.offset.y + y),
                *pixel,
            );
        }
    }
}

/// The area a path covers, on the fine grid.
///
/// The path's own points are where the shape is, so the area is the
/// smallest box that holds them.
fn area_of(path: &Path) -> Option<Rect> {
    let mut left = i32::MAX;
    let mut top = i32::MAX;
    let mut right = i32::MIN;
    let mut bottom = i32::MIN;
    let mut seen = false;

    for segment in &path.segments {
        for point in points_of(segment) {
            seen = true;
            left = left.min(point.x);
            top = top.min(point.y);
            right = right.max(point.x);
            bottom = bottom.max(point.y);
        }
    }
    if !seen {
        return None;
    }
    Some(Rect::new(
        left,
        top,
        (right - left + 1) as u32,
        (bottom - top + 1) as u32,
    ))
}

/// The path moved so that its area begins at the frame's origin.
///
/// A glyph stands on its baseline, so its outline reaches above it, and
/// the frame a glyph is drawn on begins at its own corner. The path is
/// moved into the frame and the place it came from is remembered by the
/// caller.
fn moved(path: &Path, left: i32, top: i32) -> Path {
    let mut moved = Path {
        segments: Vec::with_capacity(path.segments.len()),
    };
    for segment in &path.segments {
        moved.segments.push(match segment {
            Segment::Move { to } => Segment::Move {
                to: Point::new(to.x - left, to.y - top),
            },
            Segment::Line { to } => Segment::Line {
                to: Point::new(to.x - left, to.y - top),
            },
            Segment::Quadratic { control, to } => Segment::Quadratic {
                control: Point::new(control.x - left, control.y - top),
                to: Point::new(to.x - left, to.y - top),
            },
            Segment::Cubic {
                control1,
                control2,
                to,
            } => Segment::Cubic {
                control1: Point::new(control1.x - left, control1.y - top),
                control2: Point::new(control2.x - left, control2.y - top),
                to: Point::new(to.x - left, to.y - top),
            },
            Segment::Close => Segment::Close,
        });
    }
    moved
}

/// The points a segment names.
fn points_of(segment: &Segment) -> Vec<Point> {
    match segment {
        Segment::Move { to } => vec![*to],
        Segment::Line { to } => vec![*to],
        Segment::Quadratic { control, to } => vec![*control, *to],
        Segment::Cubic {
            control1,
            control2,
            to,
        } => vec![*control1, *control2, *to],
        Segment::Close => Vec::new(),
    }
}
