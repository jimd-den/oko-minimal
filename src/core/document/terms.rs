//! The lines of a document that declare terminal viewports, feeds, and focus.
//!
//! A panel may place named rectangular terminal viewports (`term`), attach
//! command feeds (`feed`) that populate them, and choose which terminal starts
//! with keyboard focus (`input`).

use super::super::geom::Rect;
use super::super::panel::Feed;
use super::super::term::Term;
use super::motion;
use super::panel;
use super::words::{EVERY, Words};

/// A terminal viewport, read from the words after `term`.
///
/// Grammar: `term <x> <y> <w> <h> <name> <scale> <colour> [when <flag>]`.
pub(crate) fn term(words: &mut Words) -> Result<Term, String> {
    let x = words.coordinate()?;
    let y = words.coordinate()?;
    let w = words.extent()?;
    let h = words.extent()?;
    let name = words.value().ok_or("expected a terminal name")?.to_string();
    let scale = words.scale()?;
    let color = words.colour()?;
    let condition = panel::condition(words)?;
    words.end()?;
    Ok(Term::new(
        name,
        Rect::new(x, y, w, h),
        scale,
        color,
        condition,
    ))
}

/// A command feed, read from the words after `feed`.
///
/// Grammar: `feed <name> "<command>" [every <duration>]`.
pub(crate) fn feed(words: &mut Words) -> Result<Feed, String> {
    let name = words.value().ok_or("expected a terminal name")?.to_string();
    let command = words.label()?;
    let every = if words.is(0, EVERY) {
        words.next();
        Some(motion::duration(words)?)
    } else {
        None
    };
    words.end()?;
    Ok(Feed::new(name, command, every))
}

/// The terminal that starts with keyboard focus, read after `input`.
pub(crate) fn input(words: &mut Words) -> Result<String, String> {
    let name = words.value().ok_or("expected a terminal name")?.to_string();
    words.end()?;
    Ok(name)
}
