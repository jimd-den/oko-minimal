//! The motion a directive may state.
//!
//! A shape may say what it becomes and when: a delay, and then any number of
//! legs. A leg says what the shape becomes and how long the change takes.
//!
//! The clauses are recognised by their KEYWORD, never by the length of the
//! fields before them. That is what lets a shape of any length carry a
//! motion: a path with twenty segments ends its fields at the word `to` just
//! as a rectangle with five numbers does.
//!
//! The shape and the target are handed to the core, which refuses the pairs
//! that may not be one - a text that becomes a different text, a clear that
//! moves - and names them.

use super::super::motion::{Leg, Motion};
use super::fields::{self, Kind};
use super::words::{AFTER, LOOP, OVER, TO, Words};

/// A duration, read from the next words: a number and a unit.
///
/// The unit is always written, so `500ms` and `2s` are the only spellings,
/// and a fraction of a second is written in milliseconds. A duration is the
/// motion's own unit, so it is read here rather than among the words.
pub(crate) fn duration(words: &mut Words) -> Result<u64, String> {
    let word = words.next().unwrap_or_default();
    let digits = word.chars().take_while(char::is_ascii_digit).count();
    let (number, unit) = word.split_at(digits);
    if number.is_empty() {
        return Err("a duration needs a number, like `500ms`".to_string());
    }
    let value: u64 = number
        .parse()
        .map_err(|_| format!("`{word}` is too large to be a duration"))?;
    let unit = if unit.is_empty() {
        words.next().unwrap_or_default().to_string()
    } else {
        unit.to_string()
    };
    match unit.as_str() {
        "ms" => Ok(value),
        "s" => value
            .checked_mul(1000)
            .ok_or_else(|| format!("`{word}` is too large to be a duration")),
        "" => Err("a duration needs a unit, `ms` or `s`".to_string()),
        other => Err(format!(
            "unknown unit `{other}`; a duration is in `ms` or `s`"
        )),
    }
}

/// The motion the words after a shape's own fields state.
pub(crate) fn read(words: &mut Words, kind: Kind) -> Result<Motion, String> {
    let mut delay = 0;
    if words.is(0, AFTER) {
        words.next();
        delay = duration(words)?;
    }

    let mut legs = Vec::new();
    while words.is(0, TO) {
        words.next();
        let target = fields::target(kind, words)?;
        if !words.is(0, OVER) {
            return Err("a leg needs `over` and a duration".to_string());
        }
        words.next();
        let over = duration(words)?;
        legs.push(Leg::new(target, over));
    }
    if words.is(0, OVER) {
        return Err("`over` needs a `to` to say what the shape becomes".to_string());
    }
    if words.is(0, LOOP) {
        words.next();
        return Ok(Motion::looping(delay, legs));
    }

    Ok(Motion::new(delay, legs))
}

#[cfg(test)]
mod tests {
    use super::super::words::Words;
    use super::*;

    fn duration_of(line: &str) -> Result<u64, String> {
        let mut words = Words::new(line).expect("a line of words");
        duration(&mut words)
    }

    #[test]
    fn a_duration_is_read_in_milliseconds() {
        assert_eq!(duration_of("500ms"), Ok(500));
    }

    #[test]
    fn a_duration_in_seconds_is_read_in_milliseconds() {
        assert_eq!(duration_of("2s"), Ok(2_000));
    }

    #[test]
    fn a_duration_with_no_number_is_refused() {
        let error = duration_of("ms").expect_err("no number");

        assert!(error.contains("number"), "{error}");
    }

    #[test]
    fn a_duration_with_no_unit_is_refused() {
        let error = duration_of("500").expect_err("no unit");

        assert!(error.contains("unit"), "{error}");
    }

    #[test]
    fn a_duration_with_an_unknown_unit_is_refused() {
        let error = duration_of("2weeks").expect_err("an unknown unit");

        assert!(error.contains("weeks"), "{error}");
    }
}
