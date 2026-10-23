//! The shapes of a display list, seen from the outside.

mod tests {
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::path::{Path, Segment};
    use crate::core::shape::Shape;

    fn rect() -> Shape {
        Shape::Rect {
            area: Rect::new(0, 0, 4, 4),
            color: Rgba::WHITE,
        }
    }

    fn line() -> Shape {
        Shape::Line {
            from: Point::new(0, 0),
            to: Point::new(8, 8),
            color: Rgba::WHITE,
        }
    }

    fn text(label: &str) -> Shape {
        Shape::Text {
            spacing: 0,
            at: Point::new(0, 0),
            text: label.to_string(),
            scale: 1,
            color: Rgba::WHITE,
        }
    }

    fn path(segments: Vec<Segment>) -> Shape {
        Shape::Path {
            path: Path { segments },
            fill: Some(Rgba::WHITE),
            stroke: None,
        }
    }

    #[test]
    fn a_rectangle_may_become_a_rectangle() {
        assert!(rect().can_become(&rect()).is_ok());
    }

    #[test]
    fn a_rectangle_may_not_become_a_line() {
        let refusal = rect().can_become(&line()).expect_err("another kind");

        assert!(refusal.contains("rectangle"), "{refusal}");
        assert!(refusal.contains("line"), "{refusal}");
    }

    #[test]
    fn a_text_may_not_become_a_different_text() {
        let refusal = text("OK")
            .can_become(&text("NO"))
            .expect_err("another label");

        assert!(refusal.contains("text"), "{refusal}");
    }

    #[test]
    fn a_text_may_become_the_same_text() {
        assert!(text("OK").can_become(&text("OK")).is_ok());
    }

    #[test]
    fn a_path_may_not_become_a_path_of_another_shape() {
        let one = path(vec![Segment::Move {
            to: Point::new(0, 0),
        }]);
        let other = path(vec![
            Segment::Move {
                to: Point::new(0, 0),
            },
            Segment::Line {
                to: Point::new(4, 4),
            },
        ]);

        let refusal = one.can_become(&other).expect_err("another shape");

        assert!(refusal.contains("path"), "{refusal}");
    }

    #[test]
    fn a_path_may_become_a_path_of_the_same_shape() {
        let one = path(vec![Segment::Move {
            to: Point::new(0, 0),
        }]);
        let other = path(vec![Segment::Move {
            to: Point::new(9, 9),
        }]);

        assert!(one.can_become(&other).is_ok());
    }

    #[test]
    fn a_clear_may_not_move() {
        let refusal = Shape::Clear(Rgba::WHITE)
            .can_become(&rect())
            .expect_err("a clear has nothing to move to");

        assert!(refusal.contains("clear"), "{refusal}");
    }
}
