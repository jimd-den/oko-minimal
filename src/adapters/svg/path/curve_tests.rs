//! The curve commands, seen from the outside.

use super::*;

/// The segments a path turned out to be.
fn segments_of(source: &str) -> Vec<Segment> {
    match segments(source, 1).expect("a path") {
        Parsed::Segments(shape) => shape.segments,
        Parsed::Undrawn(reason) => panic!("the path was not drawn: {reason}"),
    }
}

#[test]
fn an_absolute_cubic_is_read_into_a_cubic_segment() {
    let segments = segments_of("M0 0 C 10 20 30 40 50 60");

    assert_eq!(
        segments[1],
        Segment::Cubic {
            control1: Point::new(10, 20),
            control2: Point::new(30, 40),
            to: Point::new(50, 60),
        }
    );
}

#[test]
fn a_relative_cubic_accumulates_from_the_current_point() {
    let segments = segments_of("M10 10 c 5 0 5 10 0 10");

    assert_eq!(
        segments[1],
        Segment::Cubic {
            control1: Point::new(15, 10),
            control2: Point::new(15, 20),
            to: Point::new(10, 20),
        }
    );
}

#[test]
fn a_smooth_cubic_reflects_the_previous_control_point() {
    let segments = segments_of("M0 0 C 10 0 20 0 30 0 S 35 5 40 0");

    assert_eq!(
        segments[2],
        Segment::Cubic {
            control1: Point::new(40, 0),
            control2: Point::new(35, 5),
            to: Point::new(40, 0),
        }
    );
}

#[test]
fn a_smooth_cubic_after_a_line_uses_the_current_point() {
    let segments = segments_of("M0 0 L 10 10 S 20 20 30 30");

    assert_eq!(
        segments[2],
        Segment::Cubic {
            control1: Point::new(10, 10),
            control2: Point::new(20, 20),
            to: Point::new(30, 30),
        }
    );
}

#[test]
fn a_smooth_quadratic_reflects_the_previous_control_point() {
    let segments = segments_of("M0 0 Q 10 0 20 0 T 40 0");

    assert_eq!(
        segments[2],
        Segment::Quadratic {
            control: Point::new(30, 0),
            to: Point::new(40, 0),
        }
    );
}

#[test]
fn a_smooth_quadratic_after_a_line_uses_the_current_point() {
    let segments = segments_of("M0 0 L 10 10 T 30 30");

    assert_eq!(
        segments[2],
        Segment::Quadratic {
            control: Point::new(10, 10),
            to: Point::new(30, 30),
        }
    );
}

#[test]
fn a_quadratic_is_read_into_a_quadratic_segment() {
    let segments = segments_of("M0 0 Q 5 10 10 0");

    assert_eq!(
        segments[1],
        Segment::Quadratic {
            control: Point::new(5, 10),
            to: Point::new(10, 0),
        }
    );
}

#[test]
fn horizontal_and_vertical_commands_become_lines() {
    let segments = segments_of("M5 5 H15 V25 h-5 v-5");

    assert_eq!(
        segments[1],
        Segment::Line {
            to: Point::new(15, 5)
        }
    );
    assert_eq!(
        segments[2],
        Segment::Line {
            to: Point::new(15, 25)
        }
    );
    assert_eq!(
        segments[3],
        Segment::Line {
            to: Point::new(10, 25)
        }
    );
    assert_eq!(
        segments[4],
        Segment::Line {
            to: Point::new(10, 20)
        }
    );
}

#[test]
fn a_control_point_off_the_pixel_grid_is_rounded() {
    let segments = segments_of("M0 0 C 1.4 1.6 2 2 3 3");

    assert_eq!(
        segments[1],
        Segment::Cubic {
            control1: Point::new(1, 2),
            control2: Point::new(2, 2),
            to: Point::new(3, 3),
        }
    );
}
