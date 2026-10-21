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

    assert_eq!(paint, Paint::Colour(Rgba::rgb(1, 2, 3)));
}

#[test]
fn the_attribute_wins_over_the_style() {
    let element = element(&[("fill", "#ffffff"), ("style", "fill: rgb(1, 2, 3)")]);

    let paint = Inherited::stated(&element, "fill").expect("a fill");

    assert_eq!(paint, Paint::Colour(Rgba::rgb(255, 255, 255)));
}

#[test]
fn a_paint_server_is_named() {
    let element = element(&[("fill", "url(#gradient)")]);

    let paint = Inherited::stated(&element, "fill").expect("a fill");

    assert_eq!(
        paint,
        Paint::Unsupported("the fill is the paint server `url(#gradient)`".to_string())
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

    assert_eq!(Inherited::default().outline_fill_of(&shape), Paint::Nothing);
}

#[test]
fn an_opacity_below_one_makes_the_fill_translucent() {
    let element = element(&[("fill", "#ff0000"), ("opacity", "0.5")]);

    let paint = Inherited::default().fill_of(&element);

    let reason = match &paint {
        Paint::Unsupported(reason) => reason.clone(),
        other => panic!("the paint was {other:?}"),
    };
    assert!(reason.contains("translucent"), "the reason was {reason}");
}

#[test]
fn an_opacity_of_one_is_drawn() {
    let element = element(&[("fill", "#ff0000"), ("opacity", "1")]);

    assert_eq!(
        Inherited::default().fill_of(&element),
        Paint::Colour(Rgba::rgb(255, 0, 0))
    );
}

#[test]
fn a_fill_opacity_below_one_makes_the_fill_translucent() {
    let element = element(&[("fill", "#ff0000"), ("fill-opacity", "0.5")]);

    assert!(matches!(
        Inherited::default().fill_of(&element),
        Paint::Unsupported(_)
    ));
}

#[test]
fn a_group_fades_the_shapes_inside_it() {
    let group = element(&[("opacity", "0.5")]);
    let shape = element(&[("fill", "#ff0000")]);

    let inherited = Inherited::default().inherit(&group);

    assert!(matches!(inherited.fill_of(&shape), Paint::Unsupported(_)));
}
