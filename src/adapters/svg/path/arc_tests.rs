//! The shape an arc makes, seen from the outside.

use super::arc::{self, Piece};
use super::centre::Arc;

/// An arc of a circle of radius one, with the given flags and end.
fn unit(large: bool, sweep: bool, to: (f64, f64)) -> Arc {
    Arc {
        rx: 1.0,
        ry: 1.0,
        rotation: 0.0,
        large,
        sweep,
        to,
    }
}

/// The point at a fraction of a cubic.
fn point_at(from: (f64, f64), piece: &Piece, t: f64) -> (f64, f64) {
    let Piece::Cubic {
        control1,
        control2,
        to,
    } = piece
    else {
        panic!("the piece is a cubic");
    };
    let u = 1.0 - t;
    (
        u * u * u * from.0
            + 3.0 * u * u * t * control1.0
            + 3.0 * u * t * t * control2.0
            + t * t * t * to.0,
        u * u * u * from.1
            + 3.0 * u * u * t * control1.1
            + 3.0 * u * t * t * control2.1
            + t * t * t * to.1,
    )
}

#[test]
fn a_quarter_arc_becomes_one_cubic() {
    let pieces = arc::pieces((1.0, 0.0), &unit(false, true, (0.0, 1.0)));

    assert_eq!(pieces.len(), 1, "a quarter of a circle is one turn");
    assert!(matches!(pieces[0], Piece::Cubic { .. }));
}

#[test]
fn a_half_circle_becomes_two_cubics() {
    let pieces = arc::pieces((0.0, 0.0), &unit(false, true, (2.0, 0.0)));

    assert_eq!(pieces.len(), 2, "half a circle is two turns");
}

#[test]
fn the_large_arc_flag_picks_the_longer_way_round() {
    let short = arc::pieces((1.0, 0.0), &unit(false, true, (0.0, 1.0)));
    let long = arc::pieces((1.0, 0.0), &unit(true, true, (0.0, 1.0)));

    assert_eq!(short.len(), 1, "the short way round is a quarter turn");
    assert_eq!(long.len(), 3, "the long way round is three quarters");
}

#[test]
fn the_sweep_flag_picks_the_direction() {
    let with = arc::pieces((1.0, 0.0), &unit(false, true, (0.0, 1.0)));
    let against = arc::pieces((1.0, 0.0), &unit(false, false, (0.0, 1.0)));

    let one = point_at((1.0, 0.0), &with[0], 0.5);
    let two = point_at((1.0, 0.0), &against[0], 0.5);

    assert!(
        (one.0 - 0.707).abs() < 0.01 && (one.1 - 0.707).abs() < 0.01,
        "the arc with the sweep goes down and right: {one:?}"
    );
    assert!(
        (two.0 - 0.293).abs() < 0.01 && (two.1 - 0.293).abs() < 0.01,
        "the arc against the sweep goes down and left: {two:?}"
    );
}

#[test]
fn a_zero_radius_becomes_a_straight_line() {
    let arc = Arc {
        rx: 0.0,
        ry: 5.0,
        rotation: 0.0,
        large: false,
        sweep: true,
        to: (10.0, 0.0),
    };

    let pieces = arc::pieces((0.0, 0.0), &arc);

    assert_eq!(pieces.len(), 1);
    let Piece::Line { to } = &pieces[0] else {
        panic!("a radius of zero is a straight line");
    };
    assert_eq!(*to, (10.0, 0.0));
}

#[test]
fn an_arc_to_the_same_point_draws_nothing() {
    let pieces = arc::pieces((1.0, 1.0), &unit(false, true, (1.0, 1.0)));

    assert!(pieces.is_empty(), "the same point twice is no arc");
}

#[test]
fn a_point_on_the_arc_is_within_a_thousandth_of_the_radius() {
    let radius = 100.0;
    let arc = Arc {
        rx: radius,
        ry: radius,
        rotation: 0.0,
        large: false,
        sweep: true,
        to: (0.0, radius),
    };
    let from = (radius, 0.0);

    let pieces = arc::pieces(from, &arc);

    for step in 0..=10 {
        let t = f64::from(step) / 10.0;
        let point = point_at(from, &pieces[0], t);
        let distance = point.0.hypot(point.1);
        assert!(
            (distance - radius).abs() < radius / 1000.0,
            "the point {point:?} is {distance} from the centre, not {radius}"
        );
    }
}

#[test]
fn a_rotation_turns_the_ellipse() {
    let circle = Arc {
        rx: 5.0,
        ry: 5.0,
        rotation: 0.0,
        large: false,
        sweep: true,
        to: (0.0, 5.0),
    };
    let turned_circle = Arc {
        rotation: 90.0,
        ..circle
    };
    let ellipse = Arc {
        rx: 5.0,
        ry: 2.0,
        rotation: 0.0,
        large: false,
        sweep: true,
        to: (0.0, 2.0),
    };
    let turned_ellipse = Arc {
        rotation: 90.0,
        ..ellipse
    };

    let one = point_at((5.0, 0.0), &arc::pieces((5.0, 0.0), &circle)[0], 0.5);
    let two = point_at((5.0, 0.0), &arc::pieces((5.0, 0.0), &turned_circle)[0], 0.5);
    let three = point_at((5.0, 0.0), &arc::pieces((5.0, 0.0), &ellipse)[0], 0.5);
    let four = point_at(
        (5.0, 0.0),
        &arc::pieces((5.0, 0.0), &turned_ellipse)[0],
        0.5,
    );

    assert!(
        (one.0 - two.0).abs() < 1e-9 && (one.1 - two.1).abs() < 1e-9,
        "a circle is a circle, whatever the rotation: {one:?} and {two:?}"
    );
    assert!(
        (three.0 - four.0).abs() > 0.5,
        "the rotation turns the ellipse: {three:?} and {four:?}"
    );
}
