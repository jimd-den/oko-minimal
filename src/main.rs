//! The composition root: the only file that knows about files and terminals.
//!
//! It reads a document, asks the adapter to paint it, and writes the bytes. A
//! future loop will turn this one-shot paint into a watch: read again when the
//! file changes, dispatch keys, present on demand.

use std::env;
use std::fs;
use std::process::ExitCode;

/// The terminal's default shape, in character cells.
const DEFAULT_COLUMNS: u32 = 80;
const DEFAULT_ROWS: u32 = 24;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: oko <panel.oko> [columns rows]");
        return ExitCode::FAILURE;
    };

    let columns = number(args.next(), DEFAULT_COLUMNS);
    let rows = number(args.next(), DEFAULT_ROWS);

    let document = match fs::read_to_string(&path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("oko: {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    match oko::adapters::terminal::paint(&document, columns, rows.saturating_mul(2)) {
        Ok(bytes) => {
            print!("{bytes}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprint!("{path}: {error}");
            ExitCode::FAILURE
        }
    }
}

fn number(word: Option<String>, fallback: u32) -> u32 {
    word.and_then(|word| word.parse().ok()).unwrap_or(fallback)
}
