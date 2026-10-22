//! Writing a font's outline into the core's path.
//!
//! `skrifa` hands an outline over as a pen's calls: a move, some lines, a
//! curve or two, a close. The core wants a path. This is the pen that
//! takes those calls and writes the core's own segments.
//!
//! The path is on the fine grid and the y axis is turned over, because the
//! font's y axis points up and the panel's points down (ADR 0013).

use skrifa::outline::OutlinePen;

use super::GRID;
use crate::core::geom::Point;
use crate::core::path::{Path, Segment};

/// A pen that writes a glyph's outline into the core's path.
#[derive(Default)]
pub struct PathPen {
    path: Path,
}

impl PathPen {
    /// The path written so far.
    pub fn into_path(self) -> Path {
        self.path
    }

    /// A point on the fine grid.
    ///
    /// The font's y axis points up and the panel's points down, so the y is
    /// turned over as well as scaled.
    fn point(&self, x: f32, y: f32) -> Point {
        Point::new((x * GRID).round() as i32, (-y * GRID).round() as i32)
    }
}

impl OutlinePen for PathPen {
    fn move_to(&mut self, x: f32, y: f32) {
        let to = self.point(x, y);
        self.path.segments.push(Segment::Move { to });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let to = self.point(x, y);
        self.path.segments.push(Segment::Line { to });
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let control = self.point(cx0, cy0);
        let to = self.point(x, y);
        self.path.segments.push(Segment::Quadratic { control, to });
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let control1 = self.point(cx0, cy0);
        let control2 = self.point(cx1, cy1);
        let to = self.point(x, y);
        self.path.segments.push(Segment::Cubic {
            control1,
            control2,
            to,
        });
    }

    fn close(&mut self) {
        self.path.segments.push(Segment::Close);
    }
}
