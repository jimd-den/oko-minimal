//! One property of a `style` attribute.
//!
//! An export hides a fill in a style attribute when its tool puts presentation
//! in CSS: `style="...; fill: rgb(176, 176, 176); ..."`. The attribute is a
//! list of declarations separated by semicolons, and each declaration is a
//! name and a value separated by a colon.
//!
//! This module reads one property at a time. It does not cascade, and it does
//! not know which properties a shape may state.

/// The value of one property in a `style` attribute, if it is there.
///
/// The name must match exactly: `fill` is not `fill-opacity`.
pub fn property(style: &str, name: &str) -> Option<String> {
    style.split(';').find_map(|declaration| {
        let (key, value) = declaration.split_once(':')?;
        if key.trim() == name {
            Some(value.trim().to_string())
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_property_is_found_among_others() {
        let style = "font-weight: 700; fill: rgb(176, 176, 176); fill-opacity: 1;";

        assert_eq!(
            property(style, "fill"),
            Some("rgb(176, 176, 176)".to_string())
        );
    }

    #[test]
    fn a_property_that_is_not_there_is_none() {
        assert_eq!(property("font-weight: 700;", "fill"), None);
    }

    #[test]
    fn a_near_name_is_not_the_property() {
        assert_eq!(property("fill-opacity: 1;", "fill"), None);
    }
}
