//! What a shape becomes, and when.
//!
//! A motion is a delay and a list of legs. A leg says what the shape becomes
//! and how long the change takes. The legs chain: the second leg starts where
//! the first one ended, and the shape holds its last state when they are
//! done.
//!
//! This module holds what a motion IS. How a motion PLAYS OUT over time is
//! `core::timeline`, which is a different question and a different file.
//!
//! The motion has no clock. It is asked for the shape at a moment, and the
//! moment is an argument, so the same motion and the same moment always give
//! the same shape.

use super::shape::Shape;

/// One leg of a motion: the state the shape becomes, and how long it takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Leg {
    target: Shape,
    over: u64,
}

impl Leg {
    pub fn new(target: Shape, over: u64) -> Self {
        Self { target, over }
    }

    /// The state the shape becomes at the end of this leg.
    pub fn target(&self) -> &Shape {
        &self.target
    }

    /// How long the change takes, in milliseconds.
    pub fn over(&self) -> u64 {
        self.over
    }
}

/// A motion: a delay, then the legs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Motion {
    delay: u64,
    legs: Vec<Leg>,
}

impl Motion {
    /// A motion that changes nothing.
    pub fn none() -> Self {
        Self::default()
    }

    /// A motion that waits, then makes each leg in turn.
    pub fn new(delay: u64, legs: Vec<Leg>) -> Self {
        Self { delay, legs }
    }

    /// How long the shape waits before its first leg, in milliseconds.
    pub fn delay(&self) -> u64 {
        self.delay
    }

    /// The legs, in the order they happen.
    pub fn legs(&self) -> &[Leg] {
        &self.legs
    }
}
