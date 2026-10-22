//! The outline pen, seen from the inside.

mod tests {
    use skrifa::outline::OutlinePen;

    use super::super::pen::PathPen;
    use crate::core::geom::Point;
    use crate::core::path::Segment;
    use crate::core::typeface::Face;

    #[test]
    fn the_pen_writes_the_outline_into_the_core_path() {
        let mut pen = PathPen::default();

        pen.move_to(1.0, 0.0);
        pen.line_to(2.0, 0.5);

        assert_eq!(
            pen.into_path().segments,
            vec![
                Segment::Move {
                    to: Point::new(4, 0)
                },
                Segment::Line {
                    to: Point::new(8, -2)
                },
            ],
            "the pen writes the core's own segments, on the fine grid, upside down"
        );
    }

    #[test]
    fn a_face_glyph_is_written_into_the_core_path() {
        let face = Face::by_name("dejavu-mono").expect("a face");

        let glyph = face.glyph('A', 16.0, None).expect("an A");

        assert!(
            glyph
                .path
                .segments
                .iter()
                .any(|segment| matches!(segment, Segment::Move { .. })),
            "the outline starts a shape"
        );
        assert!(
            glyph
                .path
                .segments
                .iter()
                .any(|segment| matches!(segment, Segment::Close)),
            "the outline closes its shape"
        );
    }
}
