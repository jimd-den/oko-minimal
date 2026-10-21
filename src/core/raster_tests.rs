mod tests {
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::motion::Motion;
    use crate::core::path::{Path, Segment};
    use crate::core::raster::*;
    use crate::core::scene::Element;
    use crate::core::scene::Scene;
    use crate::core::shape::Shape;

    /// Put a shape with no motion into a scene.
    fn push(scene: &mut Scene, shape: Shape) {
        let element = Element::new(shape, Motion::none()).expect("a shape with no motion");
        scene.push(element);
    }

    #[test]
    fn a_rectangle_paints_its_area() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Rect {
                area: Rect::new(1, 1, 2, 2),
                color: Rgba::WHITE,
            },
        );

        let frame = render(&scene, 4, 3).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(1, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 1)), Some(Rgba::BLACK));
        assert_eq!(frame.color_at(Point::new(3, 3)), None);
    }

    #[test]
    fn a_clear_paints_over_what_came_before() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Rect {
                area: Rect::new(0, 0, 2, 2),
                color: Rgba::WHITE,
            },
        );
        push(&mut scene, Shape::Clear(Rgba::BLACK));

        let frame = render(&scene, 2, 2).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_reaches_both_ends() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Line {
                from: Point::new(0, 0),
                to: Point::new(3, 3),
                color: Rgba::WHITE,
            },
        );

        let frame = render(&scene, 4, 4).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(1, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 3)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(3, 0)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_from_far_outside_is_clipped_and_does_not_hang() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Line {
                from: Point::new(-1_000_000, 2),
                to: Point::new(1_000_000, 2),
                color: Rgba::WHITE,
            },
        );

        let frame = render(&scene, 8, 5).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(7, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 1)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_text_operation_paints_its_glyph() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Text {
                at: Point::new(0, 0),
                text: "I".to_string(),
                scale: 1,
                color: Rgba::WHITE,
                spacing: 0,
            },
        );

        let frame = render(&scene, 5, 7).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(2, 3)), Some(Rgba::WHITE));
    }

    #[test]
    fn a_path_paints_its_interior() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Path {
                path: Path {
                    segments: vec![
                        Segment::Move {
                            to: Point::new(1, 1),
                        },
                        Segment::Line {
                            to: Point::new(9, 1),
                        },
                        Segment::Line {
                            to: Point::new(1, 9),
                        },
                        Segment::Close,
                    ],
                },
                fill: Some(Rgba::WHITE),
                stroke: None,
            },
        );

        let frame = render(&scene, 12, 12).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(9, 9)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_stroked_path_paints_its_edge_and_not_its_inside() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Path {
                path: Path {
                    segments: vec![
                        Segment::Move {
                            to: Point::new(1, 1),
                        },
                        Segment::Line {
                            to: Point::new(9, 1),
                        },
                        Segment::Line {
                            to: Point::new(1, 9),
                        },
                        Segment::Close,
                    ],
                },
                fill: None,
                stroke: Some(Rgba::WHITE),
            },
        );

        let frame = render(&scene, 12, 12).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(5, 1)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_line_entirely_outside_paints_nothing() {
        let mut scene = Scene::new();
        push(
            &mut scene,
            Shape::Line {
                from: Point::new(-10, -10),
                to: Point::new(-10, 10),
                color: Rgba::WHITE,
            },
        );

        let frame = render(&scene, 4, 4).expect("a small frame");

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }
}
