//! One line of a document, and what it adds to the scene.
//!
//! The grammar is small on purpose: a directive name, then the shape's own
//! arguments, and then any motion the shape states. The motion is recognised
//! by its keyword, so a shape never needs to know how long its own fields
//! are.

use super::super::scene::Element;
use super::fields::{self, Kind};
use super::motion;
use super::words::Words;

pub(crate) fn parse(source: &str) -> Result<Element, String> {
    let mut words = Words::new(source)?;
    let name = words.next().unwrap_or_default();
    let Some(kind) = Kind::of(name) else {
        return Err(format!("unknown directive `{name}`"));
    };
    let shape = fields::read(kind, &mut words)?;
    let motion = motion::read(&mut words, kind)?;
    words.end()?;
    Element::new(shape, motion)
}
