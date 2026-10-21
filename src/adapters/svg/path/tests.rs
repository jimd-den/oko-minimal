//! The path grammar, seen from the outside.

use super::*;

#[test]
fn absolute_commands_become_points() {
    let path = outline("M2 2 H8 V6 H2 Z", 1).expect("a rectangle");

    assert_eq!(
        path,
        Path::Edges(Outline {
            points: [
                Point::new(2, 2),
                Point::new(8, 2),
                Point::new(8, 6),
                Point::new(2, 6),
            ],
            closed: true,
        })
    );
}

#[test]
fn a_closed_path_does_not_repeat_its_first_point() {
    let path = outline("M0 0 H10 V10 H0 V0 Z", 1).expect("a rectangle");

    assert_eq!(
        path,
        Path::Edges(Outline {
            points: [
                Point::new(0, 0),
                Point::new(10, 0),
                Point::new(10, 10),
                Point::new(0, 10),
            ],
            closed: true,
        })
    );
}

#[test]
fn relative_commands_accumulate() {
    let path = outline("m 2 2 l 6 0 v 4 h -6 z", 1).expect("a rectangle");

    assert_eq!(
        path,
        Path::Edges(Outline {
            points: [
                Point::new(2, 2),
                Point::new(8, 2),
                Point::new(8, 6),
                Point::new(2, 6),
            ],
            closed: true,
        })
    );
}

#[test]
fn a_curve_is_reported_by_name() {
    let path = outline("M0 0 C 1 1 2 2 3 3", 4).expect("a curve");

    assert_eq!(
        path,
        Path::Undrawn(
            "the path uses `C`, which this console does not draw".to_string()
        )
    );
}

#[test]
fn a_second_shape_is_reported() {
    let path = outline("M0 0 L1 1 M2 2 L3 3", 1).expect("two shapes");

    assert_eq!(
        path,
        Path::Undrawn(
            "the path has more than one shape, which this console does not draw".to_string()
        )
    );
}
