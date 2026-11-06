//! The `extern "C"` surface the preview page calls. Each export is a
//! thin wrapper over the logic in `lib.rs`; nothing is decided here.

use crate::queries::{dimensions, flag_bits, is_looping, write_term};
use crate::{ERROR, SCRATCH, STATE, init_panel, press_key, read_str, render_frame};

/// Load panel `0` (LCARS) or `1` (modern 1080p). See `init_panel`.
#[unsafe(no_mangle)]
pub extern "C" fn preview_init(which: u32) -> i32 {
    init_panel(which)
}

/// Render at `now_ms`; see `render_frame` for the bytes' lifetime.
#[unsafe(no_mangle)]
pub extern "C" fn preview_render(now_ms: u64) -> *const u8 {
    render_frame(now_ms)
}

#[unsafe(no_mangle)]
pub extern "C" fn preview_width() -> u32 {
    dimensions().0
}

#[unsafe(no_mangle)]
pub extern "C" fn preview_height() -> u32 {
    dimensions().1
}

/// A key press. See `press_key` for the code and return values.
#[unsafe(no_mangle)]
pub extern "C" fn preview_key(code: u32) -> i32 {
    press_key(code)
}

/// The value of a flag: 1 true, 0 false, -1 no such flag.
#[unsafe(no_mangle)]
pub extern "C" fn preview_flag(ptr: *const u8, len: usize) -> i32 {
    let name = read_str(ptr, len);
    STATE.with(
        |state| match state.borrow().as_ref().and_then(|p| p.panel.flag(&name)) {
            Some(true) => 1,
            Some(false) => 0,
            None => -1,
        },
    )
}

/// Flag bits for the page's chips; see `flag_bits`.
#[unsafe(no_mangle)]
pub extern "C" fn preview_flags() -> u32 {
    flag_bits()
}

/// Write text into a terminal viewport; see `write_term`.
#[unsafe(no_mangle)]
pub extern "C" fn preview_write_term(
    term_ptr: *const u8,
    term_len: usize,
    text_ptr: *const u8,
    text_len: usize,
) -> i32 {
    let term = read_str(term_ptr, term_len);
    let text = read_str(text_ptr, text_len);
    write_term(&term, &text)
}

#[unsafe(no_mangle)]
pub extern "C" fn preview_error_ptr() -> *const u8 {
    ERROR.with(|error| error.borrow().as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn preview_error_len() -> usize {
    ERROR.with(|error| error.borrow().len())
}

#[unsafe(no_mangle)]
pub extern "C" fn preview_is_looping() -> i32 {
    is_looping()
}

/// Pointer to the 2048-byte scratch buffer for passing strings in.
#[unsafe(no_mangle)]
pub extern "C" fn preview_scratch_ptr() -> *mut u8 {
    SCRATCH.with(|scratch| scratch.borrow_mut().as_mut_ptr())
}
