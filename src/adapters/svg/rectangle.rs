//! Recognising a rectangle in a straight outline.
//!
//! Most filled shapes in an instrument panel are rectangles, and the document
//! format has a rectangle primitive. The translator accepts only an outline
//! that really is the four corners of one. Anything else is refused by the
//! caller rather than approximated by scan lines, because an approximation is
//! a picture nobody can explain from the source.

use super::path::Outline;
use crate::core::geom::{Point, Rect};

pub fn from_outline(outline: &Outline) -> Option<Rect> {
    if !outline.closed || outline.points.len() != 4 {
        return None;
    }
    let points = &outline.points;
    let left = points.iter().map(|point| point.x).min()?;
    let right = points.iter().map(|point| point.x).max()?;
    let top = points.iter().map(|point| point.y).min()?;
    let bottom = points.iter().map(|point| point.y).max()?;
    if right <= left || bottom <= top {
        return None;
    }

    let on_corner = |point: &Point| {
        (point.x == left || point.x == right) && (point.y == top || point.y == bottom)
    };
    let adjacent = |a: &Point, b: &Point| a.x == b.x || a.y == b.y;
    if !points.iter().all(on_corner) {
        return None;
    }
    if !points.windows(2).all(|pair| adjacent(&pair[0], &pair[1]))
        || !adjacent(&points[3], &points[0])
    {
        return None;
    }

    Some(Rect::new(
        left,
        top,
        (right - left) as u32,
        (bottom - top) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::svg::path::outline;

    #[test]
    fn a_closed_four_corner_path_is_a_rectangle() {
        let outline = outline("M2 2 H8 V6 H2 Z", 1).expect("a rectangle");

        assert_eq!(from_outline(&outline), Some(Rect::new(2, 2, 6, 4)));
    }

    #[test]
    fn an_open_path_is_not_a_rectangle() {
        let outline = outline("M2 2 H8 V6 H2", 1).expect("an open path");

        assert_eq!(from_outline(&outline), None);
    }
}
