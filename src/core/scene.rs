//! A scene is a display list, not a canvas: an ordered record of what to
//! paint, which any renderer can walk.
//!
//! The distinction matters for hot editing. A frame is expensive and dies at
//! the next present; a scene is small, comparable, and can be rebuilt from a
//! document in microseconds. Renderers are interchangeable because none of
//! them can see anything the scene does not state.

use super::geom::{Point, Rect, Rgba};
use super::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// Paint the whole frame, discarding what is under it.
    Clear(Rgba),
    Rect {
        area: Rect,
        color: Rgba,
    },
    Line {
        from: Point,
        to: Point,
        color: Rgba,
    },
    Text {
        at: Point,
        text: String,
        scale: u8,
        color: Rgba,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    ops: Vec<Op>,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, op: Op) {
        self.ops.push(op);
    }

    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}
