//! The text of a panel, seen from the outside.

mod tests {
    use crate::core::frame::Frame;
    use crate::core::geom::{Point, Rgba};
    use crate::core::text::{Text, paint, width};

    #[test]
    fn an_i_is_a_column_with_two_rules() {
        let mut frame = Frame::new(5, 7).expect("a glyph-sized frame");

        paint(
            &mut frame,
            &Text {
                at: Point::new(0, 0),
                text: "I",
                scale: 1,
                spacing: 0,
                colour: Rgba::WHITE,
            },
        );

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(4, 6)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(2, 3)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 3)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_scale_multiplies_the_glyph() {
        let mut frame = Frame::new(10, 14).expect("a doubled frame");

        paint(
            &mut frame,
            &Text {
                at: Point::new(0, 0),
                text: "I",
                scale: 2,
                spacing: 0,
                colour: Rgba::WHITE,
            },
        );

        assert_eq!(frame.color_at(Point::new(9, 0)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(4, 6)), Some(Rgba::WHITE));
        assert_eq!(frame.color_at(Point::new(0, 6)), Some(Rgba::BLACK));
    }

    #[test]
    fn a_text_is_as_wide_as_its_letters() {
        let mut frame = Frame::new(20, 7).expect("a frame");

        paint(
            &mut frame,
            &Text {
                at: Point::new(0, 0),
                text: "ii",
                scale: 1,
                spacing: 0,
                colour: Rgba::WHITE,
            },
        );

        assert_eq!(
            frame.color_at(Point::new(4, 2)),
            Some(Rgba::WHITE),
            "the second letter follows the first one's own width"
        );
        assert_eq!(
            frame.color_at(Point::new(2, 4)),
            Some(Rgba::BLACK),
            "a letter is separated from the next by one column"
        );
        assert_eq!(
            frame.color_at(Point::new(3, 4)),
            Some(Rgba::WHITE),
            "the second letter stands where the first one ended"
        );
    }

    #[test]
    fn a_spacing_separates_the_letters() {
        let mut frame = Frame::new(20, 7).expect("a frame");

        paint(
            &mut frame,
            &Text {
                at: Point::new(0, 0),
                text: "ii",
                scale: 1,
                spacing: 2,
                colour: Rgba::WHITE,
            },
        );

        assert_eq!(
            frame.color_at(Point::new(6, 2)),
            Some(Rgba::WHITE),
            "the second letter is two pixels further along"
        );
    }

    #[test]
    fn a_text_has_a_width() {
        assert_eq!(width("ii", 1, 0), 5, "two letters and a gap");
        assert_eq!(width("ii", 1, 2), 7, "the spacing is between the letters");
    }

    #[test]
    fn a_space_leaves_the_frame_alone() {
        let mut frame = Frame::new(6, 7).expect("a small frame");

        paint(
            &mut frame,
            &Text {
                at: Point::new(0, 0),
                text: " ",
                scale: 1,
                spacing: 0,
                colour: Rgba::WHITE,
            },
        );

        assert_eq!(frame.color_at(Point::new(0, 0)), Some(Rgba::BLACK));
    }
}
