//! Where the path is, and the grid its points are on.
//!
//! The state of a path being read: the point it is at, the point the current
//! shape started at, and what the command before this one was — because `S`
//! reflects the second control point of the previous cubic and `T` reflects
//! the control point of the previous quadratic.
//!
//! The grid lives here too: every point a path names is a whole pixel,
//! because the panel format has no subpixel geometry.

use super::super::error::SvgError;
use super::reader::MAX_COORDINATE;
use crate::core::geom::Point;
use crate::core::path::Segment;

/// What the command before this one was.
pub(super) enum Previous {
    /// The last command was a cubic: its second control point.
    Cubic(Point),
    /// The last command was a quadratic: its control point.
    Quadratic(Point),
    /// The last command was neither.
    Other,
}

/// Where the path is now.
pub(super) struct State {
    /// The line of the export the path is on.
    pub(super) line: usize,
    /// The point the path is at.
    pub(super) current: (f64, f64),
    /// The point the current shape started at.
    pub(super) origin: (f64, f64),
    /// What the command before this one was.
    pub(super) previous: Previous,
}

impl State {
    pub(super) fn new(line: usize) -> Self {
        Self {
            line,
            current: (0.0, 0.0),
            origin: (0.0, 0.0),
            previous: Previous::Other,
        }
    }

    /// A point on the pixel grid.
    pub(super) fn at(&self, (x, y): (f64, f64)) -> Result<Point, SvgError> {
        Ok(Point::new(self.round(x)?, self.round(y)?))
    }

    /// The reflection of a control point about the current point.
    pub(super) fn reflect(&self, previous: Point) -> (f64, f64) {
        (
            2.0 * self.current.0 - f64::from(previous.x),
            2.0 * self.current.1 - f64::from(previous.y),
        )
    }

    /// A straight edge, and where the path is afterwards.
    pub(super) fn line(&mut self, to: (f64, f64)) -> Result<Segment, SvgError> {
        let point = self.at(to)?;
        self.current = to;
        Ok(Segment::Line { to: point })
    }

    /// The cubic segment, and where the path is afterwards.
    pub(super) fn cubic(
        &mut self,
        control1: (f64, f64),
        control2: (f64, f64),
        to: (f64, f64),
    ) -> Result<Segment, SvgError> {
        let control1 = self.at(control1)?;
        let control2 = self.at(control2)?;
        self.current = to;
        self.previous = Previous::Cubic(control2);
        Ok(Segment::Cubic {
            control1,
            control2,
            to: self.at(to)?,
        })
    }

    /// The quadratic segment, and where the path is afterwards.
    pub(super) fn quadratic(
        &mut self,
        control: (f64, f64),
        to: (f64, f64),
    ) -> Result<Segment, SvgError> {
        let control = self.at(control)?;
        self.current = to;
        self.previous = Previous::Quadratic(control);
        Ok(Segment::Quadratic {
            control,
            to: self.at(to)?,
        })
    }

    /// A coordinate on the pixel grid, refused when it is out of range.
    fn round(&self, value: f64) -> Result<i32, SvgError> {
        if !value.is_finite() || value.abs() > MAX_COORDINATE {
            return Err(SvgError::new(
                self.line,
                "a path coordinate is outside the drawable range",
            ));
        }
        Ok(value.round() as i32)
    }
}
