//! The fields of a shape, and the fields of the state it becomes.
//!
//! A shape's own fields and the fields of the state it becomes are read by
//! the same rules: a rectangle is `x y w h colour` whether it opens a line
//! or follows a `to`. One reader, so a shape and the state it becomes
//! cannot drift apart.
//!
//! What a target is, is read from its own words. A path command names a
//! path and a quoted label names a text; numbers name a rectangle or a
//! line, and which of the two the source is, because the two write the same
//! words. A target of another kind is refused here, by name, so the person
//! who wrote it is told which of their shapes is the wrong one.

use super::super::font;
use super::super::geom::{Point, Rect};
use super::super::shape::Shape;
use super::path;
use super::words::{SPACING, Words};

/// The kinds of shape a document may name.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Clear,
    Rect,
    Line,
    Text,
    Path,
}

impl Kind {
    /// The kind a directive's name asks for.
    pub(crate) fn of(name: &str) -> Option<Kind> {
        match name {
            "clear" => Some(Kind::Clear),
            "rect" => Some(Kind::Rect),
            "line" => Some(Kind::Line),
            "text" => Some(Kind::Text),
            "path" => Some(Kind::Path),
            _ => None,
        }
    }

    /// The name of this kind, for a message that names it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Kind::Clear => "clear",
            Kind::Rect => "rectangle",
            Kind::Line => "line",
            Kind::Text => "text",
            Kind::Path => "path",
        }
    }
}

/// The shape a kind's fields make, read from the next words.
pub(crate) fn read(kind: Kind, words: &mut Words) -> Result<Shape, String> {
    match kind {
        Kind::Clear => Ok(Shape::Clear(words.colour()?)),
        Kind::Rect => Ok(Shape::Rect {
            area: area(words)?,
            color: words.colour()?,
        }),
        Kind::Line => Ok(Shape::Line {
            from: point(words)?,
            to: point(words)?,
            color: words.colour()?,
        }),
        Kind::Text => text(words),
        Kind::Path => path::read(words),
    }
}

/// The state a leg names, read from the next words.
///
/// A target of another kind is refused here, by name: the shape and the
/// target would be refused by the core as well, but the core says "a
/// rectangle cannot become a text", which reads as though the reader had
/// mixed up two shapes.
pub(crate) fn target(source: Kind, words: &mut Words) -> Result<Shape, String> {
    let kind = kind_of(source, words);
    if kind != source {
        return Err(format!(
            "a {} cannot become a {}",
            source.name(),
            kind.name()
        ));
    }
    read(kind, words)
}

/// The kind a target's own words name.
fn kind_of(source: Kind, words: &Words) -> Kind {
    if source == Kind::Clear {
        return Kind::Clear;
    }
    if is_path_command(words.peek(0)) {
        return Kind::Path;
    }
    if words.peek(2).is_some_and(|word| word.starts_with('"')) {
        return Kind::Text;
    }
    match source {
        Kind::Line => Kind::Line,
        _ => Kind::Rect,
    }
}

/// Whether a word starts a path.
fn is_path_command(word: Option<&str>) -> bool {
    matches!(word, Some("M" | "L" | "C" | "Q" | "Z"))
}

/// A point, read from the next two words.
fn point(words: &mut Words) -> Result<Point, String> {
    let x = words.coordinate()?;
    let y = words.coordinate()?;
    Ok(Point::new(x, y))
}

/// An area, read from the next four words.
fn area(words: &mut Words) -> Result<Rect, String> {
    let x = words.coordinate()?;
    let y = words.coordinate()?;
    let w = words.extent()?;
    let h = words.extent()?;
    Ok(Rect::new(x, y, w, h))
}

/// A text, read from the next words.
fn text(words: &mut Words) -> Result<Shape, String> {
    let at = point(words)?;
    let label = words.label()?;
    for character in label.chars() {
        if !font::supports(character) {
            return Err(format!("`{character}` is not in the built-in font"));
        }
    }
    let scale = words.scale()?;
    let color = words.colour()?;
    let spacing = if words.is(0, SPACING) {
        words.next();
        words.number()?
    } else {
        0
    };
    Ok(Shape::Text {
        at,
        text: label,
        scale,
        color,
        spacing,
    })
}
