//! Reading one tag: its name, its attributes, and how it ends.
//!
//! The scanner decides where a tag starts; this module decides what is inside
//! it. A closing tag names the element it closes, and a self-closing tag is
//! marked so the translator knows it has no children.
//!
//! Attribute values are decoded as they are read: an export writes a font
//! family with a space in it as `&quot;Open Sans&quot;`, and refusing the
//! entity would refuse the whole file over a character the author never typed.

use super::cursor::Cursor;
use super::entity;
use super::error::SvgError;
use super::xml::MAX_ATTRIBUTES;

/// One tag, read.
#[derive(Clone, Debug)]
pub enum Tag {
    /// A start tag, self-closing or not.
    Open {
        name: String,
        attributes: Vec<(String, String)>,
        line: usize,
        /// The tag closes itself: `<rect/>`.
        empty: bool,
    },
    /// A closing tag.
    Close { name: String, line: usize },
}

/// Read one tag.
pub fn read(cursor: &mut Cursor<'_>) -> Result<Tag, SvgError> {
    if cursor.begins("</") {
        cursor.skip_to(cursor.at + 2);
        let line = cursor.line;
        let name = cursor.name()?;
        cursor.whitespace();
        cursor.expect(b'>', "expected `>` after a closing tag")?;
        return Ok(Tag::Close { name, line });
    }

    cursor.skip_to(cursor.at + 1);
    let line = cursor.line;
    let name = cursor.name()?;
    let mut attributes = Vec::new();

    loop {
        cursor.whitespace();
        match cursor.byte() {
            Some(b'>') => {
                cursor.skip_to(cursor.at + 1);
                return Ok(Tag::Open {
                    name,
                    attributes,
                    line,
                    empty: false,
                });
            }
            Some(b'/') => {
                cursor.skip_to(cursor.at + 1);
                cursor.expect(b'>', "expected `>` after `/`")?;
                return Ok(Tag::Open {
                    name,
                    attributes,
                    line,
                    empty: true,
                });
            }
            Some(_) => {
                if attributes.len() >= MAX_ATTRIBUTES {
                    return Err(cursor.error("an element has too many attributes"));
                }
                let key = cursor.name()?;
                cursor.whitespace();
                cursor.expect(b'=', "expected `=` after an attribute name")?;
                cursor.whitespace();
                let value = cursor.quoted()?;
                if attributes.iter().any(|(existing, _)| existing == &key) {
                    return Err(cursor.error(format!("`{key}` appears twice")));
                }
                attributes.push((key, entity::decode(&value, line)?));
            }
            None => return Err(cursor.error("an unterminated tag")),
        }
    }
}
