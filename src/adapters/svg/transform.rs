//! Reading the transform an export writes on a group.
//!
//! An export moves its groups with `transform="translate(x y)"`, and Penpot
//! writes the same move as a matrix. A matrix whose linear part is the
//! identity is exactly a translation, so it is read as one; any other matrix
//! rotates or scales, which this console does not draw.
//!
//! `rotate(...)` and `scale(...)` are not implemented at all: they change what
//! a shape is, and a shape the console cannot place is left out of the panel
//! rather than guessed at.

/// What a transform does that the console can act on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transform {
    /// The transform moves everything by this much.
    Shift(i32, i32),
    /// The transform does something this console does not draw, and why.
    Undrawn(String),
}

/// The transform a word writes.
pub fn transform(word: &str) -> Transform {
    let word = word.trim();
    if let Some(inner) = word
        .strip_prefix("translate(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts = numbers(inner);
        return match parts.as_slice() {
            [x] => Transform::Shift(*x, 0),
            [x, y] => Transform::Shift(*x, *y),
            _ => Transform::Undrawn(format!("`{word}` needs one or two numbers")),
        };
    }
    if let Some(inner) = word
        .strip_prefix("matrix(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts = decimals(inner);
        if parts.len() != 6 {
            return Transform::Undrawn(format!("`{word}` needs six numbers"));
        }
        let (a, b, c, d, e, f) = (parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]);
        if b == 0.0 && c == 0.0 && a == 1.0 && d == 1.0 {
            return Transform::Shift(e.round() as i32, f.round() as i32);
        }
        return Transform::Undrawn(format!(
            "the matrix transform `{word}` rotates or scales, which this console does not draw"
        ));
    }
    Transform::Undrawn(format!(
        "the transform `{word}` is not one this console draws"
    ))
}

/// The numbers of a transform, as whole pixels.
fn numbers(inner: &str) -> Vec<i32> {
    decimals(inner)
        .into_iter()
        .map(|value| value.round() as i32)
        .collect()
}

/// The numbers of a transform, as they were written.
fn decimals(inner: &str) -> Vec<f64> {
    inner
        .split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<f64>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_translation_is_read() {
        assert_eq!(transform("translate(10 3)"), Transform::Shift(10, 3));
    }

    #[test]
    fn an_identity_matrix_is_a_translation() {
        assert_eq!(
            transform("matrix(1.000000, 0.000000, 0.000000, 1.000000, 4.000000, 5.000000)"),
            Transform::Shift(4, 5)
        );
    }

    #[test]
    fn a_matrix_that_scales_is_not_drawn() {
        assert!(matches!(
            transform("matrix(2, 0, 0, 2, 0, 0)"),
            Transform::Undrawn(_)
        ));
    }

    #[test]
    fn a_rotation_is_not_drawn() {
        assert!(matches!(transform("rotate(45)"), Transform::Undrawn(_)));
    }
}
