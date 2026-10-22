//! Drawing a real font's glyph, seen from the outside.

mod tests {
    use crate::core::frame::Frame;
    use crate::core::geom::{Point, Rgba};
    use crate::core::typeface::Face;
    use crate::core::typeface::draw::draw;

    #[test]
    fn a_glyph_is_drawn_on_the_frame() {
        let face = Face::by_name("dejavu-mono").expect("a face");
        let glyph = face.glyph('A', 16.0, None).expect("an A");
        let mut frame = Frame::new(40, 40).expect("a frame");

        draw(&mut frame, &glyph, Point::new(2, 30), Rgba::WHITE);

        let painted =
            (0..40).any(|y| (0..40).any(|x| frame.color_at(Point::new(x, y)) == Some(Rgba::WHITE)));
        assert!(painted, "the A is on the frame");
    }

    #[test]
    fn a_glyph_stands_on_its_baseline() {
        let face = Face::by_name("dejavu-mono").expect("a face");
        let glyph = face.glyph('A', 16.0, None).expect("an A");
        let mut frame = Frame::new(40, 40).expect("a frame");

        draw(&mut frame, &glyph, Point::new(2, 30), Rgba::WHITE);

        let below = (30..40)
            .any(|y| (0..40).any(|x| frame.color_at(Point::new(x, y)) == Some(Rgba::WHITE)));
        assert!(!below, "the A stands on the line, not below it");
    }
}
