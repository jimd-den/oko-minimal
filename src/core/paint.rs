//! Painting a path: the non-zero fill and the one-pixel stroke.
//!
//! The renderer fills with the non-zero winding rule, the rule SVG fills by,
//! and strokes along the edges of the shape. A path that states a fill but
//! no close is filled as if the closing edge were there, which is what SVG
//! does.
//!
//! The arithmetic lives here and not in the renderer because it is the part
//! of drawing a path that is genuinely new: a rectangle never needed it.

use super::curve::Sample;
use super::frame::Frame;
use super::geom::{Point, Rect, Rgba};
use super::path::{Path, Polyline, sample};

/// A run of pixels in one row that a filled shape covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub y: i32,
    pub from: i32,
    pub to: i32,
}

/// Paint a path into a frame: the fill first, then the stroke.
pub fn path(frame: &mut Frame, path: &Path, fill: Option<Rgba>, stroke: Option<Rgba>) {
    let polyline = sample(path);
    if let Some(color) = fill {
        fill_shape(frame, &polyline, color);
    }
    if let Some(color) = stroke {
        stroke_shape(frame, &polyline, color);
    }
}

/// Fill a shape with the non-zero winding rule.
fn fill_shape(frame: &mut Frame, polyline: &Polyline, color: Rgba) {
    for span in spans(polyline, frame.bounds()) {
        let area = Rect::new(span.from, span.y, (span.to - span.from) as u32, 1);
        frame.fill(area, color);
    }
}

/// Stroke a shape: a line one pixel wide along its edges.
///
/// The stroke follows the sampled polyline, so a curve is stroked as the
/// curve and not as the straight lines between its samples.
fn stroke_shape(frame: &mut Frame, polyline: &Polyline, color: Rgba) {
    for shape in polyline {
        for pair in shape.windows(2) {
            frame.line(pixel(pair[0]), pixel(pair[1]), color);
        }
    }
}

/// A sampled point, rounded to the pixel it lands on.
fn pixel(at: Sample) -> Point {
    Point::new(at.x.round() as i32, at.y.round() as i32)
}

/// The pixels a shape covers, one span per run of covered pixels.
///
/// Every row is scanned once. The edges that cross the row are sorted, and
/// the winding number between two of them says whether the run of pixels
/// between them is inside the shape.
pub fn spans(polyline: &Polyline, bounds: Rect) -> Vec<Span> {
    let mut spans = Vec::new();
    for row in i64::from(bounds.y)..bounds.bottom() {
        let y = row as i32;
        let mut crossings = crossings(polyline, row as f64 + 0.5);
        crossings.sort_by(|one, other| one.0.total_cmp(&other.0));
        let mut winding = 0;

        for pair in crossings.windows(2) {
            winding += pair[0].1;
            if winding == 0 {
                continue;
            }
            let from = (pair[0].0 - 0.5).ceil().max(f64::from(bounds.x)) as i32;
            let to = (pair[1].0 - 0.5).ceil().min(bounds.right() as f64) as i32;
            if to > from {
                spans.push(Span { y, from, to });
            }
        }
    }
    spans
}

/// Where the edges of a shape cross a scanline, and which way each one goes.
///
/// The shape is read as closed, because a fill always implies the closing
/// edge. An edge that lies along the scanline is not crossed by it.
fn crossings(polyline: &Polyline, scanline: f64) -> Vec<(f64, i32)> {
    let mut crossings = Vec::new();
    for shape in polyline {
        if shape.len() < 2 {
            continue;
        }
        for index in 0..shape.len() {
            let from = shape[index];
            let to = shape[(index + 1) % shape.len()];
            if (from.y <= scanline) == (to.y <= scanline) {
                continue;
            }
            let t = (scanline - from.y) / (to.y - from.y);
            let x = from.x + t * (to.x - from.x);
            crossings.push((x, if to.y > from.y { 1 } else { -1 }));
        }
    }
    crossings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::path::{Path, Segment};

    fn point(x: i32, y: i32) -> Point {
        Point::new(x, y)
    }

    /// A right triangle with its corner at the origin.
    fn triangle() -> Path {
        Path {
            segments: vec![
                Segment::Move { to: point(1, 1) },
                Segment::Line { to: point(9, 1) },
                Segment::Line { to: point(1, 9) },
                Segment::Close,
            ],
        }
    }

    /// A square of a given size, as the segments of one shape.
    fn square(x: i32, y: i32, size: i32) -> Vec<Segment> {
        vec![
            Segment::Move { to: point(x, y) },
            Segment::Line {
                to: point(x + size, y),
            },
            Segment::Line {
                to: point(x + size, y + size),
            },
            Segment::Line {
                to: point(x, y + size),
            },
            Segment::Close,
        ]
    }

    fn covers(spans: &[Span], x: i32, y: i32) -> bool {
        spans
            .iter()
            .any(|span| span.y == y && span.from <= x && span.to > x)
    }

    #[test]
    fn a_triangle_covers_its_interior_and_not_its_outside() {
        let path = triangle();

        let spans = spans(&sample(&path), Rect::new(0, 0, 12, 12));

        assert!(covers(&spans, 2, 2), "the interior is covered");
        assert!(!covers(&spans, 8, 2), "the far side is not covered");
        assert!(!covers(&spans, 2, 9), "the far row is not covered");
    }

    #[test]
    fn a_self_overlapping_shape_fills_under_the_non_zero_rule() {
        let mut segments = square(1, 1, 6);
        segments.extend(square(5, 5, 6));
        let path = Path { segments };

        let spans = spans(&sample(&path), Rect::new(0, 0, 14, 14));

        assert!(
            covers(&spans, 6, 6),
            "the overlap is filled, which the even-odd rule would leave empty"
        );
    }

    #[test]
    fn a_filled_shape_paints_its_interior() {
        let mut frame = Frame::new(12, 12).expect("a small frame");
        let shape = triangle();

        path(&mut frame, &shape, Some(Rgba::WHITE), None);

        assert_eq!(frame.color_at(point(2, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(point(8, 2)), Some(Rgba::BLACK));
    }
}
