//! The runtime: the state of a console, with no device in it.
//!
//! It holds the session, the last good document, and the frame that is on
//! the screen. It is told two things: that an event arrived, and that the
//! document changed. It answers with what should happen: paint this frame,
//! or hand the screen back.
//!
//! The device is outside it. The terminal's raw mode, the bytes it reads,
//! the file's timestamp and the sleep between looks belong to the shell
//! that drives this, and the shell is what talks to the operating system.

use crate::core::document::{self, ParseError};
use crate::core::event::Event;
use crate::core::frame::Frame;
use crate::core::lease::{Generation, Lease, LeaseError, OwnerId};
use crate::core::raster;
use crate::core::scene::Scene;
use crate::core::session::{Effect, Session};

use super::sequence::CHORD;

/// The owner of the display: the console itself.
const OWNER: OwnerId = OwnerId(1);

/// The state of a console that is presenting a document.
pub struct Runtime {
    session: Session,
    scene: Option<Scene>,
    frame: Option<Frame>,
    size: (u32, u32),
    /// The moment the picture was last painted at.
    at: Option<u64>,
    /// Whether the picture has come to rest at the end of the timeline.
    settled: bool,
}

impl Runtime {
    /// A runtime with a frame of the given size, in pixels.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            session: Session::new(CHORD),
            scene: None,
            frame: None,
            size: (width, height),
            at: None,
            settled: false,
        }
    }

    /// Take the display for this session.
    ///
    /// The console owns the screen and grants the lease once; the escape
    /// chord is the only way to lose it.
    pub fn take_display(&mut self) -> Result<Generation, LeaseError> {
        self.session.grant_display(OWNER)
    }

    /// The lease: who may present, and since when.
    pub fn lease(&self) -> &Lease {
        self.session.lease()
    }

    /// A document arrived. The panel is now the frame it describes.
    ///
    /// A document that does not parse leaves the last good panel in place
    /// and says why, with the line it was on.
    pub fn document(&mut self, text: &str) -> Result<(), String> {
        match document::parse(text) {
            Ok(scene) => {
                self.scene = Some(scene);
                self.at = None;
                self.settled = false;
                self.render();
                Ok(())
            }
            Err(errors) => Err(name(&errors)),
        }
    }

    /// An event arrived. The session decides what it means.
    pub fn event(&mut self, event: Event) -> Option<Effect> {
        if let Event::Resize { width, height } = event {
            self.size = (width, height);
            self.render();
        }
        self.session.dispatch(event)
    }

    /// The clock says the time is now. Paint the panel at this moment if the
    /// picture has changed.
    ///
    /// Contract:
    ///   - `now` is the milliseconds since the document arrived.
    ///   - True when the frame is new and should be painted.
    ///   - A panel with no motion is painted once, when it is loaded.
    ///   - After the timeline's end the picture has come to rest and stays.
    pub fn tick(&mut self, now: u64) -> bool {
        let Some(scene) = self.scene.as_ref() else {
            return false;
        };
        if scene.duration() == 0 || self.settled || self.at == Some(now) {
            return false;
        }
        self.at = Some(now);
        if now >= scene.duration() {
            self.settled = true;
        }
        self.render_at(now);
        true
    }

    /// The frame on the screen, if there is one.
    pub fn frame(&self) -> Option<&Frame> {
        self.frame.as_ref()
    }

    /// Draw the last good document again, at the current size.
    fn render(&mut self) {
        let now = self.at.unwrap_or(0);
        self.render_at(now);
    }

    /// Draw the document as it is at this moment.
    fn render_at(&mut self, now: u64) {
        let frame = self
            .scene
            .as_ref()
            .and_then(|scene| raster::render(&scene.at(now), self.size.0, self.size.1));
        self.frame = frame;
    }
}

/// The errors of a document that did not parse, one per line.
fn name(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
