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

#[cfg(test)]
mod tests {
    use super::*;

    /// The name and attributes of the first tag in a source.
    fn start(source: &str) -> (String, Vec<(String, String)>) {
        let mut cursor = Cursor::new(source);
        match read(&mut cursor).expect("a tag") {
            Tag::Open {
                name, attributes, ..
            } => (name, attributes),
            Tag::Close { .. } => panic!("a start tag was expected"),
        }
    }

    #[test]
    fn a_tag_keeps_its_name_and_attributes() {
        let (name, attributes) = start("<rect x=\"2\" y='3' fill=\"#000000\"/>");

        assert_eq!(name, "rect");
        assert!(attributes.contains(&("x".to_string(), "2".to_string())));
        assert!(attributes.contains(&("fill".to_string(), "#000000".to_string())));
    }

    #[test]
    fn an_entity_in_an_attribute_value_is_decoded() {
        let (_, attributes) = start("<text font-family=\"&quot;Open Sans&quot;\"/>");

        assert!(attributes.contains(&("font-family".to_string(), "\"Open Sans\"".to_string())));
    }

    #[test]
    fn an_unknown_entity_in_an_attribute_value_is_refused_by_name() {
        let mut cursor = Cursor::new("<text font-family=\"&nbsp;\"/>");

        let error = read(&mut cursor).expect_err("an entity");

        assert!(error.problem.contains("&nbsp;"));
    }
}
