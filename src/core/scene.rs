//! A scene is a display list, not a canvas: an ordered record of what to
//! paint, which any renderer can walk.
//!
//! The distinction matters for hot editing. A frame is expensive and dies at
//! the next present; a scene is small, comparable, and can be rebuilt from a
//! document in microseconds. Renderers are interchangeable because none of
//! them can see anything the scene does not state.
//!
//! An element is a shape, a motion and a condition: what to paint, what it
//! becomes, and when it is painted at all. The scene is a function of time -
//! `at` gives the scene as it is at a moment - but the scene never reads a
//! clock. The runtime owns the clock and asks.
//!
//! The condition names a flag. The flags live in the panel that holds the
//! scene, so here a condition is only a name; the panel says whether the
//! name is true.

use super::motion::Motion;
use super::shape::Shape;
use super::timeline;
use super::typeface::Setting;

/// The condition under which an element is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// Drawn whatever the flags are.
    Always,
    /// Drawn only while the named flag is true.
    Flag(String),
}

/// One entry of the display list: a shape, the motion it may follow, and the
/// condition under which it is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    /// The shape at the start of the element.
    pub shape: Shape,
    /// What the shape becomes, and when.
    pub motion: Motion,
    /// The condition under which the element is drawn.
    pub condition: Condition,
}

impl Element {
    /// An element whose motion may follow its shape, drawn always.
    ///
    /// The refusal names the shape it is about: the person who wrote the
    /// document has to be told which of their shapes is the wrong one.
    pub fn new(shape: Shape, motion: Motion) -> Result<Self, String> {
        Self::when(shape, motion, Condition::Always)
    }

    /// An element that is drawn only while its condition holds.
    ///
    /// Why a separate name and not a third argument to `new`: an element
    /// without a condition is the common case, and the document layer should
    /// not have to say `Always` to say nothing.
    pub fn when(shape: Shape, motion: Motion, condition: Condition) -> Result<Self, String> {
        timeline::check(&motion, &shape)?;
        Ok(Self {
            shape,
            motion,
            condition,
        })
    }

    /// The shape as it is at `now` milliseconds.
    pub fn shape_at(&self, now: u64) -> Shape {
        timeline::at(&self.motion, &self.shape, now)
    }
}

/// A display list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    elements: Vec<Element>,
    /// The font the scene is drawn in, if it is drawn in one.
    font: Option<Setting>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, element: Element) {
        self.elements.push(element);
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    /// The font the scene is drawn in, if it is drawn in one.
    ///
    /// The font travels with the display list, not with a shape: the scene
    /// is what the raster walks, and the raster paints the text in the
    /// scene's font (ADR 0014).
    pub fn font(&self) -> Option<&Setting> {
        self.font.as_ref()
    }

    /// The font the scene is drawn in.
    pub fn set_font(&mut self, font: Option<Setting>) {
        self.font = font;
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// The scene as it is at `now` milliseconds.
    ///
    /// A pure function of the scene and the moment: the same scene and the
    /// same moment always give the same scene.
    pub fn at(&self, now: u64) -> Scene {
        let elements = self
            .elements
            .iter()
            .map(|element| Element {
                shape: element.shape_at(now),
                motion: Motion::none(),
                condition: element.condition.clone(),
            })
            .collect();
        Scene {
            elements,
            font: self.font.clone(),
        }
    }

    /// The scene of the elements whose condition holds.
    ///
    /// A pure function of the scene and the predicate: the panel decides what
    /// holds, because the flags live there and not here.
    pub fn satisfying(&self, holds: impl Fn(&Condition) -> bool) -> Scene {
        Scene {
            elements: self
                .elements
                .iter()
                .filter(|element| holds(&element.condition))
                .cloned()
                .collect(),
            font: self.font.clone(),
        }
    }

    /// The length of the scene: the moment the last shape stops changing.
    ///
    /// A scene with no motion has length zero. A runtime that wraps its clock
    /// must not wrap it by zero.
    pub fn duration(&self) -> u64 {
        self.elements
            .iter()
            .map(|element| timeline::duration(&element.motion))
            .max()
            .unwrap_or(0)
    }
}
