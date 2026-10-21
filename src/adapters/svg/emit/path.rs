//! Writing the path directive: the segments of a shape, then its paints.
//!
//! The document keeps the curve, so the writer says `C` where the shape
//! curves and `L` where it does not. The words a segment is written as are
//! the document grammar's words, and they are the same words the core's
//! parser reads, so a panel written here is a panel the core can load.
//!
//! A path with no paint at all is not written: there is nothing to draw, and
//! a directive that says nothing is a hole in the document.

use super::{Writer, coordinate, hex};
use crate::adapters::svg::error::SvgError;
use crate::core::geom::{Point, Rgba};
use crate::core::path::{Path, Segment};

/// Write a path directive: the segments, then the paints.
pub fn write(
    document: &mut Writer,
    shape: &Path,
    fill: Option<Rgba>,
    stroke: Option<Rgba>,
    line: usize,
) -> Result<(), SvgError> {
    if fill.is_none() && stroke.is_none() {
        return Ok(());
    }
    let mut text = String::from("path");
    for segment in &shape.segments {
        text.push_str(&format!(" {}", words(segment, line)?));
    }
    for (name, colour) in [("fill", fill), ("stroke", stroke)] {
        if let Some(colour) = colour {
            text.push_str(&format!(" {name} {}", hex(colour)));
        }
    }
    document.write(&format!("{text}\n"), line)
}

/// The words one segment is written as.
fn words(segment: &Segment, line: usize) -> Result<String, SvgError> {
    let at = |point: Point| {
        coordinate(point.x, line)?;
        coordinate(point.y, line)?;
        Ok(format!("{} {}", point.x, point.y))
    };
    Ok(match segment {
        Segment::Move { to } => format!("M {}", at(*to)?),
        Segment::Line { to } => format!("L {}", at(*to)?),
        Segment::Cubic {
            control1,
            control2,
            to,
        } => format!("C {} {} {}", at(*control1)?, at(*control2)?, at(*to)?),
        Segment::Quadratic { control, to } => format!("Q {} {}", at(*control)?, at(*to)?),
        Segment::Close => "Z".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::Point;

    fn shape() -> Path {
        Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(2, 2),
                },
                Segment::Cubic {
                    control1: Point::new(4, 0),
                    control2: Point::new(6, 0),
                    to: Point::new(8, 2),
                },
                Segment::Close,
            ],
        }
    }

    #[test]
    fn a_path_becomes_one_document_line() {
        let mut document = Writer::default();

        write(
            &mut document,
            &shape(),
            Some(Rgba::rgb(255, 157, 0)),
            None,
            1,
        )
        .expect("a path");

        assert_eq!(
            document.into_text(),
            "path M 2 2 C 4 0 6 0 8 2 Z fill #ff9d00\n"
        );
    }

    #[test]
    fn a_path_with_no_paint_is_not_written() {
        let mut document = Writer::default();

        write(&mut document, &shape(), None, None, 1).expect("a path");

        assert_eq!(document.into_text(), "");
    }
}
