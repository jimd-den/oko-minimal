//! The lines of a document that belong to the panel, not to a shape.
//!
//! A panel declares its flags and binds its keys. The flags are the panel's
//! own content state; the session's lease and focus are about the screen's
//! authority, and the two are not merged (ADR 0009).
//!
//! The lines are:
//!
//! ```text
//! flag cursor
//! flag running true
//! on key enter set cursor true
//! on key escape quit
//! ```
//!
//! A panel may not run a command, read a file, or change anything outside
//! itself, so the actions are the whole vocabulary: set a flag, or quit.

use super::super::binding::{Action, Binding};
use super::super::event::{Key, KeyCode};
use super::super::panel::Flag;
use super::super::scene::Condition;
use super::super::typeface::Setting;
use super::words::{WHEN, Words};

/// A flag, read from the words after `flag`.
///
/// A flag with no value is false: a flag is what a key may turn on, and a
/// panel that has just been drawn has nothing turned on yet.
pub(crate) fn flag(words: &mut Words) -> Result<Flag, String> {
    let name = words.value().ok_or("expected a flag name")?;
    let value = match words.value() {
        None => false,
        Some("true") => true,
        Some("false") => false,
        Some(other) => return Err(format!("a flag is true or false, not `{other}`")),
    };
    words.end()?;
    Ok(Flag::new(name, value))
}

/// The font a document sets its panel in, read from the words after `font`.
///
/// The font belongs to the panel, so a document that changes it changes one
/// thing: the shapes it draws are the shapes it had.
pub(crate) fn font(words: &mut Words) -> Result<Setting, String> {
    let name = words
        .label()
        .map_err(|problem| format!("{problem}; a font is named in double quotes"))?;
    let mut weight = None;
    while let Some(word) = words.value() {
        let (tag, value) = word
            .split_once('=')
            .ok_or_else(|| format!("a font is set at an axis, written `wght=700`, not `{word}`"))?;
        if tag != "wght" {
            return Err(format!("`{tag}` is not an axis a font may be set at"));
        }
        let value: u16 = value
            .parse()
            .map_err(|_| format!("`{value}` is not a weight"))?;
        if !(1..=1000).contains(&value) {
            return Err(format!("a weight is between 1 and 1000, not {value}"));
        }
        weight = Some(value);
    }
    words.end()?;
    Setting::named(&name, weight)
}

/// A binding, read from the words after `on`.
pub(crate) fn binding(words: &mut Words) -> Result<Binding, String> {
    if !words.is(0, "key") {
        return Err("expected `on key`".to_string());
    }
    words.next();
    let name = words.next().unwrap_or_default();
    let Some(key) = key_named(name) else {
        return Err(format!("`{name}` is not a key a panel may bind"));
    };
    let action = action(words)?;
    words.end()?;
    Binding::new(key, action).map_err(|error| error.to_string())
}

/// The action, read from the words after the key.
fn action(words: &mut Words) -> Result<Action, String> {
    let word = words.next().unwrap_or_default();
    match word {
        "quit" => Ok(Action::Quit),
        "set" => {
            let name = words.value().ok_or("expected a flag name")?;
            let value = match words.value() {
                Some("true") => true,
                Some("false") => false,
                Some(other) => return Err(format!("a flag is true or false, not `{other}`")),
                None => return Err("expected `true` or `false`".to_string()),
            };
            Ok(Action::Set {
                flag: name.to_string(),
                value,
            })
        }
        "toggle" => {
            let name = words.value().ok_or("expected a flag name")?;
            Ok(Action::Toggle {
                flag: name.to_string(),
            })
        }
        other => Err(format!(
            "expected `set`, `toggle`, or `quit`, not `{other}`"
        )),
    }
}

/// The condition, read from the words after a shape's own fields.
///
/// The clause comes last, after any motion, so that a line reads as the
/// shape, then what it becomes, then when it is painted at all.
pub(crate) fn condition(words: &mut Words) -> Result<Condition, String> {
    if !words.is(0, WHEN) {
        return Ok(Condition::Always);
    }
    words.next();
    let name = words.value().ok_or("expected a flag name")?;
    Ok(Condition::Flag(name.to_string()))
}

/// The key a name speaks of: a named key, or a single character.
///
/// The escape chord is not here: it is the way back to the console, and no
/// panel may bind it (ADR 0009).
fn key_named(name: &str) -> Option<Key> {
    let code = match name {
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "escape" => KeyCode::Escape,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        _ => {
            let mut characters = name.chars();
            let character = characters.next()?;
            if characters.next().is_some() {
                return None;
            }
            KeyCode::Char(character)
        }
    };
    Some(Key::plain(code))
}
