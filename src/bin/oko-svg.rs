//! Translate an SVG export into a panel document on standard output.
//!
//! `oko-svg panel.svg > panel.oko` is the whole authoring bridge: draw in
//! Penpot or Figma, export SVG, translate, then edit the text or watch it.
//!
//! The document goes to standard output and the report of what could not be
//! drawn goes to standard error, so a pipe keeps only the panel and a person
//! still sees what was left out. A file that cannot be read at all is an
//! error and exits non-zero; a panel with holes is still a panel.

use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: oko-svg <panel.svg>");
        return ExitCode::FAILURE;
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("oko-svg: {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    match oko::adapters::svg::translate(&source) {
        Ok(translation) => {
            print!("{}", translation.document);
            report(&path, &translation);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{path}:{error}");
            ExitCode::FAILURE
        }
    }
}

/// Write what the translation could not draw, one shape per line.
///
/// The report names the line of the export each shape was on, so a person can
/// find it in the file they exported.
fn report(path: &str, translation: &oko::adapters::svg::Translation) {
    if translation.skipped.is_empty() {
        return;
    }
    eprintln!(
        "{path}: {} shapes could not be drawn:",
        translation.skipped.len()
    );
    for skipped in &translation.skipped {
        eprintln!("  {path}:{}: {}", skipped.line, skipped.reason);
    }
}
