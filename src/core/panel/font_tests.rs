//! The font a panel is set in, and the scene it is set in.
//!
//! The font belongs to the panel and not to a shape (ADR 0014). A panel
//! that changes its font changes the letterforms and nothing else: the
//! shapes it draws are the shapes it had.

use crate::core::geom::{Point, Rgba};
use crate::core::motion::Motion;
use crate::core::panel::Panel;
use crate::core::raster;
use crate::core::scene::{Element, Scene};
use crate::core::shape::Shape;
use crate::core::typeface::Setting;

/// A face the program carries, in the setting a panel may be set in.
fn inter() -> Setting {
    Setting::named("inter", None).expect("a face the program carries")
}

/// A panel with one label in it.
fn panel_with_a_label() -> Panel {
    let mut scene = Scene::new();
    let shape = Shape::Text {
        at: Point::new(0, 0),
        text: "I".to_string(),
        scale: 1,
        color: Rgba::WHITE,
        spacing: 0,
    };
    scene.push(Element::new(shape, Motion::none()).expect("a text"));
    Panel::new(scene, vec![], vec![]).expect("a panel")
}

#[test]
fn the_panels_font_is_the_scenes_font() {
    let mut panel = Panel::new(Scene::new(), vec![], vec![]).expect("a panel");

    panel.set_font(Some(inter()));

    assert_eq!(panel.font().map(|font| font.face.name()), Some("inter"));
    assert_eq!(
        panel.scene(0).font().map(|font| font.face.name()),
        Some("inter"),
        "the scene the panel draws is drawn in that font"
    );
}

#[test]
fn a_panel_with_no_font_is_drawn_in_the_built_in_font() {
    let panel = Panel::new(Scene::new(), vec![], vec![]).expect("a panel");

    assert_eq!(panel.font(), None);
}

#[test]
fn changing_a_panels_font_changes_the_letters_and_nothing_else() {
    let mut panel = panel_with_a_label();
    let before = raster::render(&panel.scene(0), 20, 12).expect("a frame");
    let elements = panel.scene(0).elements().to_vec();

    panel.set_font(Some(inter()));

    let after = raster::render(&panel.scene(0), 20, 12).expect("a frame");
    assert_eq!(
        elements,
        panel.scene(0).elements().to_vec(),
        "the shapes are the shapes the panel had"
    );
    assert_ne!(before, after, "the letters are drawn differently");
}
