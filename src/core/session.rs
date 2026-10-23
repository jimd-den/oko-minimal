//! One display session: the lease it holds, the surface it has focused, and
//! what an event asks of the runtime.
//!
//! `dispatch` is the reducer the loop drives. It has no clock and no I/O: an
//! event goes in, at most one effect comes out. The runtime performs effects;
//! the session decides. That split is what lets every rule here be tested
//! without a terminal, a device or a child process.

use super::event::{Event, Key};
use super::focus::{Focus, Route, SurfaceId};
use super::lease::{Generation, Lease, LeaseError, OwnerId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// The console takes the screen back; the session's lease is gone.
    ReclaimDisplay,
    Deliver {
        surface: SurfaceId,
        key: Key,
    },
    /// Run a command and write its output to the named terminal viewport.
    Run {
        term: String,
        command: String,
    },
    Repaint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Session {
    lease: Lease,
    focus: Focus,
}

impl Session {
    pub fn new(escape: Key) -> Self {
        Self {
            lease: Lease::new(),
            focus: Focus::new(escape),
        }
    }

    pub fn lease(&self) -> &Lease {
        &self.lease
    }

    pub fn grant_display(&mut self, owner: OwnerId) -> Result<Generation, LeaseError> {
        self.lease.grant(owner)
    }

    pub fn revoke_display(&mut self) -> Option<Generation> {
        self.lease.revoke()
    }

    pub fn focus(&mut self, surface: SurfaceId) {
        self.focus.focus(surface);
    }

    pub fn dispatch(&mut self, event: Event) -> Option<Effect> {
        match event {
            Event::Key(key) => match self.focus.route(key) {
                Route::Escape => {
                    self.revoke_display();
                    Some(Effect::ReclaimDisplay)
                }
                Route::Surface(surface) => Some(Effect::Deliver { surface, key }),
                Route::Dropped => None,
            },
            Event::Resize { .. } => Some(Effect::Repaint),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{KeyCode, Modifiers};

    fn escape() -> Key {
        Key::new(KeyCode::Escape, Modifiers::CTRL_ALT)
    }

    #[test]
    fn the_escape_chord_reclaims_the_display_from_a_focused_surface() {
        let mut session = Session::new(escape());
        session.grant_display(OwnerId(1)).expect("a grant");
        session.focus(SurfaceId(7));

        assert_eq!(
            session.dispatch(Event::Key(escape())),
            Some(Effect::ReclaimDisplay)
        );
        assert_eq!(session.lease().holder(), None);
    }

    #[test]
    fn a_key_goes_to_the_focused_surface() {
        let mut session = Session::new(escape());
        session.focus(SurfaceId(7));
        let key = Key::plain(KeyCode::Char('a'));

        assert_eq!(
            session.dispatch(Event::Key(key)),
            Some(Effect::Deliver {
                surface: SurfaceId(7),
                key,
            })
        );
    }

    #[test]
    fn a_key_with_nothing_focused_asks_for_nothing() {
        let mut session = Session::new(escape());

        assert_eq!(
            session.dispatch(Event::Key(Key::plain(KeyCode::Enter))),
            None
        );
    }

    #[test]
    fn a_resize_asks_for_a_repaint() {
        let mut session = Session::new(escape());

        assert_eq!(
            session.dispatch(Event::Resize {
                width: 80,
                height: 48,
            }),
            Some(Effect::Repaint)
        );
    }
}
