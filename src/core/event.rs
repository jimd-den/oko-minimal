//! The vocabulary of things that happen.
//!
//! An adapter turns bytes from a device into one of these values and hands it
//! to a pure function. Nothing in `core` reads a device, and nothing here knows
//! what a terminal is. A test writes `Event::Key(...)` and asks what the session
//! does; that is why the loop can be tested without a terminal.
//!
//! Pointing devices join the vocabulary when a surface exists to hit-test.
//! Guessing their shape now would only make them harder to remove.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl Key {
    pub const fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn plain(code: KeyCode) -> Self {
        Self::new(code, Modifiers::NONE)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Function(u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        ctrl: false,
        alt: false,
        shift: false,
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        alt: false,
        shift: false,
    };
    pub const ALT: Self = Self {
        ctrl: false,
        alt: true,
        shift: false,
    };
    pub const CTRL_ALT: Self = Self {
        ctrl: true,
        alt: true,
        shift: false,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Key(Key),
    Resize { width: u32, height: u32 },
}
