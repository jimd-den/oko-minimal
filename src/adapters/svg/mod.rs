//! SVG in, a panel document out.
//!
//! Penpot and Figma can both export SVG. Rather than embed either editor, the
//! console accepts their export through one bounded translator whose output is
//! the same text a person edits by hand. The document stays the source of truth:
//! readable, diffable, hot-reloadable, and independent of the tool that drew it.
//!
//! The translator is a whitelist, not a renderer. It reads `rect`, `line`,
//! `polyline`, straight-line `path`, `text` with its `tspan` lines, and `g`
//! with a translation; it refuses curves, opacity, rotation and unknown
//! elements by name and line. The output obeys the document parser's limits,
//! so a translated file can never be rejected later for a size the translator
//! accepted.
//!
//! This is the only new component in the authoring path. If the translator is
//! wrong, the document it wrote is still readable, and the fix is a diff.

mod cursor;
mod emit;
mod error;
mod paint;
pub mod path;
mod rectangle;
mod scanner;
mod shapes;
mod tag;
mod text;
mod translator;
mod values;
mod xml;

pub use error::SvgError;

/// One mebibyte of SVG is a large hand-drawn panel; beyond it, the file is not
/// an export of a console screen.
pub const MAX_INPUT: usize = 1 << 20;

pub fn translate(source: &str) -> Result<String, SvgError> {
    if source.len() > MAX_INPUT {
        return Err(SvgError::new(1, "the SVG is larger than 1 MiB"));
    }
    translator::Translator::default().run(scanner::events(source)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_translate_moves_its_children() {
        let svg = r##"<svg viewBox="0 0 20 10"><g transform="translate(10 3)"><rect x="2" y="2" width="4" height="5" fill="#ff9d00"/></g></svg>"##;

        assert_eq!(
            translate(svg).expect("a translated group"),
            "rect 12 5 4 5 #ff9d00\n"
        );
    }

    #[test]
    fn a_filled_rectangle_path_becomes_one_rect() {
        let svg = r##"<svg viewBox="0 0 10 10"><path d="M2 2 H8 V6 H2 Z" fill="#102030"/></svg>"##;

        assert_eq!(
            translate(svg).expect("a translated path"),
            "rect 2 2 6 4 #102030\n"
        );
    }

    #[test]
    fn an_svg_text_becomes_a_text_directive() {
        let svg = "<svg viewBox=\"0 0 80 48\"><text x=\"6\" y=\"24\" font-size=\"21\" fill=\"#ff9d00\">OKO</text></svg>";

        assert_eq!(
            translate(svg).expect("a translated label"),
            "text 6 3 \"OKO\" 3 #ff9d00\n"
        );
    }

    #[test]
    fn a_self_closing_text_leaves_no_style_behind() {
        let error =
            translate("<svg viewBox=\"0 0 20 20\"><text/>stray</svg>").expect_err("a stray label");

        assert!(error.problem.contains("only supported inside"));
    }

    #[test]
    fn an_unknown_element_is_refused() {
        let error = translate("<svg><circle/></svg>").expect_err("a circle");

        assert!(error.problem.contains("circle"));
    }
}
