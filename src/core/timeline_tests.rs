//! The timeline, seen from the outside: how a motion plays out over time.

use super::timeline::*;
use crate::core::geom::{Point, Rect, Rgba};
use crate::core::motion::{Leg, Motion};
use crate::core::shape::Shape;

fn rect_at(x: i32) -> Shape {
    Shape::Rect {
        area: Rect::new(x, 0, 4, 4),
        color: Rgba::WHITE,
    }
}

fn text(label: &str) -> Shape {
    Shape::Text {
        at: Point::new(0, 0),
        text: label.to_string(),
        scale: 1,
        color: Rgba::WHITE,
        spacing: 0,
    }
}

fn motion(delay: u64, legs: Vec<Leg>) -> Motion {
    Motion::new(delay, legs)
}

fn leg(target: Shape, over: u64) -> Leg {
    Leg::new(target, over)
}

fn x_of(shape: &Shape) -> i32 {
    match shape {
        Shape::Rect { area, .. } => area.x,
        other => panic!("not a rectangle: {other:?}"),
    }
}

#[test]
fn a_motion_without_legs_holds_its_shape() {
    let motion = Motion::none();

    assert_eq!(at(&motion, &rect_at(0), 5_000), rect_at(0));
    assert_eq!(duration(&motion), 0);
}

#[test]
fn a_motion_is_over_when_its_delay_and_its_legs_are() {
    let motion = motion(500, vec![leg(rect_at(10), 2_000)]);

    assert_eq!(duration(&motion), 2_500);
}

#[test]
fn a_shape_waits_for_its_delay() {
    let motion = motion(500, vec![leg(rect_at(10), 1_000)]);

    assert_eq!(at(&motion, &rect_at(0), 499), rect_at(0));
}

#[test]
fn a_shape_is_halfway_at_the_halfway_moment() {
    let motion = motion(0, vec![leg(rect_at(10), 1_000)]);

    assert_eq!(x_of(&at(&motion, &rect_at(0), 500)), 5);
}

#[test]
fn a_shape_holds_its_end_state_after_its_last_leg() {
    let motion = motion(0, vec![leg(rect_at(10), 1_000)]);

    assert_eq!(at(&motion, &rect_at(0), 5_000), rect_at(10));
}

#[test]
fn a_sequence_chains_so_the_second_leg_starts_where_the_first_ended() {
    let motion = motion(0, vec![leg(rect_at(10), 1_000), leg(rect_at(20), 1_000)]);

    assert_eq!(x_of(&at(&motion, &rect_at(0), 1_000)), 10);
    assert_eq!(x_of(&at(&motion, &rect_at(0), 1_500)), 15);
    assert_eq!(duration(&motion), 2_000);
}

#[test]
fn a_text_may_not_become_a_different_text() {
    let motion = motion(0, vec![leg(text("NO"), 1_000)]);

    let refusal = check(&motion, &text("OK")).expect_err("another label");

    assert!(refusal.contains("text"), "{refusal}");
}

#[test]
fn a_clear_may_not_move() {
    let motion = motion(0, vec![leg(rect_at(10), 1_000)]);

    let refusal = check(&motion, &Shape::Clear(Rgba::WHITE)).expect_err("a clear");

    assert!(refusal.contains("clear"), "{refusal}");
}

#[test]
fn the_legs_are_checked_against_each_other() {
    let motion = motion(0, vec![leg(text("OK"), 1_000), leg(text("NO"), 1_000)]);

    let refusal = check(&motion, &text("OK")).expect_err("the second leg");

    assert!(refusal.contains("text"), "{refusal}");
}

#[test]
fn a_looping_motion_repeats_its_legs_after_they_end() {
    let motion = Motion::looping(500, vec![leg(rect_at(10), 1_000), leg(rect_at(0), 1_000)]);

    assert_eq!(x_of(&at(&motion, &rect_at(0), 400)), 0);
    assert_eq!(x_of(&at(&motion, &rect_at(0), 1_000)), 5);
    assert_eq!(x_of(&at(&motion, &rect_at(0), 2_000)), 5);
    assert_eq!(x_of(&at(&motion, &rect_at(0), 3_000)), 5);
}

#[test]
fn a_motion_without_legs_may_not_loop() {
    let motion = Motion::looping(0, Vec::new());

    let refusal = check(&motion, &rect_at(0)).expect_err("no legs to loop");

    assert!(refusal.contains("loop"), "{refusal}");
}
