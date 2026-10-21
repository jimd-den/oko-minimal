//! Reading the stream of tags an SVG export contains.
//!
//! This is not an XML parser. It reads elements, attributes, comments, the XML
//! declaration and runs of character data, and it refuses doctypes. The
//! entity references an export writes are decoded, because a real export
//! writes them and refusing a file over a font family with a space in it is
//! refusing the export. That is the whole input surface of the translator, and
//! keeping the surface this small is what makes "bounded" a property of the
//! code rather than a promise.
//!
//! Every limit is a refusal, never a truncation: too many elements, too many
//! attributes, an oversized value, or an unterminated construct stops with a
//! line number.

use super::cursor::Cursor;
use super::entity;
use super::error::SvgError;
use super::tag;
use super::xml::{Event, MAX_ELEMENTS};

pub fn events(source: &str) -> Result<Vec<Event>, SvgError> {
    let mut cursor = Cursor::new(source);
    let mut events = Vec::new();

    while let Some(byte) = cursor.byte() {
        let event = if byte == b'<' {
            header(&mut cursor)?
        } else {
            text(&mut cursor)?
        };
        if let Some(event) = event {
            if events.len() >= MAX_ELEMENTS {
                return Err(SvgError::new(
                    cursor.line,
                    "the document has too many elements",
                ));
            }
            events.push(event);
        }
    }

    Ok(events)
}

/// Comments and the XML declaration carry no meaning for a panel, and are
/// skipped. Everything else that begins with `<` is a tag.
fn header(cursor: &mut Cursor<'_>) -> Result<Option<Event>, SvgError> {
    if cursor.begins("<!--") {
        cursor.skip_past("-->", "an unterminated comment")?;
        return Ok(None);
    }
    if cursor.begins("<?") {
        cursor.skip_past("?>", "an unterminated processing instruction")?;
        return Ok(None);
    }
    if cursor.begins("<!") {
        return Err(cursor.error("declarations are not allowed"));
    }
    tag::read(cursor)
}

/// A run of character data.
///
/// Whitespace between elements carries nothing, so it stays ignored. A run
/// with a word in it is delivered as the author wrote it, with the entity
/// references decoded: `&amp;` is the character `&`, which is what the
/// author typed.
fn text(cursor: &mut Cursor<'_>) -> Result<Option<Event>, SvgError> {
    let source = cursor.source();
    let rest = &source.as_bytes()[cursor.at..];
    let end = cursor.at
        + rest
            .iter()
            .position(|&byte| byte == b'<')
            .unwrap_or(rest.len());
    let line = cursor.line;
    let content = source[cursor.at..end].to_string();
    cursor.skip_to(end);

    let content = entity::decode(&content, line)?;
    if content.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(Event::Text { content, line }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_declaration_and_comments_are_skipped() {
        let events = events("<?xml version=\"1.0\"?><!-- a note --><g></g>").expect("a group");

        assert_eq!(events.len(), 2);
    }

    #[test]
    fn a_doctype_is_refused() {
        let error = events("<!DOCTYPE svg>").expect_err("a doctype");

        assert!(error.problem.contains("declarations"));
    }

    #[test]
    fn a_run_of_text_is_delivered() {
        let events = events("<text>hello</text>").expect("a label");

        assert_eq!(events.len(), 3);
        assert_eq!(
            events[1],
            Event::Text {
                content: "hello".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn an_entity_in_a_run_is_decoded() {
        let events = events("<text>a&amp;b</text>").expect("a label");

        assert_eq!(
            events[1],
            Event::Text {
                content: "a&b".to_string(),
                line: 1,
            }
        );
    }

    #[test]
    fn an_unknown_entity_in_a_run_is_refused_by_name() {
        let error = events("<text>a&nbsp;b</text>").expect_err("an entity");

        assert_eq!(error.line, 1);
        assert!(error.problem.contains("&nbsp;"));
    }
}
