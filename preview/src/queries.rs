//! Reading the preview's state: flags, size, and terminal writes.
//!
//! Nothing here changes what is on screen; the driving half — loading a
//! panel, rendering a frame, answering a key — lives in `lib.rs`.

use crate::STATE;

/// Flag bits for the page's chips: tactical 1, engineering 2, science 4,
/// alert 8.
pub(crate) fn flag_bits() -> u32 {
    STATE.with(|state| {
        let borrowed = state.borrow();
        let Some(preview) = borrowed.as_ref() else {
            return 0;
        };
        let mut bits = 0;
        for (name, bit) in [
            ("tactical", 1),
            ("engineering", 2),
            ("science", 4),
            ("alert", 8),
        ] {
            if preview.panel.flag(name) == Some(true) {
                bits |= bit;
            }
        }
        bits
    })
}

/// Write text into a terminal viewport; returns 1 when it exists.
pub(crate) fn write_term(term: &str, text: &str) -> i32 {
    STATE.with(|state| {
        let mut borrowed = state.borrow_mut();
        let Some(preview) = borrowed.as_mut() else {
            return 0;
        };
        i32::from(preview.panel.write(term, text))
    })
}

/// The frame's size in pixels, or (0, 0) before a panel is loaded.
pub(crate) fn dimensions() -> (u32, u32) {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map_or((0, 0), |p| (p.width, p.height))
    })
}

/// Whether the loaded panel loops its motion forever.
pub(crate) fn is_looping() -> i32 {
    STATE.with(|state| {
        state
            .borrow()
            .as_ref()
            .map_or(0, |p| i32::from(p.panel.is_looping()))
    })
}
