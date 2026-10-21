//! How a motion plays out over time.
//!
//! A motion says what a shape becomes and when. This module is the other
//! half of that question: given a motion and a moment, what is the shape now?
//!
//! The rules are three, and they are the whole of it. The motion is over when
//! its delay and its legs are spent - the delay counts, because a shape that
//! waits half a second and then takes two seconds to move is over at two and
//! a half. Before the delay the shape is at its start state. After the last
//! leg it holds its end state. And the legs are checked against each other,
//! not only against the first shape, because a text that becomes a different
//! text in the second leg is as wrong as one that becomes it in the first.
//!
//! The interpolation between two states is linear, and it is
//! `core::interpolate` that does it: this module decides WHERE between the
//! two states the moment falls, and that module decides what the shape
//! between them is.

use super::interpolate;
use super::motion::Motion;
use super::shape::Shape;

/// The moment the motion is over, counted from the start.
pub fn duration(motion: &Motion) -> u64 {
    motion.delay() + motion.legs().iter().map(|leg| leg.over()).sum::<u64>()
}

/// The shape at `now` milliseconds.
pub fn at(motion: &Motion, start: &Shape, now: u64) -> Shape {
    if now < motion.delay() {
        return start.clone();
    }
    let mut elapsed = now - motion.delay();
    let mut from = start.clone();
    for leg in motion.legs() {
        if elapsed < leg.over() {
            let progress = elapsed as f64 / leg.over() as f64;
            return interpolate::between(&from, leg.target(), progress);
        }
        elapsed -= leg.over();
        from = leg.target().clone();
    }
    from
}

/// Whether these legs may follow `start`.
pub fn check(motion: &Motion, start: &Shape) -> Result<(), String> {
    if motion.legs().is_empty() {
        return Ok(());
    }
    if matches!(start, Shape::Clear(_)) {
        return Err("a clear has nothing to move to, so it cannot move".to_string());
    }
    let mut from = start.clone();
    for leg in motion.legs() {
        from.can_become(leg.target())?;
        from = leg.target().clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::geom::{Point, Rect, Rgba};
    use crate::core::motion::{Leg, Motion};

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
}
