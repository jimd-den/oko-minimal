//! Reading the tree an export writes.
//!
//! This is not an XML parser. It reads elements, attributes, comments, the XML
//! declaration and runs of character data, and it refuses a declaration. That
//! is the whole input surface of the translator, and keeping the surface this
//! small is what makes "bounded" a property of the code rather than a promise.
//!
//! The tree it builds is what the translator walks. Building it first is what
//! lets an element be skipped with the contents it holds: a `<defs>` is skipped
//! as a whole, and what is inside it never becomes a shape.
//!
//! Every limit is a refusal, never a truncation: too many elements, too many
//! attributes, an oversized value, or an unterminated construct stops with a
//! line number.

use super::cursor::Cursor;
use super::entity;
use super::error::SvgError;
use super::tag::{self, Tag};
use super::xml::{Element, Node, Text, MAX_ELEMENTS};

/// The tree an export writes, with its root element.
pub fn document(source: &str) -> Result<Element, SvgError> {
    let mut reader = Reader {
        cursor: Cursor::new(source),
        elements: 0,
    };
    reader.skip_noise()?;
    let root = reader.element()?;
    reader.skip_noise()?;
    if reader.cursor.byte().is_some() {
        return Err(reader.cursor.error("the export has content after its root element"));
    }
    Ok(root)
}

/// A reader over one export.
struct Reader<'a> {
    cursor: Cursor<'a>,
    elements: usize,
}

impl Reader<'_> {
    /// Read one element and everything inside it.
    fn element(&mut self) -> Result<Element, SvgError> {
        let (name, attributes, line, empty) = match tag::read(&mut self.cursor)? {
            Tag::Open {
                name,
                attributes,
                line,
                empty,
            } => (name, attributes, line, empty),
            Tag::Close { name, line } => {
                return Err(SvgError::new(line, format!("`{name}` closes nothing")));
            }
        };
        self.elements += 1;
        if self.elements > MAX_ELEMENTS {
            return Err(self.cursor.error("the export has too many elements"));
        }
        let mut element = Element {
            name,
            attributes,
            line,
            children: Vec::new(),
        };
        if empty {
            return Ok(element);
        }
        loop {
            match self.cursor.byte() {
                None => {
                    return Err(SvgError::new(
                        line,
                        format!("`{}` is not closed", element.name),
                    ));
                }
                Some(b'<') => {
                    if self.cursor.begins("</") {
                        let closed = tag::read(&mut self.cursor)?;
                        let Tag::Close { name, line } = closed else {
                            unreachable!("a closing tag was read");
                        };
                        if name != element.name {
                            return Err(SvgError::new(
                                line,
                                format!("`{name}` closes `{}`", element.name),
                            ));
                        }
                        return Ok(element);
                    }
                    let child = self.element()?;
                    element.children.push(Node::Element(child));
                }
                Some(_) => {
                    let (content, line) = self.text()?;
                    if content.trim().is_empty() {
                        continue;
                    }
                    element.children.push(Node::Text(Text { content, line }));
                }
            }
        }
    }

    /// A run of character data, with its entities decoded.
    fn text(&mut self) -> Result<(String, usize), SvgError> {
        let source = self.cursor.source();
        let rest = &source.as_bytes()[self.cursor.at..];
        let end = self.cursor.at
            + rest
                .iter()
                .position(|&byte| byte == b'<')
                .unwrap_or(rest.len());
        let line = self.cursor.line;
        let content = source[self.cursor.at..end].to_string();
        self.cursor.skip_to(end);
        Ok((entity::decode(&content, line)?, line))
    }

    /// Skip the whitespace, the comments and the declarations around a tag.
    fn skip_noise(&mut self) -> Result<(), SvgError> {
        loop {
            self.cursor.whitespace();
            if self.cursor.begins("<!--") {
                self.cursor
                    .skip_past("-->", "an unterminated comment")?;
                continue;
            }
            if self.cursor.begins("<?") {
                self.cursor
                    .skip_past("?>", "an unterminated processing instruction")?;
                continue;
            }
            if self.cursor.begins("<!") {
                return Err(self.cursor.error("a declaration is not allowed"));
            }
            return Ok(());
        }
    }
}
