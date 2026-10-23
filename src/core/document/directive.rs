//! One line of a document, and what it adds to the panel.
//!
//! The grammar is small on purpose: a directive name, then the shape's own
//! arguments, then any motion the shape states, then the flag it is drawn
//! under. The motion and the condition are recognised by their keyword, so
//! a shape never needs to know how long its own fields are.
//!
//! Three kinds of line: a shape, a flag the panel declares, and a key the
//! panel binds.

use super::super::binding::Binding;
use super::super::panel::Flag;
use super::super::scene::Element;
use super::super::typeface::Setting;
use super::fields::{self, Kind};
use super::motion;
use super::panel;
use super::words::Words;

/// What a line of a document adds to the panel.
pub(crate) enum Addition {
    /// A shape, with its motion and its condition.
    Shape(Element),
    /// A flag, declared.
    Flag(Flag),
    /// A key, bound to an action.
    Binding(Binding),
    /// The font the panel is set in.
    Font(Setting),
}

pub(crate) fn parse(source: &str) -> Result<Addition, String> {
    let mut words = Words::new(source)?;
    let name = words.next().unwrap_or_default();
    match name {
        "flag" => Ok(Addition::Flag(panel::flag(&mut words)?)),
        "font" => Ok(Addition::Font(panel::font(&mut words)?)),
        "on" => Ok(Addition::Binding(panel::binding(&mut words)?)),
        _ => {
            let Some(kind) = Kind::of(name) else {
                return Err(format!("unknown directive `{name}`"));
            };
            let shape = fields::read(kind, &mut words)?;
            let motion = motion::read(&mut words, kind)?;
            let condition = panel::condition(&mut words)?;
            words.end()?;
            Ok(Addition::Shape(Element::when(shape, motion, condition)?))
        }
    }
}
