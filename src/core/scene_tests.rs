//! The display list's own rules: what an element carries, and what a scene
//! keeps when a condition does not hold.

mod tests {
    use crate::core::geom::{Rect, Rgba};
    use crate::core::motion::{Leg, Motion};
    use crate::core::scene::{Condition, Element, Scene};
    use crate::core::shape::Shape;
    use crate::core::typeface::Setting;

    /// A face the program carries, in the setting a panel may be set in.
    fn inter() -> Setting {
        Setting::named("inter", None).expect("a face the program carries")
    }

    fn element(motion: Motion) -> Element {
        Element::new(
            Shape::Rect {
                area: Rect::new(0, 0, 4, 4),
                color: Rgba::WHITE,
            },
            motion,
        )
        .expect("a shape with a motion")
    }

    #[test]
    fn a_scene_without_motion_has_no_length() {
        let mut scene = Scene::new();
        scene.push(element(Motion::none()));

        assert_eq!(scene.duration(), 0);
    }

    #[test]
    fn a_scene_is_as_long_as_its_longest_motion() {
        let mut scene = Scene::new();
        scene.push(element(Motion::new(
            500,
            vec![Leg::new(
                Shape::Rect {
                    area: Rect::new(0, 0, 4, 4),
                    color: Rgba::WHITE,
                },
                2_000,
            )],
        )));
        scene.push(element(Motion::none()));

        assert_eq!(scene.duration(), 2_500);
    }

    fn rect() -> Shape {
        Shape::Rect {
            area: Rect::new(0, 0, 4, 4),
            color: Rgba::WHITE,
        }
    }

    fn conditioned(condition: Condition) -> Element {
        Element::when(rect(), Motion::none(), condition).expect("a conditioned element")
    }

    #[test]
    fn an_element_without_a_condition_is_drawn_always() {
        let element = Element::new(rect(), Motion::none()).expect("an element");

        assert_eq!(element.condition, Condition::Always);
    }

    #[test]
    fn an_element_carries_its_condition() {
        let element = conditioned(Condition::Flag("shown".to_string()));

        assert_eq!(element.condition, Condition::Flag("shown".to_string()));
    }

    #[test]
    fn a_scene_keeps_the_elements_whose_condition_holds() {
        let mut scene = Scene::new();
        scene.push(conditioned(Condition::Always));
        scene.push(conditioned(Condition::Flag("shown".to_string())));

        let kept = scene.satisfying(|condition| matches!(condition, Condition::Flag(_)));

        assert_eq!(kept.len(), 1);
        assert_eq!(
            kept.elements()[0].condition,
            Condition::Flag("shown".to_string())
        );
    }

    #[test]
    fn a_scene_drops_the_elements_whose_condition_does_not_hold() {
        let mut scene = Scene::new();
        scene.push(conditioned(Condition::Always));
        scene.push(conditioned(Condition::Flag("shown".to_string())));

        let kept = scene.satisfying(|condition| !matches!(condition, Condition::Flag(_)));

        assert_eq!(kept.len(), 1);
        assert_eq!(kept.elements()[0].condition, Condition::Always);
    }

    #[test]
    fn the_scene_at_a_moment_keeps_the_condition() {
        let mut scene = Scene::new();
        scene.push(conditioned(Condition::Flag("shown".to_string())));

        assert_eq!(
            scene.at(0).elements()[0].condition,
            Condition::Flag("shown".to_string())
        );
    }

    #[test]
    fn a_scene_carries_the_font_it_is_drawn_in() {
        let mut scene = Scene::new();

        scene.set_font(Some(inter()));

        assert_eq!(scene.font().map(|font| font.face.name()), Some("inter"));
    }

    #[test]
    fn the_scene_at_a_moment_keeps_the_font() {
        let mut scene = Scene::new();
        scene.set_font(Some(inter()));

        assert_eq!(
            scene.at(500).font().map(|font| font.face.name()),
            Some("inter")
        );
    }

    #[test]
    fn the_scene_of_the_elements_whose_condition_holds_keeps_the_font() {
        let mut scene = Scene::new();
        scene.set_font(Some(inter()));

        let kept = scene.satisfying(|condition| matches!(condition, Condition::Always));

        assert_eq!(kept.font().map(|font| font.face.name()), Some("inter"));
    }
}
