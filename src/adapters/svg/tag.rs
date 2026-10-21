//! Reading one tag: its name, its attributes, and how it ends.
//!
//! The scanner decides where a tag starts; this module decides what is inside
//! it. Closing tags produce an `End` event, everything else a `Start`, and a
//! self-closing tag is marked so the translator knows it has no children.
//!
//! An attribute value is decoded here, because a value is what carries the
//! grammar: `font-family: &quot;Open Sans&quot;` names a font the exporter
//! wrote, and refusing the file over it would be refusing a real export.

use super::cursor::Cursor;
use super::entity;
use super::error::SvgError;
use super::xml::{Element, Event, MAX_ATTRIBUTES};

pub fn read(cursor: &mut Cursor<'_>) -> Result<Option<Event>, SvgError> {
    if cursor.begins("</") {
        cursor.skip_to(cursor.at + 2);
        let name = cursor.name()?;
        cursor.whitespace();
        cursor.expect(b'>', "expected `>` after a closing tag")?;
        return Ok(Some(Event::End {
            name,
            line: cursor.line,
        }));
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
                return Ok(Some(Event::Start(Element {
                    name,
                    attributes,
                    closed: false,
                    line,
                })));
            }
            Some(b'/') => {
                cursor.skip_to(cursor.at + 1);
                cursor.expect(b'>', "expected `>` after `/`")?;
                return Ok(Some(Event::Start(Element {
                    name,
                    attributes,
                    closed: true,
                    line,
                })));
            }
            Some(_) => {
                if attributes.len() >= MAX_ATTRIBUTES {
                    return Err(cursor.error("an element has too many attributes"));
                }
                let key = cursor.name()?;
                cursor.whitespace();
                cursor.expect(b'=', "expected `=` after an attribute name")?;
                cursor.whitespace();
                let at = cursor.line;
                let value = entity::decode(&cursor.quoted()?, at)?;
                if attributes.iter().any(|(existing, _)| existing == &key) {
                    return Err(cursor.error(format!("`{key}` appears twice")));
                }
                attributes.push((key, value));
            }
            None => return Err(cursor.error("an unterminated tag")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tag_keeps_its_name_and_attributes() {
        let mut cursor = Cursor::new("<rect x=\"2\" y='3' fill=\"#000000\"/>");

        let Some(Event::Start(element)) = read(&mut cursor).expect("a tag") else {
            panic!("a start tag was expected");
        };
        assert_eq!(element.name, "rect");
        assert_eq!(element.attribute("x"), Some("2"));
        assert_eq!(element.attribute("fill"), Some("#000000"));
        assert!(element.closed);
    }

    #[test]
    fn an_entity_in_an_attribute_value_is_decoded() {
        let mut cursor = Cursor::new("<text font-family=\"&quot;Open Sans&quot;\"/>");

        let Some(Event::Start(element)) = read(&mut cursor).expect("a tag") else {
            panic!("a start tag was expected");
        };
        assert_eq!(element.attribute("font-family"), Some("\"Open Sans\""));
    }

    #[test]
    fn an_unknown_entity_in_an_attribute_value_is_refused_by_name() {
        let mut cursor = Cursor::new("<text font-family=\"&nbsp;\"/>");

        let error = read(&mut cursor).expect_err("an entity");

        assert!(error.problem.contains("&nbsp;"));
    }
}
