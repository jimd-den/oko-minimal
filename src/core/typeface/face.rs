//! Reading a font file: the faces the program carries, and what a face can
//! say about itself.
//!
//! The faces are compiled into the program with `include_bytes!`, and their
//! licences sit beside them in `assets/fonts`. A face is the file and the
//! reader over it; the outline itself is written by `pen`, and the core's
//! path painter draws it.
//!
//! What a face gives is plain data: a path and an advance, in the core's
//! own types. Nothing here draws, and nothing here shapes.

use std::sync::LazyLock;

use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::DrawSettings;
use skrifa::{FontRef, MetadataProvider, Tag};

use super::GRID;
use super::pen::PathPen;
use crate::core::path::Path;

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

    /// The glyph for a character, at an em size in pixels.
    ///
    /// The size is the em size: a size of sixteen is sixteen pixels from the
    /// baseline to the top of the em. A weight is a number from 100 to 900;
    /// a face without weights takes the weight it was drawn with.
    pub fn glyph(&self, character: char, em_px: f32, weight: Option<u16>) -> Option<Glyph> {
        let id = self.font.charmap().map(character)?;
        let outline = self.font.outline_glyphs().get(id)?;
        let mut location = self.location(weight);
        let place = LocationRef::new(location.coords_mut());
        let em = Size::new(em_px);
        let settings = DrawSettings::unhinted(em, place);
        let mut pen = PathPen::default();
        outline.draw(settings, &mut pen).ok()?;

        let advance = self.font.glyph_metrics(em, place).advance_width(id)?;
        Some(Glyph {
            path: pen.into_path(),
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

/// A glyph: the path of its outline, and the room it takes.
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    /// The outline, on the fine grid, with the baseline at zero and the y
    /// axis pointing down, as the panel's is.
    pub path: Path,
    /// How far the pen moves after the glyph, on the fine grid.
    pub advance: f32,
}
