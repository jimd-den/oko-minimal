//! What an element hands to the elements inside it.
//!
//! A group hands down where its origin sits and what paint is in force; a
//! label hands down the style and the colour of the text inside it. This
//! module holds that inherited context, apart from the translator that walks
//! the tree, because two modules read it: the translator for shapes, and
//! `label` for text.

use super::paint::Inherited;
use crate::core::geom::Rgba;

/// What an element hands to the elements inside it.
pub(super) struct Scope {
    /// Where the origin of this element sits, after the transforms above it.
    pub(super) offset: (i32, i32),
    /// The paints the ancestors stated.
    pub(super) paint: Inherited,
    /// The style of the text this element is inside, if any.
    pub(super) style: Option<super::text::Style>,
    /// The colour of the text this element is inside, if any.
    pub(super) colour: Option<Rgba>,
}

impl Scope {
    /// A scope with no text in it.
    pub(super) fn plain(&self) -> Scope {
        Scope {
            offset: self.offset,
            paint: self.paint.clone(),
            style: None,
            colour: None,
        }
    }
}
