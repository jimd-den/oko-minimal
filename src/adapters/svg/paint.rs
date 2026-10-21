//! Reading how a shape is painted.
//!
//! SVG paints with inherited presentation attributes, opacity, gradients and
//! patterns. This translator accepts the flat subset an export of a console
//! panel actually uses: a fill, a stroke, and full opacity. Everything else is
//! refused with the element's line, because a colour that is almost right is
//! harder to find than an error.

use super::error::SvgError;
use super::values;
use super::xml::Element;
use crate::core::geom::Rgba;

pub fn fill(element: &Element) -> Result<Option<Rgba>, SvgError> {
    match element.attribute("fill") {
        Some("none") => Ok(None),
        Some(word) => Ok(Some(values::colour(word, element.line)?)),
        None => Ok(Some(Rgba::BLACK)),
    }
}

/// A stroked outline without a fill is stroke-only in practice, even though SVG
/// would fill it black. The convenience is deliberate and visible in the tests.
pub fn outline_fill(element: &Element) -> Result<Option<Rgba>, SvgError> {
    if element.attribute("fill").is_none() && element.attribute("stroke").is_some() {
        return Ok(None);
    }
    fill(element)
}

pub fn stroke(element: &Element) -> Result<Option<Rgba>, SvgError> {
    match element.attribute("stroke") {
        Some("none") | None => Ok(None),
        Some(word) => Ok(Some(values::colour(word, element.line)?)),
    }
}

pub fn plain_group(element: &Element) -> Result<(), SvgError> {
    for name in ["fill", "stroke"] {
        if element.attribute(name).is_some() {
            return Err(element.error(format!(
                "`{name}` on `<{}>` would be inherited; put it on each shape",
                element.name
            )));
        }
    }
    Ok(())
}

pub fn require_opaque(element: &Element) -> Result<(), SvgError> {
    for name in ["opacity", "fill-opacity", "stroke-opacity"] {
        if let Some(word) = element.attribute(name) {
            let value = values::scalar(word, element.line)?;
            if (value - 1.0).abs() > f64::EPSILON {
                return Err(element.error(format!(
                    "`{name}` is not supported yet; draw the colour as opaque"
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(attributes: &[(&str, &str)]) -> Element {
        Element {
            name: "rect".to_string(),
            attributes: attributes
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
            closed: true,
            line: 1,
        }
    }

    #[test]
    fn a_missing_fill_is_black() {
        assert_eq!(fill(&element(&[])), Ok(Some(Rgba::BLACK)));
    }

    #[test]
    fn an_opacity_below_one_is_refused() {
        let error = require_opaque(&element(&[("opacity", "0.5")])).expect_err("an opacity");

        assert!(error.problem.contains("opacity"));
    }
}
