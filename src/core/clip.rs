//! Cutting a line down to the part inside the frame.
//!
//! Clipping before stepping is what keeps rendering bounded. A segment from
//! `i32::MIN` to `i32::MAX` has four billion steps; the visible part of it has
//! at most the frame's width plus its height. The outcode test and the
//! edge-by-edge cut are Cohen–Sutherland, written out rather than hidden,
//! because a renderer that can be made to spin is a renderer that can be
//! stopped from outside.
//!
//! The arithmetic is 128-bit: a product of two 32-bit differences does not fit
//! in 64, and a wrap here would paint the wrong pixels rather than none.

use super::geom::Point;

/// The four edges a point can lie beyond. A point inside the frame has no bits.
const LEFT: u8 = 1 << 0;
const RIGHT: u8 = 1 << 1;
const TOP: u8 = 1 << 2;
const BOTTOM: u8 = 1 << 3;

fn outcode(x: i128, y: i128, width: i128, height: i128) -> u8 {
    let mut code = 0;
    if x < 0 {
        code |= LEFT;
    } else if x >= width {
        code |= RIGHT;
    }
    if y < 0 {
        code |= TOP;
    } else if y >= height {
        code |= BOTTOM;
    }
    code
}

/// Cohen–Sutherland: cut a segment down to the part inside the frame.
///
/// The alternative — stepping the whole segment and letting out-of-range
/// writes fall on the floor — would let a line from `i32::MIN` to `i32::MAX`
/// spin for four billion iterations. Clipping first bounds the work by the
/// frame, not by the coordinates. The arithmetic is 128-bit because a product
/// of two 32-bit differences does not fit in 64.
pub(crate) fn clip_line(from: Point, to: Point, width: u32, height: u32) -> Option<(Point, Point)> {
    let (width, height) = (i128::from(width), i128::from(height));
    let (mut x0, mut y0) = (i128::from(from.x), i128::from(from.y));
    let (mut x1, mut y1) = (i128::from(to.x), i128::from(to.y));
    let mut code0 = outcode(x0, y0, width, height);
    let mut code1 = outcode(x1, y1, width, height);

    loop {
        if code0 | code1 == 0 {
            let start = Point::new(x0 as i32, y0 as i32);
            let end = Point::new(x1 as i32, y1 as i32);
            return Some((start, end));
        }
        if code0 & code1 != 0 {
            return None;
        }

        let outside = if code0 != 0 { code0 } else { code1 };
        let (x, y) = if outside & TOP != 0 {
            (x0 + (x1 - x0) * -y0 / (y1 - y0), 0)
        } else if outside & BOTTOM != 0 {
            (x0 + (x1 - x0) * (height - 1 - y0) / (y1 - y0), height - 1)
        } else if outside & RIGHT != 0 {
            (width - 1, y0 + (y1 - y0) * (width - 1 - x0) / (x1 - x0))
        } else {
            (0, y0 + (y1 - y0) * -x0 / (x1 - x0))
        };

        if outside == code0 {
            x0 = x;
            y0 = y;
            code0 = outcode(x0, y0, width, height);
        } else {
            x1 = x;
            y1 = y;
            code1 = outcode(x1, y1, width, height);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_segment_inside_the_frame_keeps_its_endpoints() {
        assert_eq!(
            clip_line(Point::new(1, 2), Point::new(6, 4), 8, 5),
            Some((Point::new(1, 2), Point::new(6, 4)))
        );
    }

    #[test]
    fn a_segment_crossing_the_left_edge_is_cut_at_the_edge() {
        assert_eq!(
            clip_line(Point::new(-10, 2), Point::new(4, 2), 8, 5),
            Some((Point::new(0, 2), Point::new(4, 2)))
        );
    }

    #[test]
    fn a_segment_entirely_outside_is_dropped() {
        assert_eq!(clip_line(Point::new(-10, 0), Point::new(-2, 4), 8, 5), None);
    }

    #[test]
    fn a_segment_through_a_corner_is_clipped_without_hanging() {
        assert_eq!(
            clip_line(Point::new(-4, -4), Point::new(4, 4), 8, 5),
            Some((Point::new(0, 0), Point::new(4, 4)))
        );
    }
}
