//! Reading the stream of tags an SVG export contains.
//!
//! This is not an XML parser. It reads elements, attributes, comments and the
//! XML declaration, and it refuses doctypes, entities and text content. That is
//! the whole input surface of the translator, and keeping the surface this small
//! is what makes "bounded" a property of the code rather than a promise.
//!
//! Every limit is a refusal, never a truncation: too many elements, too many
//! attributes, an oversized value, or an unterminated construct stops with a
//! line number.

use super::cursor::Cursor;
use super::error::SvgError;
use super::tag;
use super::xml::{Event, MAX_ELEMENTS};

pub fn events(source: &str) -> Result<Vec<Event>, SvgError> {
    let mut cursor = Cursor::new(source);
    let mut events = Vec::new();

    while let Some(byte) = cursor.byte() {
        if byte == b'<' {
            if let Some(event) = header(&mut cursor)? {
                if events.len() >= MAX_ELEMENTS {
                    return Err(SvgError::new(
                        cursor.line,
                        "the document has too many elements",
                    ));
                }
                events.push(event);
            }
        } else {
            text(&mut cursor)?;
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
        return Err(cursor.error("declarations and entities are not allowed"));
    }
    tag::read(cursor)
}

fn text(cursor: &mut Cursor<'_>) -> Result<(), SvgError> {
    let source = cursor.source();
    let rest = &source.as_bytes()[cursor.at..];
    let end = cursor.at
        + rest
            .iter()
            .position(|&byte| byte == b'<')
            .unwrap_or(rest.len());

    if !source[cursor.at..end].trim().is_empty() {
        return Err(cursor.error("text content is not supported"));
    }
    cursor.skip_to(end);
    Ok(())
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
    fn text_content_is_refused() {
        let error = events("<text>hello</text>").expect_err("text");

        assert_eq!(error.line, 1);
        assert!(error.problem.contains("text content"));
    }
}
