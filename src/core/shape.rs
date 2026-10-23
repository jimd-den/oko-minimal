//! The shapes a panel is made of.
//!
//! A shape is what the display list has always held: a clear, a rectangle, a
//! line, a text, a path. It says what to paint and nothing else. It carries
//! no clock and no history - what a shape becomes over time is a separate
//! thing, a motion, and the two travel together in the scene's element.
//!
//! Nothing here reads the clock, touches a device, or knows a document.

use super::geom::{Point, Rect, Rgba};
use super::path::Path;

/// What a display list paints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shape {
    /// Paint the whole frame, discarding what is under it.
    Clear(Rgba),
    /// A filled rectangle.
    Rect { area: Rect, color: Rgba },
    /// A straight line.
    Line { from: Point, to: Point, color: Rgba },
    /// A label, drawn in the built-in font.
    ///
    /// The spacing is the space the design put between the letters, on top
    /// of the font's own advance (ADR 0010).
    Text {
        at: Point,
        text: String,
        scale: u8,
        color: Rgba,
        spacing: i64,
    },
    /// A path, with the paints it carries. Either paint may be absent: a path
    /// that is only filled is not outlined, and one that is only stroked is
    /// not filled.
    Path {
        path: Path,
        fill: Option<Rgba>,
        stroke: Option<Rgba>,
    },
}

impl Shape {
    /// The name of this kind of shape, for a message that names it.
    pub fn kind(&self) -> &'static str {
        match self {
            Shape::Clear(_) => "a clear",
            Shape::Rect { .. } => "a rectangle",
            Shape::Line { .. } => "a line",
            Shape::Text { .. } => "a text",
            Shape::Path { .. } => "a path",
        }
    }

    /// Whether this shape may become `other`.
    ///
    /// A refusal names the shapes it is about: the person who wrote the
    /// document has to be told which of their shapes is the wrong one.
    pub fn can_become(&self, other: &Shape) -> Result<(), String> {
        match (self, other) {
            (Shape::Clear(_), _) | (_, Shape::Clear(_)) => Err(format!(
                "a clear has nothing to move to, and a {} cannot become one",
                self.kind()
            )),
            (Shape::Rect { .. }, Shape::Rect { .. }) => Ok(()),
            (Shape::Line { .. }, Shape::Line { .. }) => Ok(()),
            (
                Shape::Text {
                    text: one,
                    spacing: one_spacing,
                    ..
                },
                Shape::Text {
                    text: other,
                    spacing: other_spacing,
                    ..
                },
            ) if one == other && one_spacing == other_spacing => Ok(()),
            (Shape::Text { .. }, Shape::Text { .. }) => {
                Err("a text cannot become a different text".to_string())
            }
            (Shape::Path { path: one, .. }, Shape::Path { path: other, .. })
                if same_shape(one, other) =>
            {
                Ok(())
            }
            (Shape::Path { .. }, Shape::Path { .. }) => {
                Err("a path can only move to a path of the same shape".to_string())
            }
            (one, other) => Err(format!("a {} cannot become a {}", one.kind(), other.kind())),
        }
    }
}

/// Whether two paths have the same shape: the same segments, in the same order.
///
/// A path that moves to a path of another shape has no meaning, because
/// there is nothing to say which point of the first is which point of the
/// second.
fn same_shape(one: &Path, other: &Path) -> bool {
    one.segments.len() == other.segments.len()
        && one
            .segments
            .iter()
            .zip(other.segments.iter())
            .all(|(one, other)| std::mem::discriminant(one) == std::mem::discriminant(other))
}
