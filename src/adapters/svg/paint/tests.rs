//! The paint rules, seen from the outside.

use super::{Inherited, Paint};
use crate::adapters::svg::xml::Element;
use crate::core::geom::Rgba;

fn element(attributes: &[(&str, &str)]) -> Element {
    Element {
        name: "rect".to_string(),
        attributes: attributes
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect(),
        line: 1,
        children: Vec::new(),
    }
}

#[test]
fn a_fill_in_a_style_is_found() {
    let element = element(&[("style", "fill: rgb(1, 2, 3)")]);

    let paint = Inherited::stated(&element, "fill").expect("a fill");

    assert_eq!(paint, Some(Paint::Colour(Rgba::rgb(1, 2, 3))));
}

#[test]
fn the_attribute_wins_over_the_style() {
    let element = element(&[("fill", "#ffffff"), ("style", "fill: rgb(1, 2, 3)")]);

    let paint = Inherited::stated(&element, "fill").expect("a fill");

    assert_eq!(paint, Some(Paint::Colour(Rgba::rgb(255, 255, 255))));
}

#[test]
fn a_paint_server_is_named() {
    let element = element(&[("fill", "url(#gradient)")]);

    let paint = Inherited::stated(&element, "fill").expect("a fill");

    assert_eq!(
        paint,
        Some(Paint::Unsupported(
            "the fill is the paint server `url(#gradient)`".to_string()
        ))
    );
}

#[test]
fn a_group_paints_its_children() {
    let group = element(&[("fill", "#ffffff")]);
    let shape = element(&[]);

    let inherited = Inherited::default().inherit(&group);

    assert_eq!(inherited.fill_of(&shape), Paint::Colour(Rgba::WHITE));
}

#[test]
fn a_shape_nobody_filled_is_black() {
    let shape = element(&[]);

    assert_eq!(
        Inherited::default().fill_of(&shape),
        Paint::Colour(Rgba::BLACK)
    );
}

#[test]
fn a_stroked_outline_without_a_fill_is_an_outline() {
    let shape = element(&[("stroke", "#000000")]);

    assert_eq!(
        Inherited::default().outline_fill_of(&shape),
        Paint::Nothing
    );
}
