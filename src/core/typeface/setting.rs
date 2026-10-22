//! The font a panel is set in: the name of a face, and the weight it is
//! set at.
//!
//! The font belongs to the panel and not to a shape. A panel that changes
//! its font changes one thing, and the shapes it draws are the shapes it
//! had; the letterforms are looked up again when the text is drawn.
//!
//! A face the program does not carry is refused by name: a panel that asks
//! for a typeface that is not here is told so, and is not drawn in some
//! other face instead.

use super::Face;

/// The font a panel is set in.
///
/// Contract: the face is one the program carries, and two settings of one
/// face at one weight are the same setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setting {
    /// The face the panel is set in.
    pub face: &'static Face,
    /// The weight, for a face that has weights.
    pub weight: Option<u16>,
}

impl Setting {
    /// The setting a name asks for.
    ///
    /// A face the program does not carry is an error, and the error names
    /// the face that was asked for.
    pub fn named(name: &str, weight: Option<u16>) -> Result<Self, String> {
        let face = Face::by_name(name)
            .ok_or_else(|| format!("the program carries no face named `{name}`"))?;
        Ok(Self { face, weight })
    }
}
