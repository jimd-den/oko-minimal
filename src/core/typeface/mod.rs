//! A real font: the reading of it, borrowed from skrifa.
//!
//! The project keeps the drawing and borrows the reading (ADR 0011).
//! `skrifa` reads TrueType outlines, CFF outlines, the character map, the
//! advances and a variable font's deltas; it does not shape, hint or draw.
//! A glyph's outline arrives here as a path, and the core's own path
//! painter fills it, so a glyph is a path and not a second renderer.
//!
//! What is not here is shaping: there is no kerning, no ligature and no
//! hinting. A character the face does not have is a hole in the panel, and
//! the panel says so.
//!
//! The faces are compiled into the program, and their licences are beside
//! them in `assets/fonts`. The program carries three: a monospaced face, a
//! proportional face and a variable one.

use std::sync::LazyLock;

use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, MetadataProvider, Tag};

pub mod draw;
#[cfg(test)]
mod draw_tests;

use super::geom::Point;
use super::path::{Path, Segment};

/// The faces the program carries.
const FACES: &[(&str, &[u8])] = &[
    (
        "dejavu-mono",
        include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf"),
    ),
    (
        "source-sans-3",
        include_bytes!("../../../assets/fonts/SourceSans3-Regular.otf"),
    ),
    (
        "inter",
        include_bytes!("../../../assets/fonts/InterVariable.ttf"),
    ),
];

/// The grid a glyph is drawn on: four times finer than the panel's.
///
/// The core's path holds whole pixels, and a glyph's outline is not whole
/// pixels. The outline is scaled up by this factor, drawn on the finer
/// grid, and brought down (ADR 0013).
pub const GRID: f32 = 4.0;

/// A face: a font file and the font that reads it.
///
/// Two faces are the same face when they carry the same name: the file
/// behind a name is the program's business, and a panel's font is compared
/// by what it asks for.
#[derive(Clone)]
pub struct Face {
    name: &'static str,
    font: FontRef<'static>,
}

impl std::fmt::Debug for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl PartialEq for Face {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Face {}

impl Face {
    fn new(name: &'static str, bytes: &'static [u8]) -> Option<Self> {
        Some(Face {
            name,
            font: FontRef::new(bytes).ok()?,
        })
    }

    /// The name the face is known by.
    pub fn name(&self) -> &str {
        self.name
    }

    /// The face with that name, if the program carries it.
    pub fn by_name(name: &str) -> Option<&'static Face> {
        READ.iter().find(|face| face.name == name)
    }

    /// The names of the faces the program carries.
    pub fn names() -> Vec<&'static str> {
        READ.iter().map(|face| face.name).collect()
    }

    /// The glyph for a character, at a size in pixels.
    ///
    /// The size is the em size: a size of sixteen is sixteen pixels from the
    /// baseline to the top of the em. A weight is a number from 100 to 900;
    /// a face without weights takes the weight it was drawn with.
    pub fn glyph(&self, character: char, size: f32, weight: Option<u16>) -> Option<Glyph> {
        let id = self.font.charmap().map(character)?;
        let outline = self.font.outline_glyphs().get(id)?;
        let mut location = self.location(weight);
        let place = LocationRef::new(location.coords_mut());
        let em = Size::new(size);
        let settings = DrawSettings::unhinted(em, place);
        let mut pen = PathPen::default();
        outline.draw(settings, &mut pen).ok()?;

        let advance = self.font.glyph_metrics(em, place).advance_width(id)?;
        Some(Glyph {
            path: pen.path,
            advance: advance * GRID,
        })
    }

    /// The height of a capital, as a fraction of the em.
    ///
    /// The built-in font's capitals are as tall as its cell, so a face drawn
    /// to the same capital height is drawn at the same size as the built-in
    /// font at the same scale (ADR 0014). A face that does not say how tall
    /// its capitals are is drawn as if they were seven tenths of the em.
    pub fn cap_ratio(&self, weight: Option<u16>) -> f32 {
        let mut location = self.location(weight);
        let place = LocationRef::new(location.coords_mut());
        let metrics = self.font.metrics(Size::unscaled(), place);
        let Some(cap) = metrics.cap_height else {
            return 0.7;
        };
        cap / f32::from(metrics.units_per_em)
    }

    /// Where on the face's axes the glyph is drawn.
    ///
    /// A face without a weight axis takes the weight it was drawn with: the
    /// weight is a place on the face's axes, and a face with no axes has no
    /// other place.
    fn location(&self, weight: Option<u16>) -> Location {
        let has_weight = self
            .font
            .axes()
            .iter()
            .any(|axis| axis.tag() == Tag::new(b"wght"));
        match weight {
            Some(weight) if has_weight => self.font.axes().location(&[("wght", f32::from(weight))]),
            _ => Location::new(0),
        }
    }
}

/// The faces, read once.
static READ: LazyLock<Vec<Face>> = LazyLock::new(|| {
    FACES
        .iter()
        .filter_map(|(name, bytes)| Face::new(name, bytes))
        .collect()
});

/// The font a panel is set in: the name of a face, and the weight it is
/// set at.
///
/// The font belongs to the panel and not to a shape. A panel that changes
/// its font changes one thing, and the shapes it draws are the shapes it
/// had; the letterforms are looked up again when the text is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    /// The face the panel is set in.
    pub face: &'static Face,
    /// The weight, for a face that has weights.
    pub weight: Option<u16>,
}

impl Setting {
    /// The setting a name asks for.
    ///
    /// A face the program does not carry is refused by name: a panel that
    /// asks for a typeface that is not here is told so, and is not drawn in
    /// some other face instead.
    pub fn named(name: &str, weight: Option<u16>) -> Result<Self, String> {
        let face = Face::by_name(name)
            .ok_or_else(|| format!("the program carries no face named `{name}`"))?;
        Ok(Self { face, weight })
    }
}

/// A glyph: the path of its outline, and the room it takes.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    /// The outline, on the fine grid, with the baseline at zero and the y
    /// axis pointing down, as the panel's is.
    pub path: Path,
    /// How far the pen moves after the glyph, on the fine grid.
    pub advance: f32,
}

/// A pen that writes a glyph's outline into the core's path.
#[derive(Default)]
struct PathPen {
    path: Path,
}

impl PathPen {
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
