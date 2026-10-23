//! The composition root: the only file that knows about files and terminals.
//!
//! It reads the arguments, and it either paints one frame and stops, or
//! hands the screen to the console and waits for the chord that takes it
//! back. Everything else lives in the adapter it calls.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use oko::adapters::{presenter, shell};

/// The terminal's default shape, in character cells.
const DEFAULT_COLUMNS: u32 = 80;
const DEFAULT_ROWS: u32 = 24;

fn main() -> ExitCode {
    let arguments = Arguments::read();

    if arguments.once {
        return paint(&arguments);
    }
    let Ok(mut terminal) = oko::adapters::terminal::Terminal::stdin() else {
        return ExitCode::FAILURE;
    };
    let mut out = std::io::stdout();
    match shell::run(&arguments.path, &mut out, &mut terminal) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("oko: {}: {error}", arguments.path.display());
            ExitCode::FAILURE
        }
    }
}

/// Paint one frame and stop, for a script that wants a frame.
fn paint(arguments: &Arguments) -> ExitCode {
    let Ok(document) = fs::read_to_string(&arguments.path) else {
        eprintln!(
            "oko: {}: cannot read the document",
            arguments.path.display()
        );
        return ExitCode::FAILURE;
    };
    let (columns, rows) = arguments.size();
    match presenter::paint(&document, columns, rows.saturating_mul(2)) {
        Ok(bytes) => {
            print!("{bytes}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: {error}", arguments.path.display());
            ExitCode::FAILURE
        }
    }
}

/// What the program was asked to do.
struct Arguments {
    path: PathBuf,
    columns: Option<u32>,
    rows: Option<u32>,
    once: bool,
}

impl Arguments {
    /// The arguments, read from the command line.
    fn read() -> Arguments {
        let mut path = None;
        let mut columns = None;
        let mut rows = None;
        let mut once = false;

        for word in env::args().skip(1) {
            match word.as_str() {
                "--once" => once = true,
                other => {
                    if path.is_none() {
                        path = Some(PathBuf::from(other));
                    } else if columns.is_none() {
                        columns = other.parse().ok();
                    } else {
                        rows = other.parse().ok();
                    }
                }
            }
        }
        Arguments {
            path: path.unwrap_or_else(|| PathBuf::from("panel.oko")),
            columns,
            rows,
            once,
        }
    }

    /// The size of the panel, in character cells.
    fn size(&self) -> (u32, u32) {
        (
            self.columns.unwrap_or(DEFAULT_COLUMNS),
            self.rows.unwrap_or(DEFAULT_ROWS),
        )
    }
}
