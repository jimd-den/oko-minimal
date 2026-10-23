//! The shell around the runtime: the file, the terminal, and the loop that
//! joins them.
//!
//! The runtime decides what should happen. This file does it. It reads the
//! document, watches the file for changes, reads the keys the terminal
//! sends, runs the panel's feeds and commands, and presents the frames.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crate::core::session::Effect;

use super::command;
use super::keys::Decoder;
use super::presenter;
use super::runtime::Runtime;
use super::screen::Screen;
use super::terminal::Terminal;

/// How long the loop waits for a key, and how often the picture is looked at.
const FRAME: Duration = Duration::from_millis(16);

const ALTERNATE: &str = "\x1b[?1049h";
const MAIN: &str = "\x1b[?1049l";
const HIDE: &str = "\x1b[?25l";
const SHOW: &str = "\x1b[?25h";

/// Run the console on a document, until the screen is handed back.
pub fn run(
    path: &Path,
    mut out: &mut (impl Write + ?Sized),
    terminal: &mut Terminal,
) -> io::Result<()> {
    let (columns, rows) = terminal.size()?;
    let mut runtime = Runtime::new(columns, rows.saturating_mul(2));
    let mut decoder = Decoder::new();
    let mut screen: Option<Screen> = None;

    let mut seen = stamp(path);
    let mut origin = Instant::now();
    let loaded = if let Some(text) = read(path) {
        load(&mut runtime, &text);
        feed(&mut runtime, 0);
        origin = Instant::now();
        true
    } else {
        false
    };

    write!(out, "{ALTERNATE}{HIDE}")?;
    out.flush()?;
    terminal.set_raw()?;
    let _ = runtime.take_display();

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
                    Some(Effect::Run { term, command: cmd }) => {
                        let text = command::run(&cmd);
                        runtime.append(&term, &text);
                        present(&mut out, &mut screen, &runtime)?;
                    }
                    Some(Effect::Deliver { .. }) | None => {}
                }
            }
        }
        let stamp = stamp(path);
        if stamp != seen {
            seen = stamp;
            if let Some(text) = read(path) {
                load(&mut runtime, &text);
                feed(&mut runtime, 0);
                origin = Instant::now();
                present(&mut out, &mut screen, &runtime)?;
            }
        }
        let now = origin.elapsed().as_millis() as u64;
        if feed(&mut runtime, now) | runtime.tick(now) {
            present(&mut out, &mut screen, &runtime)?;
        }
    }

    write!(out, "{SHOW}{MAIN}")?;
    out.flush()
}

fn feed(runtime: &mut Runtime, now: u64) -> bool {
    let mut changed = false;
    for (term, cmd) in runtime.due_feeds(now) {
        let text = command::run(&cmd);
        changed |= runtime.output(&term, &text);
    }
    changed
}

fn load(runtime: &mut Runtime, text: &str) {
    if let Err(problem) = runtime.document(text) {
        eprintln!("{problem}");
    }
}

/// Put the frame on the screen.
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

fn read(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(error) => {
            eprintln!("oko: {}: {error}", path.display());
            None
        }
    }
}

fn stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}
