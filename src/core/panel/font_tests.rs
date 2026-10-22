//! The font a panel is set in, and the scene it is set in.
//!
//! The font belongs to the panel and not to a shape (ADR 0014). A panel
//! that changes its font changes the letterforms and nothing else: the
//! shapes it draws are the shapes it had.

use crate::core::panel::Panel;
use crate::core::scene::Scene;
use crate::core::typeface::Setting;

/// A face the program carries, in the setting a panel may be set in.
fn inter() -> Setting {
    Setting::named("inter", None).expect("a face the program carries")
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
