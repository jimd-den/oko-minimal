//! The colours an export writes, seen from the outside.

use super::*;

#[test]
fn a_hex_colour_is_read() {
    assert_eq!(
        parse("#ff9d00"),
        Some(Colour::Opaque(Rgba::rgb(255, 157, 0)))
    );
}

#[test]
fn a_short_hex_colour_is_expanded() {
    assert_eq!(parse("#0f0"), Some(Colour::Opaque(Rgba::rgb(0, 255, 0))));
}

#[test]
fn an_rgb_colour_is_read() {
    assert_eq!(
        parse("rgb(12, 34, 56)"),
        Some(Colour::Opaque(Rgba::rgb(12, 34, 56)))
    );
}

#[test]
fn an_opaque_rgba_colour_is_read() {
    assert_eq!(
        parse("rgba(12, 34, 56, 1)"),
        Some(Colour::Opaque(Rgba::rgb(12, 34, 56)))
    );
}

#[test]
fn a_translucent_colour_is_reported() {
    assert_eq!(parse("rgba(0, 0, 0, 0.5)"), Some(Colour::Translucent));
}

#[test]
fn a_named_colour_is_read() {
    assert_eq!(parse("black"), Some(Colour::Opaque(Rgba::rgb(0, 0, 0))));
}

#[test]
fn a_word_that_is_not_a_colour_names_none() {
    assert_eq!(parse("chartreuse"), None);
}

/// `transparent` is a colour name, and it carries no alpha left in it: the
/// document holds opaque colours, so it cannot be drawn.
#[test]
fn a_transparent_colour_is_reported() {
    assert_eq!(parse("transparent"), Some(Colour::Translucent));
}
