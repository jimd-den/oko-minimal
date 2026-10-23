//! The shell around the runtime: the file, the terminal, and the loop that
//! joins them.
//!
//! The runtime decides what should happen. This file does it. It reads the
//! document, watches the file for changes, reads the keys the terminal
//! sends, and presents the frames. Everything here touches a device, and
//! none of it decides anything.
//!
//! The panel is presented on the alternate screen, so it does not scroll the
//! person's history away, and the cursor is hidden, because a cursor
//! blinking on top of a panel is a cursor on top of a panel.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crate::core::session::Effect;

use super::keys::Decoder;
use super::presenter;
use super::runtime::Runtime;
use super::screen::Screen;
use super::terminal::Terminal;

/// How long the loop waits for a key, and how often the picture is looked at.
///
/// The panel is a picture that moves, so the loop turns at the frame rate.
/// The file's timestamp is read on the same turn: a stat is cheap, and a
/// hundred-millisecond poll was a number chosen for the file rather than
/// for the picture.
const FRAME: Duration = Duration::from_millis(16);

/// The alternate screen: the panel does not scroll the history away.
const ALTERNATE: &str = "\x1b[?1049h";
const MAIN: &str = "\x1b[?1049l";
/// The cursor, hidden while the panel is up.
const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";

/// Run the console on a document, until the screen is handed back.
pub fn run(path: &Path) -> io::Result<()> {
    let mut terminal = Terminal::stdin()?;
    let (columns, rows) = terminal.size()?;
    let mut runtime = Runtime::new(columns, rows.saturating_mul(2));
    let mut decoder = Decoder::new();
    let mut out = io::stdout();
    // The cells the terminal is showing: what the damage is measured from.
    let mut screen: Option<Screen> = None;

    let mut seen = stamp(path);
    let mut origin = Instant::now();
    let loaded = if let Some(text) = read(path) {
        load(&mut runtime, &text);
        origin = Instant::now();
        true
    } else {
        false
    };

    write!(out, "{ALTERNATE}{HIDE}")?;
    out.flush()?;
    terminal.set_raw()?;
    let _ = runtime.take_display();

    // The first frame is painted only once the alternate screen is taken,
    // so the baseline the damage is measured from is a frame the terminal
    // actually shows. Painting before the alternate screen left the first
    // frame on the main screen and made every later frame a diff against
    // a picture that was never on the glass.
    if loaded {
        present(&mut out, &mut screen, &runtime)?;
    }

    let mut buffer = [0u8; 64];
    let mut running = true;
    while running {
        let count = terminal.read(&mut buffer, FRAME)?;
        if count > 0 {
            decoder.feed(&buffer[..count]);
            while let Some(event) = decoder.event() {
                match runtime.event(event) {
                    Some(Effect::ReclaimDisplay) => running = false,
                    Some(Effect::Repaint) => present(&mut out, &mut screen, &runtime)?,
                    Some(Effect::Deliver { .. }) | None => {}
                }
            }
        }
        let stamp = stamp(path);
        if stamp != seen {
            seen = stamp;
            if let Some(text) = read(path) {
                load(&mut runtime, &text);
                origin = Instant::now();
                present(&mut out, &mut screen, &runtime)?;
            }
        }
        if runtime.tick(origin.elapsed().as_millis() as u64) {
            present(&mut out, &mut screen, &runtime)?;
        }
    }

    write!(out, "{SHOW}{MAIN}")?;
    out.flush()
}

/// Give the runtime a document, and say why it did not take it.
///
/// A document that does not parse leaves the last good panel on the screen:
/// an editor that blanks the panel on every keystroke of a half-typed line
/// is an editor that is always empty.
fn load(runtime: &mut Runtime, text: &str) {
    if let Err(problem) = runtime.document(text) {
        eprintln!("{problem}");
    }
}

/// Put the frame on the screen.
///
/// The first frame is painted in full: the terminal has none of it yet.
/// The frames after that are sent as damage - only the cells that changed
/// - so a picture that has not moved costs nothing at all.
pub fn present(
    out: &mut impl Write,
    screen: &mut Option<Screen>,
    runtime: &Runtime,
) -> io::Result<()> {
    let Some(frame) = runtime.frame() else {
        return Ok(());
    };
    let fits = screen.as_ref().is_some_and(|screen| screen.fits(frame));
    let bytes = if fits {
        let screen = screen.as_mut().expect("a screen that fits");
        presenter::damage(screen.sync(frame))
    } else {
        let mut fresh = Screen::of(frame);
        fresh.reset(frame);
        *screen = Some(fresh);
        presenter::screen(frame)
    };
    out.write_all(bytes.as_bytes())?;
    out.flush()
}

/// Read the document, if it can be read.
fn read(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            eprintln!("oko: {}: {error}", path.display());
            None
        }
    }
}

/// When the file was last changed, which is how a change is noticed.
fn stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
