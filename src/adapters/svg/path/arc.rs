//! An arc, spelled out as the cubics that stand for it.
//!
//! The document has no arc command and the core has no arc primitive: an
//! arc is a curve, and a curve is a cubic. This module reads the command
//! and turns the arc into turns of ninety degrees or less, each one a
//! cubic, so that the document keeps a curve and the shape keeps its
//! place.
//!
//! The control points of a turn sit on the tangent, at the factor
//! `4/3 · tan(δ/4)` along it. That is the standard approximation, and its
//! error is under three parts in ten thousand of the radius — less than a
//! pixel of a panel, and less than the grid the points are put on.

use super::centre::{self, Arc, Ellipse};
use super::reader::Reader;
use super::state::{Previous, State};
use crate::adapters::svg::error::SvgError;
use crate::core::path::Segment;

/// A quarter turn, in radians.
const QUARTER_TURN: f64 = std::f64::consts::FRAC_PI_2;

/// A piece of path, in the arc's own arithmetic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Piece {
    /// A straight edge, which is what an arc of no radius is.
    Line { to: (f64, f64) },
    /// A cubic, which is what an arc is.
    Cubic {
        control1: (f64, f64),
        control2: (f64, f64),
        to: (f64, f64),
    },
}

/// Read an arc command and make the segments it stands for.
pub(super) fn command(
    reader: &mut Reader<'_>,
    state: &mut State,
    relative: bool,
) -> Result<Vec<Segment>, SvgError> {
    let rx = reader.number()?;
    let ry = reader.number()?;
    let rotation = reader.number()?;
    let large = reader.flag()?;
    let sweep = reader.flag()?;
    let to = reader.pair_at(state.current, relative)?;
    let arc = Arc {
        rx,
        ry,
        rotation,
        large,
        sweep,
        to,
    };
    let mut segments = Vec::new();
    for piece in pieces(state.current, &arc) {
        segments.push(match piece {
            Piece::Line { to } => state.line(to)?,
            Piece::Cubic {
                control1,
                control2,
                to,
            } => state.cubic(control1, control2, to)?,
        });
    }
    // An arc is not a cubic command, so a smooth command after it starts
    // from the current point.
    state.previous = Previous::Other;
    Ok(segments)
}

/// The pieces of path an arc makes, in the arc's own arithmetic.
pub(super) fn pieces(from: (f64, f64), arc: &Arc) -> Vec<Piece> {
    // A radius of zero is a straight line to the end, and the same point
    // twice is no arc at all: the specification says both.
    if arc.rx == 0.0 || arc.ry == 0.0 {
        return vec![Piece::Line { to: arc.to }];
    }
    if from == arc.to {
        return Vec::new();
    }
    turns(&centre::ellipse(from, arc))
}

/// The turns of an arc: at most ninety degrees each.
fn turns(ellipse: &Ellipse) -> Vec<Piece> {
    let count = (ellipse.span.abs() / QUARTER_TURN).ceil().max(1.0) as usize;
    let step = ellipse.span / count as f64;
    (0..count)
        .map(|index| {
            let from = ellipse.start + step * index as f64;
            cubic(ellipse, from, from + step)
        })
        .collect()
}

/// The cubic that stands for the turn from one angle to the next.
fn cubic(ellipse: &Ellipse, from: f64, to: f64) -> Piece {
    let (start, start_tangent) = at(ellipse, from);
    let (end, end_tangent) = at(ellipse, to);
    let alpha = 4.0 / 3.0 * ((to - from) / 4.0).tan();
    Piece::Cubic {
        control1: (
            start.0 + alpha * start_tangent.0,
            start.1 + alpha * start_tangent.1,
        ),
        control2: (end.0 - alpha * end_tangent.0, end.1 - alpha * end_tangent.1),
        to: end,
    }
}

/// The point of the ellipse at an angle, and the tangent there.
fn at(ellipse: &Ellipse, angle: f64) -> ((f64, f64), (f64, f64)) {
    let (cos, sin) = (angle.cos(), angle.sin());
    let (rc, rs) = (ellipse.rotation.cos(), ellipse.rotation.sin());
    let point = (
        ellipse.at.0 + ellipse.rx * cos * rc - ellipse.ry * sin * rs,
        ellipse.at.1 + ellipse.rx * cos * rs + ellipse.ry * sin * rc,
    );
    let tangent = (
        -ellipse.rx * sin * rc - ellipse.ry * cos * rs,
        -ellipse.rx * sin * rs + ellipse.ry * cos * rc,
    );
    (point, tangent)
}
