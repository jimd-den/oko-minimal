//! The runtime: the state of a console, with no device in it.
//!
//! It holds the session, the last good document, and the picture that is
//! on the screen. It is told that an event arrived, that a command produced
//! output, or that the document changed. It answers with what should happen:
//! paint this frame, run this command, or hand the screen back.

use crate::core::document::{self, ParseError};
use crate::core::event::Event;
use crate::core::frame::Frame;
use crate::core::lease::{Generation, Lease, LeaseError, OwnerId};
use crate::core::panel::{Outcome, Panel};
use crate::core::session::{Effect, Session};

use super::picture::Picture;
use super::sequence::CHORD;

/// The owner of the display: the console itself.
const OWNER: OwnerId = OwnerId(1);

/// The state of a console that is presenting a document.
pub struct Runtime {
    session: Session,
    panel: Option<Panel>,
    picture: Picture,
    at: Option<u64>,
    fed_at: Option<u64>,
    settled: bool,
}

impl Runtime {
    /// A runtime with a frame of the given size, in pixels.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            session: Session::new(CHORD),
            panel: None,
            picture: Picture::new((width, height)),
            at: None,
            fed_at: None,
            settled: false,
        }
    }

    /// Take the display for this session.
    pub fn take_display(&mut self) -> Result<Generation, LeaseError> {
        self.session.grant_display(OWNER)
    }

    /// The lease: who may present, and since when.
    pub fn lease(&self) -> &Lease {
        self.session.lease()
    }

    /// A document arrived. The panel is now the frame it describes.
    pub fn document(&mut self, text: &str) -> Result<(), String> {
        match document::parse(text) {
            Ok(panel) => {
                self.panel = Some(panel);
                self.at = None;
                self.fed_at = None;
                self.settled = false;
                self.render();
                Ok(())
            }
            Err(errors) => Err(name(&errors)),
        }
    }

    /// Replace the text in the named terminal viewport and repaint.
    pub fn output(&mut self, term: &str, text: &str) -> bool {
        let Some(panel) = self.panel.as_mut() else {
            return false;
        };
        if !panel.write(term, text) {
            return false;
        }
        self.render();
        true
    }

    /// Append text to the named terminal viewport and repaint.
    pub fn append(&mut self, term: &str, text: &str) -> bool {
        let Some(panel) = self.panel.as_mut() else {
            return false;
        };
        if !panel.append(term, text) {
            return false;
        }
        self.render();
        true
    }

    /// The `(term, command)` feeds due to run at `now`.
    pub fn due_feeds(&mut self, now: u64) -> Vec<(String, String)> {
        let Some(panel) = self.panel.as_ref() else {
            return Vec::new();
        };
        let due = panel.due_feeds(self.fed_at, now);
        self.fed_at = Some(now);
        due
    }

    /// An event arrived. The panel answers first, then the session.
    pub fn event(&mut self, event: Event) -> Option<Effect> {
        if let Event::Resize { width, height } = event {
            self.picture.resize((width, height));
            self.render();
            return self.session.dispatch(event);
        }
        if let Event::Key(key) = event
            && let Some(panel) = self.panel.as_mut()
        {
            match panel.key(key) {
                Outcome::Unchanged => {}
                Outcome::Changed => {
                    self.settled = false;
                    self.render();
                    return Some(Effect::Repaint);
                }
                Outcome::Run { term, command } => {
                    self.settled = false;
                    self.render();
                    return Some(Effect::Run { term, command });
                }
                Outcome::Quit => return Some(Effect::ReclaimDisplay),
            }
        }
        self.session.dispatch(event)
    }

    /// The clock says the time is now. Paint the panel at this moment if the
    /// picture has changed.
    pub fn tick(&mut self, now: u64) -> bool {
        let Some(panel) = self.panel.as_ref() else {
            return false;
        };
        if panel.duration() == 0 || self.settled || self.at == Some(now) {
            return false;
        }
        self.at = Some(now);
        if !panel.is_looping() && now >= panel.duration() {
            self.settled = true;
        }
        self.render_at(now)
    }

    pub fn frame(&self) -> Option<&Frame> {
        self.picture.frame()
    }

    pub fn frames_allocated(&self) -> usize {
        self.picture.frames_allocated()
    }

    fn render(&mut self) {
        let now = self.at.unwrap_or(0);
        self.render_at(now);
    }

    fn render_at(&mut self, now: u64) -> bool {
        let Some(scene) = self.panel.as_ref().map(|panel| panel.scene(now)) else {
            return false;
        };
        self.picture.paint(scene)
    }
}

fn name(errors: &[ParseError]) -> String {
    errors
        .iter()
        .map(|error| error.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}
