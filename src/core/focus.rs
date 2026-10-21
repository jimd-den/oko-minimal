//! Where a key goes: one focused surface, or nowhere.
//!
//! The escape chord is checked before the focus is consulted, because it is the
//! way back to the console when a session has stopped answering. Everything
//! else goes to the focused surface, and is dropped when nothing has focus —
//! dropped, not queued, so a session that never reads cannot grow a backlog.

use super::event::Key;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Escape,
    Surface(SurfaceId),
    Dropped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Focus {
    focused: Option<SurfaceId>,
    escape: Key,
}

impl Focus {
    pub fn new(escape: Key) -> Self {
        Self {
            focused: None,
            escape,
        }
    }

    pub fn focused(&self) -> Option<SurfaceId> {
        self.focused
    }

    pub fn focus(&mut self, surface: SurfaceId) {
        self.focused = Some(surface);
    }

    pub fn blur(&mut self) {
        self.focused = None;
    }

    pub fn route(&self, key: Key) -> Route {
        if key == self.escape {
            return Route::Escape;
        }
        match self.focused {
            Some(surface) => Route::Surface(surface),
            None => Route::Dropped,
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
    fn the_reserved_escape_comes_before_any_surface() {
        let mut focus = Focus::new(escape());
        focus.focus(SurfaceId(7));

        assert_eq!(focus.route(escape()), Route::Escape);
    }

    #[test]
    fn input_goes_to_the_focused_surface() {
        let mut focus = Focus::new(escape());
        focus.focus(SurfaceId(7));

        assert_eq!(
            focus.route(Key::plain(KeyCode::Char('a'))),
            Route::Surface(SurfaceId(7))
        );
    }

    #[test]
    fn input_is_dropped_when_nothing_has_focus() {
        let focus = Focus::new(escape());

        assert_eq!(focus.route(Key::plain(KeyCode::Enter)), Route::Dropped);
    }

    #[test]
    fn escape_is_recognised_without_any_focus() {
        let focus = Focus::new(escape());

        assert_eq!(focus.route(escape()), Route::Escape);
    }
}
