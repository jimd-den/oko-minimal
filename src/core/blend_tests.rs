mod tests {
    use crate::core::blend::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::path::{Path, Segment};

    #[test]
    fn a_point_is_halfway_between_two_points() {
        let halfway = point(Point::new(0, 0), Point::new(10, 20), 0.5);

        assert_eq!(halfway, Point::new(5, 10));
    }

    #[test]
    fn an_area_is_halfway_between_two_areas() {
        let halfway = area(Rect::new(0, 0, 4, 4), Rect::new(10, 20, 8, 12), 0.5);

        assert_eq!(halfway, Rect::new(5, 10, 6, 8));
    }

    #[test]
    fn a_colour_is_halfway_between_two_colours() {
        let halfway = colour(Rgba::rgb(255, 157, 0), Rgba::rgb(255, 0, 0), 0.5);

        assert_eq!(halfway, Rgba::rgb(255, 79, 0));
    }

    #[test]
    fn a_channel_is_rounded_to_the_nearest_whole_number() {
        // 157 and 0 are 78.5 apart, and the rule is the nearest whole number.
        assert_eq!(channel(157, 0, 0.5), 79);
        assert_eq!(channel(0, 157, 0.5), 79);
    }

    #[test]
    fn a_size_is_halfway_between_two_sizes() {
        let halfway = size(4, 12, 0.5);

        assert_eq!(halfway, 8);
    }

    #[test]
    fn a_paint_that_is_absent_stays_absent() {
        assert_eq!(paint(None, Some(Rgba::WHITE), 0.5), None);
        assert_eq!(paint(Some(Rgba::WHITE), None, 0.5), None);
    }

    #[test]
    fn a_path_moves_to_another_path_of_the_same_shape() {
        let one = Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(0, 0),
                },
                Segment::Line {
                    to: Point::new(10, 0),
                },
            ],
        };
        let other = Path {
            segments: vec![
                Segment::Move {
                    to: Point::new(0, 10),
                },
                Segment::Line {
                    to: Point::new(10, 10),
                },
            ],
        };

        let halfway = path(&one, &other, 0.5);

        assert_eq!(
            halfway.segments[0],
            Segment::Move {
                to: Point::new(0, 5)
            }
        );
        assert_eq!(
            halfway.segments[1],
            Segment::Line {
                to: Point::new(10, 5)
            }
        );
    }
}
