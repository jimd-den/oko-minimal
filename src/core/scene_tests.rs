//! The display list's own rules: what an element carries, and what a scene
//! keeps when a condition does not hold.

mod tests {
    use crate::core::geom::{Rect, Rgba};
    use crate::core::motion::{Leg, Motion};
    use crate::core::scene::{Element, Scene};
    use crate::core::shape::Shape;

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
}
