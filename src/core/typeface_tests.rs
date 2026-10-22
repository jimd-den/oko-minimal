//! The real fonts, seen from the outside.

mod tests {
    use crate::core::typeface::{Face, Setting};

    #[test]
    fn the_program_carries_three_faces() {
        let names = Face::names();

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"dejavu-mono"), "{names:?}");
        assert!(names.contains(&"source-sans-3"), "{names:?}");
        assert!(names.contains(&"inter"), "{names:?}");
    }

    #[test]
    fn a_face_is_found_by_its_name() {
        let face = Face::by_name("dejavu-mono").expect("a face the program carries");

        assert_eq!(face.name(), "dejavu-mono");
        assert!(Face::by_name("comic-sans").is_none());
    }

    #[test]
    fn a_glyph_is_a_path_with_segments() {
        let face = Face::by_name("dejavu-mono").expect("a face");

        let glyph = face.glyph('A', 16.0, None).expect("an A");

        assert!(!glyph.path.segments.is_empty(), "the A has an outline");
        assert!(glyph.advance > 0.0, "the A takes room on the line");
    }

    #[test]
    fn a_character_the_face_does_not_have_is_nothing() {
        let face = Face::by_name("dejavu-mono").expect("a face");

        assert!(face.glyph('\u{4e00}', 16.0, None).is_none());
    }

    #[test]
    fn a_variable_face_is_different_at_two_weights() {
        let face = Face::by_name("inter").expect("a face");

        let light = face.glyph('A', 16.0, Some(400)).expect("an A");
        let bold = face.glyph('A', 16.0, Some(700)).expect("an A");

        assert_ne!(light.path, bold.path, "the weights are different shapes");
    }

    #[test]
    fn a_face_without_weights_takes_the_weight_it_was_drawn_with() {
        let face = Face::by_name("dejavu-mono").expect("a face");

        let plain = face.glyph('A', 16.0, None).expect("an A");
        let weighted = face.glyph('A', 16.0, Some(700)).expect("an A");

        assert_eq!(
            plain, weighted,
            "the face gives one glyph, whatever the weight"
        );
    }

    #[test]
    fn a_capital_height_is_a_fraction_of_the_em() {
        for name in Face::names() {
            let face = Face::by_name(name).expect("a face the program carries");

            let ratio = face.cap_ratio(None);

            assert!(
                (0.5..=0.8).contains(&ratio),
                "{name} has capitals {ratio} of its em"
            );
        }
    }

    #[test]
    fn a_setting_names_a_face_the_program_carries() {
        let setting = Setting::named("dejavu-mono", None).expect("a face the program carries");

        assert_eq!(setting.face.name(), "dejavu-mono");
    }

    #[test]
    fn a_setting_refuses_a_face_the_program_does_not_carry() {
        let refused = Setting::named("comic-sans", None);

        assert!(refused.is_err(), "a face that is not here is refused");
    }

    #[test]
    fn two_settings_of_one_face_at_one_weight_are_equal() {
        let plain = Setting::named("dejavu-mono", None).expect("a setting");
        let same = Setting::named("dejavu-mono", None).expect("a setting");

        assert_eq!(plain, same);
    }

    #[test]
    fn two_settings_of_one_face_at_two_weights_are_not_equal() {
        let light = Setting::named("inter", Some(400)).expect("a setting");
        let bold = Setting::named("inter", Some(700)).expect("a setting");

        assert_ne!(light, bold);
    }
}
