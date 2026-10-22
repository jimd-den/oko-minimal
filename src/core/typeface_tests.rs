//! The real fonts, seen from the outside.

mod tests {
    use crate::core::typeface::Face;

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

        assert_eq!(plain.path, weighted.path);
    }
}
