//! The elements the translator draws, and the ones it only steps over.
//!
//! SVG has more elements than a console panel can show. The ones below are the
//! subset the translator draws. The structural ones hold definitions rather
//! than pictures, and are stepped over with everything inside them.

/// The elements the translator draws.
const DRAWN: [&str; 8] = [
    "svg", "g", "rect", "line", "polyline", "path", "text", "tspan",
];

/// The elements that hold definitions or metadata rather than pictures.
const STRUCTURAL: [&str; 13] = [
    "defs",
    "style",
    "pattern",
    "linearGradient",
    "radialGradient",
    "stop",
    "clipPath",
    "title",
    "desc",
    "metadata",
    "symbol",
    "mask",
    "filter",
];

/// Whether the translator draws this element.
pub fn is_drawn(name: &str) -> bool {
    DRAWN.contains(&name)
}

/// Whether this element only holds definitions.
pub fn is_structural(name: &str) -> bool {
    STRUCTURAL.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_is_drawn() {
        assert!(is_drawn("rect"));
    }

    #[test]
    fn a_definition_is_not_drawn() {
        assert!(!is_drawn("defs"));
        assert!(is_structural("defs"));
    }

    #[test]
    fn a_circle_is_neither() {
        assert!(!is_drawn("circle"));
        assert!(!is_structural("circle"));
    }
}
