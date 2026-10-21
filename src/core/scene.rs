//! A scene is a display list, not a canvas: an ordered record of what to
//! paint, which any renderer can walk.
//!
//! The distinction matters for hot editing. A frame is expensive and dies at
//! the next present; a scene is small, comparable, and can be rebuilt from a
//! document in microseconds. Renderers are interchangeable because none of
//! them can see anything the scene does not state.
//!
//! An element is a shape and a motion: what to paint, and what it becomes.
//! The scene is a function of time - `at` gives the scene as it is at a
//! moment - but the scene never reads a clock. The runtime owns the clock and
//! asks.

use super::motion::Motion;
use super::shape::Shape;
use super::timeline;

/// One entry of the display list: a shape, and the motion it may follow.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    /// The shape at the start of the element.
    pub shape: Shape,
    /// What the shape becomes, and when.
    pub motion: Motion,
}

impl Element {
    /// An element whose motion may follow its shape.
    ///
    /// The refusal names the shape it is about: the person who wrote the
    /// document has to be told which of their shapes is the wrong one.
    pub fn new(shape: Shape, motion: Motion) -> Result<Self, String> {
        timeline::check(&motion, &shape)?;
        Ok(Self { shape, motion })
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
            })
            .collect();
        Scene { elements }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Rect, Rgba};
    use crate::core::motion::{Leg, Motion};

    fn element(motion: Motion) -> Element {
        Element::new(
            Shape::Rect {
                area: Rect::new(0, 0, 4, 4),
                color: Rgba::WHITE,
            },
            motion,
        )
        .expect("a shape with a motion")
    }

    #[test]
    fn a_scene_without_motion_has_no_length() {
        let mut scene = Scene::new();
        scene.push(element(Motion::none()));

        assert_eq!(scene.duration(), 0);
    }

    #[test]
    fn a_scene_is_as_long_as_its_longest_motion() {
        let mut scene = Scene::new();
        scene.push(element(Motion::new(
            500,
            vec![Leg::new(
                Shape::Rect {
                    area: Rect::new(0, 0, 4, 4),
                    color: Rgba::WHITE,
                },
                2_000,
            )],
        )));
        scene.push(element(Motion::none()));

        assert_eq!(scene.duration(), 2_500);
    }
}
