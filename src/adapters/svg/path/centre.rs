//! The centre form of an arc.
//!
//! An arc command names its ends, its radii, its rotation and its two
//! flags: that is the endpoint form. This module turns it into the centre
//! form — the ellipse the arc is drawn on, and the way round it goes — by
//! the arithmetic the SVG specification spells out: the ends are taken to
//! the ellipse's own frame, the radii are made large enough to join them,
//! the centre is found, and the two angles are.

/// A half turn, in radians.
const HALF_TURN: f64 = std::f64::consts::PI;

/// The parameters of one arc command: the endpoint form.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Arc {
    /// The radius along the x axis.
    pub rx: f64,
    /// The radius along the y axis.
    pub ry: f64,
    /// The rotation of the ellipse's x axis, in degrees.
    pub rotation: f64,
    /// Whether the arc is the long way round.
    pub large: bool,
    /// Whether the arc is drawn in the positive angle direction.
    pub sweep: bool,
    /// The end of the arc.
    pub to: (f64, f64),
}

/// The ellipse an arc is drawn on, and the way round it goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Ellipse {
    /// The centre of the ellipse, in the user's frame.
    pub at: (f64, f64),
    /// The radius along the ellipse's own x axis.
    pub rx: f64,
    /// The radius along the ellipse's own y axis.
    pub ry: f64,
    /// The rotation of the ellipse's x axis, in radians.
    pub rotation: f64,
    /// The angle the arc starts at, in radians.
    pub start: f64,
    /// How far the arc turns, in radians.
    pub span: f64,
}

/// The ellipse an arc is drawn on, and the way round it goes.
pub(super) fn ellipse(from: (f64, f64), arc: &Arc) -> Ellipse {
    let frame = frame(from, arc);
    let radii = corrected(&frame);
    let centre = centre(&frame, radii);
    let (start, span) = angles(&frame, centre, radii);
    Ellipse {
        at: frame.to_user(centre),
        rx: radii.0,
        ry: radii.1,
        rotation: arc.rotation.to_radians(),
        start,
        span,
    }
}

/// The arc in the ellipse's own frame: its ends, its radii and its flags.
struct Frame<'a> {
    /// The arc being read.
    arc: &'a Arc,
    /// The point halfway between the ends, in the ellipse's own frame.
    at: (f64, f64),
    /// The same point in the user's frame.
    middle: (f64, f64),
    /// The cosine of the rotation of the ellipse.
    cos: f64,
    /// The sine of the rotation of the ellipse.
    sin: f64,
}

impl Frame<'_> {
    /// A point of the ellipse's own frame, in the user's frame.
    fn to_user(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (
            self.cos * x - self.sin * y + self.middle.0,
            self.sin * x + self.cos * y + self.middle.1,
        )
    }
}

/// The frame of an arc: the ends, in the ellipse's own frame.
fn frame(from: (f64, f64), arc: &Arc) -> Frame<'_> {
    let rotation = arc.rotation.to_radians();
    let (cos, sin) = (rotation.cos(), rotation.sin());
    let (dx, dy) = ((from.0 - arc.to.0) / 2.0, (from.1 - arc.to.1) / 2.0);
    Frame {
        arc,
        at: (cos * dx + sin * dy, -sin * dx + cos * dy),
        middle: ((from.0 + arc.to.0) / 2.0, (from.1 + arc.to.1) / 2.0),
        cos,
        sin,
    }
}

/// The radii of the ellipse, made large enough to join the ends.
///
/// The specification: when the radii are too small to reach from one end to
/// the other, they are scaled up until they do.
fn corrected(frame: &Frame) -> (f64, f64) {
    let (rx, ry) = (frame.arc.rx.abs(), frame.arc.ry.abs());
    let (x, y) = frame.at;
    let lambda = x * x / (rx * rx) + y * y / (ry * ry);
    if lambda <= 1.0 {
        return (rx, ry);
    }
    let scale = lambda.sqrt();
    (rx * scale, ry * scale)
}

/// The centre of the ellipse, in the ellipse's own frame.
///
/// The centre lies on one of two sides of the chord, and the flags say
/// which: the sign is positive when the flags differ.
fn centre(frame: &Frame, radii: (f64, f64)) -> (f64, f64) {
    let (rx, ry) = radii;
    let (x, y) = frame.at;
    let squares = rx * rx * y * y + ry * ry * x * x;
    if squares == 0.0 {
        return frame.at;
    }
    let numerator = (rx * rx * ry * ry - rx * rx * y * y - ry * ry * x * x).max(0.0);
    let sign = if frame.arc.large != frame.arc.sweep {
        1.0
    } else {
        -1.0
    };
    let factor = sign * (numerator / squares).sqrt();
    (factor * rx * y / ry, factor * -ry * x / rx)
}

/// The angle the arc starts at, and how far it turns.
///
/// The sweep flag says which way the arc turns: the span is positive when
/// the arc goes with the angle and negative when it goes against it.
fn angles(frame: &Frame, centre: (f64, f64), radii: (f64, f64)) -> (f64, f64) {
    let (rx, ry) = radii;
    let (x, y) = frame.at;
    let (cx, cy) = centre;
    let start = ((y - cy) / ry).atan2((x - cx) / rx);
    let end = ((-y - cy) / ry).atan2((-x - cx) / rx);
    let mut span = end - start;
    if !frame.arc.sweep && span > 0.0 {
        span -= 2.0 * HALF_TURN;
    }
    if frame.arc.sweep && span < 0.0 {
        span += 2.0 * HALF_TURN;
    }
    (start, span)
}
