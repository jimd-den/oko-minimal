//! Geometry and colour: the vocabulary every other module shares.
//!
//! A point is a pixel, a rectangle is a run of pixels, and a colour is four
//! channels. Coordinates are signed because a surface may sit partly off
//! screen; sizes are unsigned because a negative width has no meaning. A `Rect`
//! is never half-empty: a zero width or height makes the whole rectangle
//! empty.
//!
//! All of it is plain data. Nothing allocates, nothing reads the clock, and
//! nothing knows about a device.

/// A pixel position. The origin is the top-left corner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// A run of pixels: `w` columns from `x`, `h` rows from `y`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// The first column past the right edge, in 64 bits so a rectangle at the
    /// edge of the coordinate space cannot wrap around.
    pub fn right(&self) -> i64 {
        i64::from(self.x) + i64::from(self.w)
    }

    /// The first row past the bottom edge, in 64 bits for the same reason.
    pub fn bottom(&self) -> i64 {
        i64::from(self.y) + i64::from(self.h)
    }

    /// The pixels both rectangles cover, or `None` when there are none.
    ///
    /// This is the only clipping primitive in the program: a painter works out
    /// what is visible once, here, and then writes without further checks.
    pub fn intersect(&self, other: Rect) -> Option<Rect> {
        let left = i64::from(self.x).max(i64::from(other.x));
        let top = i64::from(self.y).max(i64::from(other.y));
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right <= left || bottom <= top {
            return None;
        }
        debug_assert!(left >= i64::from(i32::MIN) && left <= i64::from(i32::MAX));
        Some(Rect::new(
            left as i32,
            top as i32,
            (right - left) as u32,
            (bottom - top) as u32,
        ))
    }
}

/// An opaque or translucent pixel, four channels, no colour space.
///
/// The alpha channel is carried because the framebuffer is RGBA8888. The first
/// renderer writes every pixel opaque; blending arrives with the `opacity`
/// primitive, and only then does alpha change a result.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }
}

/// A point in fractions of a pixel.
///
/// The integer point is a pixel; this one keeps the fractions, so a curve is
/// rounded to the grid only when it is painted and a small shape is not
/// snapped out of existence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
}

/// A point on a Bézier curve, a fraction `t` of the way along it.
///
/// The control polygon is given from the start of the curve to its end, so a
/// straight edge is two points, a quadratic three and a cubic four. The
/// weights are Bernstein's: each control point's weight is its binomial
/// coefficient times the powers of `t` and of `1 - t`.
pub fn evaluate(controls: &[Sample], t: f64) -> Sample {
    let u = 1.0 - t;
    let weights: Vec<f64> = match controls.len() {
        2 => vec![u, t],
        3 => vec![u * u, 2.0 * u * t, t * t],
        _ => vec![u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t],
    };
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

    #[test]
    fn two_rectangles_intersect_over_the_pixels_they_share() {
        let left = Rect::new(0, 0, 10, 10);
        let right = Rect::new(5, 5, 10, 10);

        assert_eq!(left.intersect(right), Some(Rect::new(5, 5, 5, 5)));
    }

    #[test]
    fn two_rectangles_that_only_touch_do_not_intersect() {
        let left = Rect::new(0, 0, 5, 5);
        let right = Rect::new(5, 0, 5, 5);

        assert_eq!(left.intersect(right), None);
    }

    #[test]
    fn an_empty_rectangle_intersects_nothing() {
        let empty = Rect::new(3, 3, 0, 10);

        assert_eq!(empty.intersect(Rect::new(0, 0, 10, 10)), None);
    }

    #[test]
    fn a_curve_starts_and_ends_where_it_was_told() {
        let controls = [
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 0.0, y: 10.0 },
            Sample { x: 10.0, y: 10.0 },
            Sample { x: 10.0, y: 0.0 },
        ];

        assert_eq!(evaluate(&controls, 0.0), controls[0]);
        assert_eq!(evaluate(&controls, 1.0), controls[3]);
    }

    #[test]
    fn a_curve_leaves_its_chord() {
        let controls = [
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 0.0, y: 10.0 },
            Sample { x: 10.0, y: 10.0 },
            Sample { x: 10.0, y: 0.0 },
        ];

        assert!(evaluate(&controls, 0.5).y > 1.0);
    }
}
