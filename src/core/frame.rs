//! A frame is the screen at one instant: `width` times `height` pixels, top to
//! bottom, with no padding.
//!
//! Every write is clipped, so a scene may paint outside the frame and the
//! result is the same as if it had not. A frame can fill a rectangle and
//! draw a line into itself; the shapes of a path are painted by `core::paint`.
//! Nothing here can panic on a bad coordinate; the only refusal is a size
//! too large to allocate.

use super::clip::clip_line;
use super::geom::{Point, Rect, Rgba};

/// 16 Mi pixels: a 4096 x 4096 frame at four bytes each. Larger than any
/// console this project targets, small enough to refuse a hostile size.
pub const MAX_PIXELS: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    width: u32,
    height: u32,
    pixels: Vec<Rgba>,
}

impl Frame {
    /// A frame of opaque black, or `None` when the size is too large.
    pub fn new(width: u32, height: u32) -> Option<Self> {
        let area = u64::from(width) * u64::from(height);
        if area > MAX_PIXELS {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels: vec![Rgba::BLACK; area as usize],
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn bounds(&self) -> Rect {
        Rect::new(0, 0, self.width, self.height)
    }

    pub fn color_at(&self, at: Point) -> Option<Rgba> {
        self.index(at).map(|i| self.pixels[i])
    }

    pub fn row(&self, y: u32) -> Option<&[Rgba]> {
        if y >= self.height {
            return None;
        }
        let start = (y * self.width) as usize;
        Some(&self.pixels[start..start + self.width as usize])
    }

    pub fn set(&mut self, at: Point, color: Rgba) {
        if let Some(i) = self.index(at) {
            self.pixels[i] = color;
        }
    }

    /// Paint a straight line, clipped to the frame.
    ///
    /// Bresenham: one pixel per step, no gaps, no floating point. The line
    /// is cut to the frame before it is walked, so a line from `i32::MIN` to
    /// `i32::MAX` does not take four billion steps to reach the edge.
    pub fn line(&mut self, from: Point, to: Point, color: Rgba) {
        let Some((start, end)) = clip_line(from, to, self.width, self.height) else {
            return;
        };
        self.steps(start, end, color);
    }

    /// Walk a clipped line, one pixel per step.
    fn steps(&mut self, from: Point, to: Point, color: Rgba) {
        let (mut x, mut y) = (i64::from(from.x), i64::from(from.y));
        let (end_x, end_y) = (i64::from(to.x), i64::from(to.y));
        let dx = (end_x - x).abs();
        let dy = -(end_y - y).abs();
        let step_x = if x < end_x { 1 } else { -1 };
        let step_y = if y < end_y { 1 } else { -1 };
        let mut error = dx + dy;

        loop {
            self.set(Point::new(x as i32, y as i32), color);
            if x == end_x && y == end_y {
                break;
            }
            let twice = 2 * error;
            if twice >= dy {
                error += dy;
                x += step_x;
            }
            if twice <= dx {
                error += dx;
                y += step_y;
            }
        }
    }

    /// Empty the frame, filling it with the colour.
    ///
    /// A runtime that paints into the frame it had wants a clean frame
    /// first, and a clear is a shape like any other.
    pub fn clear(&mut self, colour: Rgba) {
        self.fill(self.bounds(), colour);
    }

    /// Paint `area`, clipped to the frame.
    pub fn fill(&mut self, area: Rect, color: Rgba) {
        let Some(clipped) = area.intersect(self.bounds()) else {
            return;
        };
        for y in clipped.y..clipped.y + clipped.h as i32 {
            for x in clipped.x..clipped.x + clipped.w as i32 {
                self.set(Point::new(x, y), color);
            }
        }
    }

    fn index(&self, at: Point) -> Option<usize> {
        if at.x < 0 || at.y < 0 {
            return None;
        }
        let (x, y) = (at.x as u32, at.y as u32);
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_frame_is_opaque_black() {
        let frame = Frame::new(3, 2).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(2, 1)), Some(Rgba::BLACK));
        assert_eq!(frame.width(), 3);
        assert_eq!(frame.height(), 2);
    }

    #[test]
    fn a_frame_too_large_to_allocate_is_refused() {
        assert_eq!(Frame::new(65_536, 65_536), None);
    }

    #[test]
    fn a_write_outside_the_frame_is_ignored() {
        let mut frame = Frame::new(2, 2).expect("a small frame");

        frame.set(Point::new(-1, 0), Rgba::WHITE);
        frame.set(Point::new(2, 2), Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_reaches_both_ends_and_stays_inside() {
        let mut frame = Frame::new(4, 4).expect("a small frame");

        frame.line(Point::new(-10, 1), Point::new(10, 1), Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(0, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(1, 2)), Some(Rgba::BLACK));
    }

    #[test]
    fn clearing_a_frame_empties_it() {
        let mut frame = Frame::new(2, 2).expect("a small frame");
        frame.set(Point::new(0, 0), Rgba::WHITE);

        frame.clear(Rgba::rgb(0, 0, 255));

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::rgb(0, 0, 255)));
        assert_eq!(frame.color_at(Point::new(1, 1)), Some(Rgba::rgb(0, 0, 255)));
    }

    #[test]
    fn filling_clips_at_the_edges() {
        let mut frame = Frame::new(4, 3).expect("a small frame");

        frame.fill(Rect::new(2, 1, 10, 10), Rgba::WHITE);

        assert_eq!(frame.color_at(Point::new(2, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(1, 2)), Some(Rgba::BLACK));
    }
}
