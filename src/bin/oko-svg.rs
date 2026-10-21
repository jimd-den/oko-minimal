//! Translate an SVG export into a panel document on standard output.
//!
//! `oko-svg panel.svg > panel.oko` is the whole authoring bridge: draw in
//! Penpot or Figma, export SVG, translate, then edit the text or watch it.

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
        Ok(document) => {
            print!("{document}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{path}:{error}");
            ExitCode::FAILURE
        }
    }
}
