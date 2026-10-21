//! What became of one shape.
//!
//! A shape either reaches the document or it does not, and when it does not
//! the reason is in words: the console reports what it could not draw, and a
//! person reads the report rather than a type.

/// The fate of one shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The shape is in the document.
    Drawn,
    /// The shape is deliberately not painted: `fill="none"`.
    Unpainted,
    /// The shape could not be drawn, and why.
    Undrawn(String),
}
