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
    if motion.is_looping() {
        let cycle: u64 = motion.legs().iter().map(|leg| leg.over()).sum();
        if cycle > 0 {
            elapsed %= cycle;
        }
    }
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
        if motion.is_looping() {
            return Err("a motion with no legs cannot loop".to_string());
        }
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
