//! What a curve command means.
//!
//! The smooth commands are why this is its own module: `S` and `T` do not
//! say where their control point is, they say what it is a reflection of.
//! `S` reflects the second control point of the previous cubic, `T` the
//! control point of the previous quadratic, and either falls back to the
//! current point when the command before was not that kind of curve.
//!
//! A curve is not flattened here. It becomes a segment, and the core samples
//! it when it paints.

use super::reader::Reader;
use super::state::{Previous, State};
use crate::adapters::svg::error::SvgError;
use crate::core::path::Segment;

/// `C`: a cubic curve.
pub(super) fn cubic(
    reader: &mut Reader<'_>,
    state: &mut State,
    relative: bool,
) -> Result<Segment, SvgError> {
    let control1 = reader.pair_at(state.current, relative)?;
    let control2 = reader.pair_at(state.current, relative)?;
    let to = reader.pair_at(state.current, relative)?;
    state.cubic(control1, control2, to)
}

/// `S`: a cubic whose first control point is reflected.
pub(super) fn smooth_cubic(
    reader: &mut Reader<'_>,
    state: &mut State,
    relative: bool,
) -> Result<Segment, SvgError> {
    let control1 = match state.previous {
        Previous::Cubic(previous) => state.reflect(previous),
        _ => state.current,
    };
    let control2 = reader.pair_at(state.current, relative)?;
    let to = reader.pair_at(state.current, relative)?;
    state.cubic(control1, control2, to)
}

/// `Q`: a quadratic curve.
pub(super) fn quadratic(
    reader: &mut Reader<'_>,
    state: &mut State,
    relative: bool,
) -> Result<Segment, SvgError> {
    let control = reader.pair_at(state.current, relative)?;
    let to = reader.pair_at(state.current, relative)?;
    state.quadratic(control, to)
}

/// `T`: a quadratic whose control point is reflected.
pub(super) fn smooth_quadratic(
    reader: &mut Reader<'_>,
    state: &mut State,
    relative: bool,
) -> Result<Segment, SvgError> {
    let control = match state.previous {
        Previous::Quadratic(previous) => state.reflect(previous),
        _ => state.current,
    };
    let to = reader.pair_at(state.current, relative)?;
    state.quadratic(control, to)
}
