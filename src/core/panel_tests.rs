//! The panel's own rules: what it holds, what a key does to it, and what it
//! draws.

mod tests {
    use crate::core::binding::{Action, Binding};
    use crate::core::event::{Key, KeyCode};
    use crate::core::geom::{Rect, Rgba};
    use crate::core::motion::{Leg, Motion};
    use crate::core::panel::{Flag, Outcome, Panel, PanelError};
    use crate::core::scene::{Condition, Element, Scene};
    use crate::core::shape::Shape;

    fn rect_at(x: i32) -> Shape {
        Shape::Rect {
            area: Rect::new(x, 0, 4, 4),
            color: Rgba::WHITE,
        }
    }

    fn element(condition: Condition, motion: Motion) -> Element {
        Element::when(rect_at(0), motion, condition).expect("a conditioned element")
    }

    fn scene_of(element: Element) -> Scene {
        let mut scene = Scene::new();
        scene.push(element);
        scene
    }

    fn binding(key: char, action: Action) -> Binding {
        Binding::new(Key::plain(KeyCode::Char(key)), action).expect("a binding")
    }

    fn set(flag: &str, value: bool) -> Action {
        Action::Set {
            flag: flag.to_string(),
            value,
        }
    }

    fn panel(flags: Vec<Flag>, bindings: Vec<Binding>) -> Panel {
        Panel::new(Scene::new(), flags, bindings).expect("a panel")
    }

    #[test]
    fn a_flag_is_held_and_read() {
        let panel = panel(vec![Flag::new("shown", true)], vec![]);

        assert_eq!(panel.flag("shown"), Some(true));
    }

    #[test]
    fn a_flag_that_was_not_given_is_not_a_flag() {
        let panel = panel(vec![Flag::new("shown", true)], vec![]);

        assert_eq!(panel.flag("other"), None);
    }

    #[test]
    fn a_shape_with_a_condition_is_not_in_the_scene_while_its_flag_is_false() {
        let scene = scene_of(element(
            Condition::Flag("shown".to_string()),
            Motion::none(),
        ));
        let panel = Panel::new(scene, vec![Flag::new("shown", false)], vec![]).expect("a panel");

        assert_eq!(panel.scene(0).len(), 0);
    }

    #[test]
    fn a_shape_with_a_condition_is_in_the_scene_while_its_flag_is_true() {
        let scene = scene_of(element(
            Condition::Flag("shown".to_string()),
            Motion::none(),
        ));
        let panel = Panel::new(scene, vec![Flag::new("shown", true)], vec![]).expect("a panel");

        assert_eq!(panel.scene(0).len(), 1);
    }

    #[test]
    fn a_shape_without_a_condition_is_always_in_the_scene() {
        let scene = scene_of(element(Condition::Always, Motion::none()));
        let panel = Panel::new(scene, vec![], vec![]).expect("a panel");

        assert_eq!(panel.scene(0).len(), 1);
    }

    #[test]
    fn a_binding_sets_a_flag() {
        let mut panel = panel(
            vec![Flag::new("shown", false)],
            vec![binding('x', set("shown", true))],
        );

        panel.key(Key::plain(KeyCode::Char('x')));

        assert_eq!(panel.flag("shown"), Some(true));
    }

    #[test]
    fn a_binding_with_the_action_quit_says_so() {
        let mut panel = panel(vec![], vec![binding('q', Action::Quit)]);

        assert_eq!(panel.key(Key::plain(KeyCode::Char('q'))), Outcome::Quit);
    }

    #[test]
    fn a_key_that_sets_a_flag_says_the_panel_changed() {
        let mut panel = panel(
            vec![Flag::new("shown", false)],
            vec![binding('x', set("shown", true))],
        );

        assert_eq!(panel.key(Key::plain(KeyCode::Char('x'))), Outcome::Changed);
    }

    #[test]
    fn a_key_that_sets_a_flag_to_the_value_it_has_says_nothing_changed() {
        let mut panel = panel(
            vec![Flag::new("shown", true)],
            vec![binding('x', set("shown", true))],
        );

        assert_eq!(
            panel.key(Key::plain(KeyCode::Char('x'))),
            Outcome::Unchanged
        );
    }

    #[test]
    fn a_key_that_is_not_bound_says_nothing_changed() {
        let mut panel = panel(
            vec![Flag::new("shown", false)],
            vec![binding('x', set("shown", true))],
        );

        assert_eq!(
            panel.key(Key::plain(KeyCode::Char('z'))),
            Outcome::Unchanged
        );
    }

    #[test]
    fn the_scene_at_a_moment_has_the_conditions_applied() {
        let motion = Motion::new(500, vec![Leg::new(rect_at(10), 1)]);
        let scene = scene_of(element(Condition::Flag("shown".to_string()), motion));
        let panel = Panel::new(scene, vec![Flag::new("shown", true)], vec![]).expect("a panel");

        assert_eq!(panel.scene(0).elements()[0].shape, rect_at(0));
        assert_eq!(panel.scene(1_000).elements()[0].shape, rect_at(10));
    }

    #[test]
    fn a_binding_that_names_a_flag_that_was_not_declared_is_refused() {
        let error = Panel::new(Scene::new(), vec![], vec![binding('x', set("shown", true))])
            .expect_err("an undeclared flag");

        assert_eq!(
            error,
            PanelError::UndeclaredFlag {
                flag: "shown".to_string()
            }
        );
        assert!(error.to_string().contains("shown"));
    }

    #[test]
    fn a_condition_that_names_a_flag_that_was_not_declared_is_refused() {
        let scene = scene_of(element(
            Condition::Flag("shown".to_string()),
            Motion::none(),
        ));
        let error = Panel::new(scene, vec![], vec![]).expect_err("an undeclared flag");

        assert_eq!(
            error,
            PanelError::UndeclaredFlag {
                flag: "shown".to_string()
            }
        );
        assert!(error.to_string().contains("shown"));
    }
}
