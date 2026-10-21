mod tests {
    use super::super::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::motion::Motion;
    use crate::core::raster;
    use crate::core::shape::Shape;

    fn motion_of(text: &str) -> Motion {
        let scene = parse(text).expect("a document");
        scene.elements()[0].motion.clone()
    }

    fn error_of(text: &str) -> String {
        let errors = parse(text).expect_err("a bad document");
        errors[0].problem.clone()
    }

    #[test]
    fn a_shape_states_a_delay_and_a_leg() {
        let motion = motion_of("rect 4 4 4 4 #ff9d00 after 500ms to 60 4 4 4 #ff0000 over 2s");

        assert_eq!(motion.delay(), 500);
        assert_eq!(motion.legs().len(), 1);
        assert_eq!(motion.legs()[0].over(), 2000);
        assert_eq!(
            motion.legs()[0].target(),
            &Shape::Rect {
                area: Rect::new(60, 4, 4, 4),
                color: Rgba::rgb(255, 0, 0),
            }
        );
    }

    #[test]
    fn a_shape_states_a_leg_without_a_delay() {
        let motion = motion_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2s");

        assert_eq!(motion.delay(), 0);
        assert_eq!(motion.legs().len(), 1);
    }

    #[test]
    fn a_duration_in_ms_and_in_s_are_the_same_duration() {
        let millis = motion_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2000ms");
        let seconds = motion_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2s");

        assert_eq!(millis, seconds);
        assert_eq!(millis.legs()[0].over(), 2000);
    }

    #[test]
    fn two_legs_chain() {
        let motion = motion_of(
            "rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2s to 60 20 4 4 #ff0000 over 1s",
        );

        assert_eq!(motion.legs().len(), 2);
        assert_eq!(motion.legs()[1].over(), 1000);
    }

    #[test]
    fn a_to_with_no_over_is_refused() {
        let error = error_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000");

        assert!(error.contains("over"), "{error}");
    }

    #[test]
    fn an_over_with_no_to_is_refused() {
        let error = error_of("rect 4 4 4 4 #ff9d00 over 2s");

        assert!(error.contains("to"), "{error}");
    }

    #[test]
    fn an_unknown_unit_is_refused() {
        let error = error_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2weeks");

        assert!(error.contains("weeks"), "{error}");
    }

    #[test]
    fn a_duration_with_no_number_is_refused() {
        let error = error_of("rect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over ms");

        assert!(error.contains("number"), "{error}");
    }

    #[test]
    fn a_target_of_another_kind_is_refused() {
        let error = error_of("rect 4 4 4 4 #ff9d00 to 0 0 \"HI\" 1 #ff0000 over 2s");

        assert!(error.contains("rectangle"), "{error}");
        assert!(error.contains("text"), "{error}");
    }

    #[test]
    fn a_clear_that_states_a_motion_is_refused() {
        let error = error_of("clear #05070a to #ff0000 over 2s");

        assert!(error.contains("clear"), "{error}");
    }

    #[test]
    fn a_text_that_changes_its_label_is_refused() {
        let error = error_of("text 0 0 \"A\" 1 #ff9d00 to 10 0 \"B\" 1 #ff0000 over 2s");

        assert!(error.contains("text"), "{error}");
    }

    #[test]
    fn a_line_states_a_line_target() {
        let motion = motion_of("line 0 0 9 4 #ff9d00 to 10 0 19 4 #ff0000 over 2s");

        assert_eq!(
            motion.legs()[0].target(),
            &Shape::Line {
                from: Point::new(10, 0),
                to: Point::new(19, 4),
                color: Rgba::rgb(255, 0, 0),
            }
        );
    }

    #[test]
    fn a_path_states_a_path_target() {
        let motion =
            motion_of("path M 0 0 L 9 4 fill #ff9d00 to M 2 2 L 11 6 fill #ff0000 over 2s");

        assert_eq!(motion.legs().len(), 1);
        assert!(matches!(motion.legs()[0].target(), Shape::Path { .. }));
    }

    #[test]
    fn a_document_with_a_motion_reaches_the_frame() {
        let scene = parse("clear #000000\nrect 4 4 4 4 #ff9d00 to 60 4 4 4 #ff0000 over 2s")
            .expect("a document");

        let midway = raster::render(&scene.at(1000), 80, 12).expect("a frame");
        let rest = raster::render(&scene.at(2000), 80, 12).expect("a frame");

        assert_eq!(
            midway.color_at(Point::new(32, 4)),
            Some(Rgba::rgb(255, 79, 0))
        );
        assert_eq!(rest.color_at(Point::new(60, 4)), Some(Rgba::rgb(255, 0, 0)));
    }
}
