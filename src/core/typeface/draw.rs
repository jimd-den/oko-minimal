//! Drawing a real font's glyph.
//!
//! A glyph is drawn on a frame of its own, four times the size of its
//! place, and brought down (ADR 0013). The core's path holds whole pixels
//! and a glyph's outline does not, so the outline is drawn on the fine grid
//! and reduced to the panel's grid.

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

/// Draw a glyph into a frame, at a baseline, in a colour.
///
/// The `at` is the baseline: the glyph stands on it, as it stands on the
/// page.
pub fn draw(frame: &mut Frame, glyph: &Glyph, at: Point, colour: Rgba) {
    let Some(area) = area_of(&glyph.path) else {
        return;
    };
    // The frame begins on the grid, so that the pixels taken from it are
    // the ones the grid names.
    let left = area.x.div_euclid(GRID) * GRID;
    let top = area.y.div_euclid(GRID) * GRID;
    let width = area.x + area.w as i32 - left;
    let height = area.y + area.h as i32 - top;
    let mut fine = match Frame::new(width as u32, height as u32) {
        Some(fine) => fine,
        None => return,
    };
    fine.fill(fine.bounds(), EMPTY);
    let placed = moved(&glyph.path, left, top);
    paint::path(&mut fine, &placed, Some(colour), None);

    for y in 0..fine.height() {
        for x in 0..fine.width() {
            if x % GRID as u32 != 0 || y % GRID as u32 != 0 {
                continue;
            }
            let Some(pixel) = fine.color_at(Point::new(x as i32, y as i32)) else {
                continue;
            };
            if pixel.a == 0 {
                continue;
            }
            frame.set(
                Point::new(
                    at.x + left / GRID + (x as i32) / GRID,
                    at.y + top / GRID + (y as i32) / GRID,
                ),
                pixel,
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
/// A glyph stands on its baseline, so its outline reaches above it, and the
/// frame a glyph is drawn on begins at its own corner. The path is moved
/// into the frame and the place it came from is remembered by the caller.
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
