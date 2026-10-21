//! Decoding the entity references an export writes.
//!
//! An export escapes a character as `&amp;` or as a numeric reference. The
//! reader decodes the five entities XML defines and any numeric reference,
//! because an export that names a font family with a space in it writes
//! `&quot;`, and a two-line label writes `&#10;`.
//!
//! An entity the translator does not know is an error naming it: a guess
//! would put a character in the panel that the author never typed.

use super::error::SvgError;

/// The entities XML defines by name.
const NAMED: [(&str, char); 5] = [
    ("amp", '&'),
    ("lt", '<'),
    ("gt", '>'),
    ("quot", '"'),
    ("apos", '\''),
];

/// Decode the entity references in `text`.
///
/// Contract: the five named entities and any numeric reference become the
/// characters they name. An entity this translator does not know is an error
/// naming it, with the line it was on. A `&` with no `;` after it is an error
/// too, because the export is not XML the translator can read.
pub fn decode(text: &str, line: usize) -> Result<String, SvgError> {
    if !text.contains('&') {
        return Ok(text.to_string());
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 1..];
        let end = tail
            .find(';')
            .ok_or_else(|| SvgError::new(line, "an entity reference has no `;`"))?;
        out.push(character(&tail[..end], line)?);
        rest = &tail[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The character an entity names.
fn character(name: &str, line: usize) -> Result<char, SvgError> {
    if let Some((_, symbol)) = NAMED.iter().find(|(known, _)| *known == name) {
        return Ok(*symbol);
    }
    if let Some(code) = name.strip_prefix('#') {
        return numeric(code, name, line);
    }
    Err(SvgError::new(
        line,
        format!("`&{name};` is not an entity this translator knows"),
    ))
}

/// The character a numeric reference names.
fn numeric(code: &str, name: &str, line: usize) -> Result<char, SvgError> {
    let value = match code.strip_prefix('x').or_else(|| code.strip_prefix('X')) {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => code.parse::<u32>().ok(),
    };
    let value = value.ok_or_else(|| SvgError::new(line, format!("`&{name};` is not a number")))?;
    char::from_u32(value)
        .ok_or_else(|| SvgError::new(line, format!("`&{name};` is not a character")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_five_named_entities_are_decoded() {
        assert_eq!(
            decode("a&amp;b&lt;c&gt;d&quot;e&apos;f", 1),
            Ok("a&b<c>d\"e'f".to_string())
        );
    }

    #[test]
    fn a_decimal_reference_is_decoded() {
        assert_eq!(decode("line&#10;break", 1), Ok("line\nbreak".to_string()));
    }

    #[test]
    fn a_hexadecimal_reference_is_decoded() {
        assert_eq!(decode("it&#x27;s", 1), Ok("it's".to_string()));
    }

    #[test]
    fn an_unknown_entity_is_an_error_naming_it() {
        let error = decode("a&nbsp;b", 7).expect_err("a named entity");

        assert_eq!(error.line, 7);
        assert!(error.problem.contains("&nbsp;"));
    }

    #[test]
    fn an_ampersand_without_a_semicolon_is_an_error() {
        let error = decode("a & b", 1).expect_err("a bare ampersand");

        assert!(error.problem.contains(';'));
    }

    /// A soft line break is written `&#10;` by a Figma export. It decodes to
    /// a newline, and the label rules collapse whitespace, so the label the
    /// console draws is still one line.
    #[test]
    fn a_soft_line_break_stays_one_line() {
        let svg = "<svg><text x=\"1\" y=\"9\">Discounted&#10;</text></svg>";

        assert_eq!(
            crate::adapters::svg::translate(svg).expect("a broken label"),
            "text 1 2 \"Discounted\" 1 #000000\n"
        );
    }

    #[test]
    fn text_without_an_entity_is_unchanged() {
        assert_eq!(decode("plain text", 1), Ok("plain text".to_string()));
    }
}
