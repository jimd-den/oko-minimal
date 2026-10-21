mod tests {
    use super::super::path::*;
    use crate::core::geom::{Point, Rgba};
    use crate::core::path::{Path, Segment};
    use crate::core::scene::Element;
    use crate::core::shape::Shape;

    #[test]
    fn a_path_directive_becomes_a_path_operation() {
        let op = parse("path M 10 10 C 20 0 30 0 30 10 Z fill #ff9d00").expect("a path");

        assert_eq!(
            op.shape,
            Shape::Path {
                path: Path {
                    segments: vec![
                        Segment::Move {
                            to: Point::new(10, 10),
                        },
                        Segment::Cubic {
                            control1: Point::new(20, 0),
                            control2: Point::new(30, 0),
                            to: Point::new(30, 10),
                        },
                        Segment::Close,
                    ],
                },
                fill: Some(Rgba::rgb(255, 157, 0)),
                stroke: None,
            }
        );
    }

    #[test]
    fn a_path_may_state_a_stroke_and_no_fill() {
        let op = parse("path M 0 0 L 10 0 stroke #1f6feb").expect("a path");

        let Element {
            shape: Shape::Path { fill, stroke, .. },
            ..
        } = op
        else {
            panic!("a path is a path operation");
        };
        assert_eq!(fill, None);
        assert_eq!(stroke, Some(Rgba::rgb(31, 111, 235)));
    }

    #[test]
    fn a_path_that_states_a_paint_twice_is_refused() {
        let error = parse("path M 0 0 L 10 0 fill #ff0000 fill #00ff00").expect_err("two fills");

        assert!(error.contains("fill"));
    }

    #[test]
    fn a_quadratic_command_makes_a_quadratic_segment() {
        let op = parse("path M 0 0 Q 5 10 10 0 fill #ff9d00").expect("a path");

        let Element {
            shape: Shape::Path { path, .. },
            ..
        } = op
        else {
            panic!("a path is a path operation");
        };
        assert_eq!(
            path.segments[1],
            Segment::Quadratic {
                control: Point::new(5, 10),
                to: Point::new(10, 0),
            }
        );
    }

    #[test]
    fn an_unknown_path_command_is_refused() {
        let error = parse("path M 0 0 A 5 5 0 0 1 10 10 fill #ff0000").expect_err("an arc");

        assert!(error.contains('A'));
    }

    #[test]
    fn a_path_without_a_paint_is_refused() {
        let error = parse("path M 0 0 L 10 10").expect_err("a path with no paint");

        assert!(error.contains("fill"));
    }

    #[test]
    fn a_path_without_segments_is_refused() {
        let error = parse("path fill #ff0000").expect_err("a path with no shape");

        assert!(error.contains("segment"));
    }

    #[test]
    fn a_path_that_does_not_start_with_a_move_is_refused() {
        let error = parse("path L 10 10 fill #ff0000").expect_err("a path with no start");

        assert!(error.contains('M'));
    }

    #[test]
    fn a_close_without_a_shape_is_refused() {
        let error = parse("path Z fill #ff0000").expect_err("a path that closes nothing");

        assert!(error.contains('Z'));
    }
}
