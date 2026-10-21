//! The arithmetic of a curve: a point in fractions of a pixel, and the point
//! at any fraction along a curve.
//!
//! A curve does not live on the pixel grid. It passes between pixels, and a
//! shape that is rounded to the grid before it is drawn is a shape that has
//! been snapped out of existence. So the curve keeps its fractions here, and
//! the renderer rounds a point only when it paints it.
//!
//! The weights are Bernstein's: each control point's weight is its binomial
//! coefficient times the powers of `t` and of `1 - t`. The control polygon is
//! given from the start of the curve to its end, so a straight edge is two
//! points, a quadratic three and a cubic four.

/// A point in fractions of a pixel.
///
/// The integer point is a pixel; this one keeps the fractions, so a curve is
/// rounded to the grid only when it is painted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
}

/// A point on a curve, a fraction `t` of the way along it.
pub fn evaluate(controls: &[Sample], t: f64) -> Sample {
    let u = 1.0 - t;
    // The weights live in a fixed array rather than a vector, because this
    // runs once per sampled point of every curve in a panel, and a heap
    // allocation per point is a cost the core should not carry.
    let mut weights = [0.0; 4];
    match controls.len() {
        2 => {
            weights[0] = u;
            weights[1] = t;
        }
        3 => {
            weights[0] = u * u;
            weights[1] = 2.0 * u * t;
            weights[2] = t * t;
        }
        _ => {
            weights[0] = u * u * u;
            weights[1] = 3.0 * u * u * t;
            weights[2] = 3.0 * u * t * t;
            weights[3] = t * t * t;
        }
    }
    let mut point = Sample { x: 0.0, y: 0.0 };
    for (weight, control) in weights.iter().zip(controls) {
        point.x += weight * control.x;
        point.y += weight * control.y;
    }
    point
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A quarter arc: a cubic from the origin over to the right.
    fn arc() -> [Sample; 4] {
        [
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 0.0, y: 10.0 },
            Sample { x: 10.0, y: 10.0 },
            Sample { x: 10.0, y: 0.0 },
        ]
    }

    #[test]
    fn a_curve_starts_and_ends_where_it_was_told() {
        let controls = arc();

        assert_eq!(evaluate(&controls, 0.0), controls[0]);
        assert_eq!(evaluate(&controls, 1.0), controls[3]);
    }

    #[test]
    fn a_curve_leaves_its_chord() {
        assert!(evaluate(&arc(), 0.5).y > 1.0);
    }
}
