//! Reading numbers out of SVG attributes.
//!
//! Everything here is total and bounded: a word either becomes a value or a
//! message with a line number. Units (`10px`), percentages and exponents are
//! refused rather than guessed, because a translator that silently rounds a
//! unit it does not understand is worse than one that stops.

use super::error::SvgError;

/// A whole number of pixels.
pub fn integer(word: &str, line: usize) -> Result<i32, SvgError> {
    Ok(scalar(word, line)?.round() as i32)
}

/// A number, in pixels.
pub fn scalar(word: &str, line: usize) -> Result<f64, SvgError> {
    let value: f64 = word
        .trim()
        .parse()
        .map_err(|_| SvgError::new(line, format!("`{word}` is not a number")))?;
    if !value.is_finite() || value.abs() > 1_000_000.0 {
        return Err(SvgError::new(
            line,
            format!("`{word}` is outside the drawable range"),
        ));
    }
    Ok(value)
}

/// Whitespace or comma separated numbers, as `points` and `viewBox` use.
pub fn numbers(word: &str, line: usize) -> Result<Vec<i32>, SvgError> {
    word.split(|character: char| character.is_whitespace() || character == ',')
        .filter(|part| !part.is_empty())
        .map(|part| integer(part, line))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_is_read() {
        assert_eq!(scalar("12.5", 1), Ok(12.5));
    }

    #[test]
    fn a_word_that_is_not_a_number_is_refused() {
        let error = scalar("wide", 3).expect_err("a word");

        assert_eq!(error.line, 3);
        assert!(error.problem.contains("not a number"));
    }

    #[test]
    fn a_list_of_numbers_is_read() {
        assert_eq!(numbers("0,0 1,1 2,2", 1), Ok(vec![0, 0, 1, 1, 2, 2]));
    }
}
