//! WASM preview of the Star Trek panels on this branch.
//!
//! A preview shim, not a fork: it reuses the branch's pure core verbatim
//! through `#[path]` — `document::parse` for the `.oko` text,
//! `Panel::scene(now)` for the display list at a moment, and
//! `raster::render_into` for pixels. No adapter is compiled in: the core
//! never touches a file, device, or terminal, which is why it can run in
//! a browser at all. The shell, PTY, feeds, and `libc` terminal layer
//! stay native and are not in this binary. The `extern` surface lives in
//! `exports.rs`; this file holds the state and the logic.

#[path = "../../src/core/mod.rs"]
pub mod core;

mod exports;
mod queries;
mod seed;

use core::document;
use core::event::{Key, KeyCode};
use core::frame::Frame;
use core::geom::Rgba;
use core::panel::{Outcome, Panel};
use core::raster;
use std::cell::RefCell;

/// The branch's Star Trek panel, 52 lines, small cell-sized coordinates.
const LCARS: &str = include_str!("../../examples/lcars.oko");
/// The heavy 1080p Star Trek design, 1920x1080, set in Inter.
const MODERN: &str = include_str!("../modern-1080.oko");

pub(crate) struct Preview {
    panel: Panel,
    frame: Frame,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

thread_local! {
    pub(crate) static STATE: RefCell<Option<Preview>> = const { RefCell::new(None) };
    pub(crate) static ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    /// Scratch bytes the page writes into before calling an export that
    /// takes a string pointer. Single-threaded WASM: this address stays
    /// valid for the life of the page.
    pub(crate) static SCRATCH: RefCell<[u8; 2048]> = const { RefCell::new([0; 2048]) };
}

pub(crate) fn set_error(message: String) {
    ERROR.with(|error| *error.borrow_mut() = message);
}

pub(crate) fn read_str(ptr: *const u8, len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // The caller is the preview page; the slice is its own memory.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// Load panel `0` (LCARS) or `1` (modern 1080p). Returns 0, or a negative
/// code; the message is available through the error exports.
pub(crate) fn init_panel(which: u32) -> i32 {
    let (text, width, height) = match which {
        0 => (LCARS, 80, 48),
        1 => (MODERN, 1920, 1080),
        _ => {
            set_error(format!("no panel {which}"));
            return -1;
        }
    };
    let mut panel = match document::parse(text) {
        Ok(panel) => panel,
        Err(errors) => {
            set_error(
                errors
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("; "),
            );
            return -2;
        }
    };
    seed::seed_terminals(&mut panel);
    let Some(frame) = Frame::new(width, height) else {
        set_error("frame too large".to_string());
        return -3;
    };
    STATE.with(|state| {
        *state.borrow_mut() = Some(Preview {
            panel,
            frame,
            bytes: Vec::new(),
            width,
            height,
        });
    });
    0
}

/// Render at `now_ms`, returning a pointer to RGBA bytes
/// (`width * height * 4`) that live until the next render call.
pub(crate) fn render_frame(now_ms: u64) -> *const u8 {
    STATE.with(|state| {
        let mut borrowed = state.borrow_mut();
        let Some(preview) = borrowed.as_mut() else {
            return std::ptr::null();
        };
        let scene = preview.panel.scene(now_ms);
        preview.frame.clear(Rgba::BLACK);
        raster::render_into(&mut preview.frame, &scene);
        preview.bytes.clear();
        for y in 0..preview.height {
            if let Some(row) = preview.frame.row(y) {
                for pixel in row {
                    preview
                        .bytes
                        .extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
                }
            }
        }
        preview.bytes.as_ptr()
    })
}

/// A key press: a character code, 13 Enter, 27 Escape. Returns 0
/// unchanged, 1 changed, 2 run (shown, not executed), 3 quit.
pub(crate) fn press_key(code: u32) -> i32 {
    let key_code = match code {
        13 => KeyCode::Enter,
        27 => KeyCode::Escape,
        8 => KeyCode::Backspace,
        9 => KeyCode::Tab,
        other => match char::from_u32(other) {
            Some(character) => KeyCode::Char(character),
            None => return 0,
        },
    };
    STATE.with(|state| {
        let mut borrowed = state.borrow_mut();
        let Some(preview) = borrowed.as_mut() else {
            return 0;
        };
        match preview.panel.key(Key::plain(key_code)) {
            Outcome::Unchanged => 0,
            Outcome::Changed => 1,
            Outcome::Run { term, command } => {
                preview.panel.append(
                    &term,
                    &format!("\n$ {command}\n[preview] commands do not run in the browser"),
                );
                2
            }
            Outcome::Quit => 3,
        }
    })
}
