//! The path grammar, seen from the outside.

use super::*;

/// The segments a path turned out to be.
fn segments_of(source: &str) -> Vec<Segment> {
    match segments(source, 1).expect("a path") {
        Parsed::Segments(shape) => shape.segments,
        Parsed::Undrawn(reason) => panic!("the path was not drawn: {reason}"),
    }
}

#[test]
fn absolute_commands_become_segments() {
    let segments = segments_of("M2 2 H8 V6 H2 Z");

    assert_eq!(
        segments,
        [
            Segment::Move {
                to: Point::new(2, 2)
            },
            Segment::Line {
                to: Point::new(8, 2)
            },
            Segment::Line {
                to: Point::new(8, 6)
            },
            Segment::Line {
                to: Point::new(2, 6)
            },
            Segment::Close,
        ]
    );
}

#[test]
fn relative_commands_accumulate() {
    let segments = segments_of("m 2 2 l 6 0 v 4 h -6 z");

    assert_eq!(
        segments,
        [
            Segment::Move {
                to: Point::new(2, 2)
            },
            Segment::Line {
                to: Point::new(8, 2)
            },
            Segment::Line {
                to: Point::new(8, 6)
            },
            Segment::Line {
                to: Point::new(2, 6)
            },
            Segment::Close,
        ]
    );
}

#[test]
fn a_closed_path_does_not_repeat_its_first_point() {
    let shape = CorePath {
        segments: segments_of("M0 0 H10 V10 H0 V0 Z"),
    };

    let outline = outline_of(&shape).expect("a straight outline");

    assert_eq!(
        outline.points,
        [
            Point::new(0, 0),
            Point::new(10, 0),
            Point::new(10, 10),
            Point::new(0, 10)
        ]
    );
    assert!(outline.closed);
}

#[test]
fn two_shapes_in_one_path_are_kept() {
    let segments = segments_of("M0 0 L1 1 M2 2 L3 3");

    assert_eq!(
        segments,
        [
            Segment::Move {
                to: Point::new(0, 0)
            },
            Segment::Line {
                to: Point::new(1, 1)
            },
            Segment::Move {
                to: Point::new(2, 2)
            },
            Segment::Line {
                to: Point::new(3, 3)
            },
        ]
    );
}

#[test]
fn a_path_with_a_curve_has_no_straight_outline() {
    let shape = CorePath {
        segments: segments_of("M0 0 C 1 1 2 2 3 3"),
    };

    assert_eq!(outline_of(&shape), None);
}

#[test]
fn an_arc_is_read_as_cubics() {
    let parsed = segments("M0 0 A 5 5 0 0 1 10 10", 4).expect("an arc");

    assert!(
        matches!(parsed, Parsed::Segments(_)),
        "an arc is drawn, not refused"
    );
}

#[test]
fn a_path_is_moved_whole() {
    let shape = CorePath {
        segments: segments_of("M0 0 C 1 1 2 2 3 3"),
    };

    let moved = translated(&shape, (10, 20));

    assert_eq!(
        moved.segments[1],
        Segment::Cubic {
            control1: Point::new(11, 21),
            control2: Point::new(12, 22),
            to: Point::new(13, 23),
        }
    );
}

#[test]
fn the_flags_written_without_a_separator_are_read_as_flags() {
    let packed = segments_of("M0 0 A 10 10 0 0110 0");
    let spaced = segments_of("M0 0 A 10 10 0 0 1 10 0");
    let other = segments_of("M0 0 A 10 10 0 0010 0");

    assert_eq!(packed, spaced, "the flags are the same arc either way");
    assert_ne!(
        packed, other,
        "the second flag is the `1` of `0110`, and not the number eleven"
    );
}

#[test]
fn a_relative_arc_is_relative_to_the_current_point() {
    let relative = segments_of("M10 10 a 10 10 0 0 1 10 0");
    let absolute = segments_of("M10 10 A 10 10 0 0 1 20 10");

    assert_eq!(relative, absolute);
}

#[test]
fn an_arc_in_a_path_becomes_cubic_segments() {
    let segments = segments_of("M0 0 A 4 4 0 0 1 4 4");

    assert_eq!(segments.len(), 2, "a move and one cubic");
    assert!(matches!(segments[1], Segment::Cubic { .. }));
}
