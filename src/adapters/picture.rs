//! The picture on the screen: the frame, and the scene it was painted from.
//!
//! The frame is made once and painted into again, so a console that
//! presents sixty pictures a second writes into the same memory every
//! time. The scene the frame was painted from is kept beside it: a scene
//! that has not changed paints the same picture, and a picture that has
//! not changed costs nothing to show.

use crate::core::frame::Frame;
use crate::core::scene::Scene;

use super::painter;

/// The picture on the screen.
pub struct Picture {
    frame: Option<Frame>,
    /// The scene the frame was painted from.
    scene: Option<Scene>,
    /// The size of the screen, in pixels.
    size: (u32, u32),
    /// How many frame buffers have been made.
    frames: usize,
}

impl Picture {
    /// A picture of the given size, in pixels, with nothing painted yet.
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            frame: None,
            scene: None,
            size,
            frames: 0,
        }
    }

    /// The frame on the screen, if there is one.
    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    /// How many frame buffers have been made.
    ///
    /// The picture is painted into one buffer and written into again, so
    /// this counts the moments the size changed and not the ticks.
    pub fn frames_allocated(&self) -> usize {
        self.frames
    }

    /// The screen is a different size now.
    ///
    /// The picture is painted again even when the scene has not changed:
    /// the frame that is there does not fit the terminal any more.
    pub fn resize(&mut self, size: (u32, u32)) {
        self.size = size;
        self.scene = None;
    }

    /// Paint the scene, unless it is the picture that is already there.
    ///
    /// Returns true when the picture changed.
    pub fn paint(&mut self, scene: Scene) -> bool {
        if self.scene.as_ref() == Some(&scene) {
            return false;
        }
        let Some(mut frame) = self.buffer() else {
            return false;
        };
        painter::paint_into(&scene, &mut frame);
        self.frame = Some(frame);
        self.scene = Some(scene);
        true
    }

    /// The frame the picture is painted into.
    ///
    /// The frame is made once and painted into again and again. A frame of
    /// another size is a new frame: the terminal is a different size now.
    fn buffer(&mut self) -> Option<Frame> {
        let size = self.size;
        let existing = self.frame.take();
        let fits = existing
            .as_ref()
            .is_some_and(|frame| (frame.width(), frame.height()) == size);
        if fits {
            return existing;
        }
        self.frames += 1;
        Frame::new(size.0, size.1)
    }
}
