//! Drawing a label in a named face.
//!
//! A named face is drawn at the em size that makes its capitals as tall as
//! the built-in font's capitals at the same scale, and its baseline is
//! where the built-in font puts it (ADR 0014). The letters are the face's
//! own outlines, drawn on the fine grid and brought down, and the pen
//! moves by the glyph's own advance.
//!
//! A glyph is drawn once and kept as a stamp (ADR 0018): a frame mostly
//! presses the same letters again, so the painting of a label is a row of
//! stamps, and only a letter the program has not drawn at this size, in
//! this colour, pays for the drawing.

use super::{Text, baseline, cap_height};
use crate::core::frame::Frame;
use crate::core::geom::Point;
use crate::core::typeface::cache::{self, Key};
use crate::core::typeface::stamp::Stamp;
use crate::core::typeface::{self, Setting};

/// The em size a face is drawn at, in pixels.
///
/// The capital height is the built-in font's, and the face says how tall
/// its capitals are as a fraction of its em, so the em follows from the
/// capital height.
fn em_px(setting: &Setting, scale: u8) -> f32 {
    cap_height(scale) as f32 / setting.face.cap_ratio(setting.weight)
}

/// Paint the text in the named face.
pub fn paint(frame: &mut Frame, text: &Text, setting: &Setting) {
    let em = em_px(setting, text.scale);
    let baseline = baseline(text.at, text.scale);
    let mut space: Option<f32> = None;
    let mut pen = 0.0f32;

    for character in text.text.chars() {
        let at = Point::new(text.at.x + pen.round() as i32, baseline.y);
        let key = Key::new(
            setting.face.name(),
            character,
            em,
            setting.weight,
            text.colour,
        );
        if let Some(stamp) = cache::get(&key) {
            stamp.blit(frame, at);
            pen += stamp.advance / typeface::GRID + text.spacing as f32;
            continue;
        }
        match setting.face.glyph(character, em, setting.weight) {
            Some(glyph) => {
                let stamp = Stamp::of(&glyph, text.colour);
                stamp.blit(frame, at);
                pen += stamp.advance / typeface::GRID + text.spacing as f32;
                cache::put(key, stamp);
            }
            None => {
                let room = *space.get_or_insert_with(|| space_advance(setting, em));
                pen += room;
            }
        }
    }
}

/// The room a character the face does not have takes.
///
/// A hole in a word is still a space on the page, and the face's own space
/// is the width of the hole.
fn space_advance(setting: &Setting, em: f32) -> f32 {
    setting
        .face
        .glyph(' ', em, setting.weight)
        .map(|glyph| glyph.advance / typeface::GRID)
        .unwrap_or(em / 2.0)
}
