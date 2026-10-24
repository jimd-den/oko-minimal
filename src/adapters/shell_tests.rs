//! The frame on the screen: what the terminal is told, and when.

use super::runtime::Runtime;
use super::shell::present;
use super::terminal::Terminal;
use super::terminal_tests::Pty;
use crate::core::event::{Event, Key, KeyCode, Modifiers};

/// A panel with a shape that moves to the right over a second.
const MOVING: &str = "clear #000000\nrect 0 0 4 4 #ff0000 to 4 0 4 4 #ff0000 over 1s";

#[test]
fn a_frame_that_did_not_change_is_not_sent_again() {
    let mut runtime = Runtime::new(8, 4);
    runtime.document("clear #ff0000").expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");
    out.clear();

    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn a_picture_that_moved_is_sent_as_damage() {
    let mut runtime = Runtime::new(8, 4);
    runtime.document(MOVING).expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");
    out.clear();

    runtime.tick(500);
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    let sent = String::from_utf8_lossy(&out);
    assert!(sent.contains("\x1b["), "the cells were addressed: {sent:?}");
    assert!(
        out.len() < 200,
        "the damage is smaller than the frame: {} bytes",
        out.len()
    );
}

#[test]
fn the_escape_chord_hands_the_screen_back_to_the_console() {
    let mut runtime = Runtime::new(8, 4);
    runtime
        .document("on key escape quit")
        .expect("a valid document");
    let mut screen = None;
    let mut out: Vec<u8> = Vec::new();
    present(&mut out, &mut screen, &runtime).expect("the frame is presented");

    let effect = runtime.event(Event::Key(Key::new(KeyCode::Escape, Modifiers::CTRL_ALT)));

    assert_eq!(effect, Some(crate::core::session::Effect::ReclaimDisplay));
}

/// The alternate screen is taken before the first frame is painted.
///
/// The regression this pins: the first `present` ran on the main screen, so
/// the damage baseline was a frame the terminal never showed, and every
/// later frame diffed against a picture that was not on the glass. The only
/// honest witness is the order of the bytes a terminal receives, which means
/// a real terminal: a pty, with its window sized so the frame has cells.
#[test]
fn the_alternate_screen_is_taken_before_the_first_frame_is_painted() {
    let pty = Pty::open();
    size(&pty);
    // The escape chord, queued before the shell starts: two escapes are the
    // way back, so the loop ends on its first read and the test does not wait.
    write_all(pty.master, b"\x1b\x1b");
    let mut terminal = Terminal::open(pty.slave).expect("a terminal");
    let path = std::env::temp_dir().join("oko-shell-first-frame.oko");
    std::fs::write(&path, "clear #05070a\nrect 0 0 2 2 #ff9d00").expect("a document");

    let mut out: Vec<u8> = Vec::new();
    super::shell::run(&path, &mut out, &mut terminal).expect("the console runs");

    let sent = String::from_utf8_lossy(&out);
    assert!(
        sent.starts_with("\u{1b}[?1049h"),
        "the first bytes are the alternate screen, not a frame: {sent:?}"
    );
}

/// An escape pressed alone releases the terminal before the next key.
///
/// The regression this pins: a lone escape byte waited in the decoder for
/// the rest of a sequence, the loop's timed-out reads never flushed it, and
/// the next key arrived glued to it — escape and `x` decoded as alt+x, and
/// the focused terminal swallowed both, so the panel never saw its key.
/// The witness needs the keys apart in time, because the bug is a timeout
/// that never fires: the gaps below are thirty times the loop's 16ms frame.
/// The escape chord at the end is the rescue: it ends the run on the broken
/// code too, so the test fails on its assertion instead of hanging.
#[test]
fn escape_releases_the_terminal_so_the_next_key_reaches_the_panel() {
    let pty = Pty::open();
    size(&pty);
    let master = pty.master;
    let keys = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(100));
        write_all(master, b"\x1b");
        std::thread::sleep(std::time::Duration::from_millis(500));
        write_all(master, b"x");
        std::thread::sleep(std::time::Duration::from_millis(500));
        write_all(master, b"\x1b\x1b");
    });
    let mut terminal = Terminal::open(pty.slave).expect("a terminal");
    let path = std::env::temp_dir().join("oko-shell-escape-release.oko");
    std::fs::write(
        &path,
        "flag shown\nclear #05070a\nrect 0 0 4 4 #123456 when shown\nterm 10 10 60 16 console 1 #ff9d00\ninput console\non key x set shown true",
    )
    .expect("a document");

    let mut out: Vec<u8> = Vec::new();
    super::shell::run(&path, &mut out, &mut terminal).expect("the console runs");
    keys.join().expect("the keys were sent");

    let sent = String::from_utf8_lossy(&out);
    assert!(
        sent.contains("38;2;18;52;86"),
        "the panel key painted the shape it sets: {sent:?}"
    );
}

/// Give the pty a size, so the frame it is handed has cells to paint.
fn size(pty: &Pty) {
    let size = libc::winsize {
        ws_row: 24,
        ws_col: 80,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let set = unsafe { libc::ioctl(pty.master, libc::TIOCSWINSZ, &size) };
    assert_eq!(set, 0, "the pty is sized");
}

/// Write bytes to a descriptor, as the terminal's other end would.
fn write_all(fd: libc::c_int, bytes: &[u8]) {
    let written = unsafe { libc::write(fd, bytes.as_ptr().cast::<libc::c_void>(), bytes.len()) };
    assert_eq!(written, bytes.len() as isize, "the bytes are written");
}
