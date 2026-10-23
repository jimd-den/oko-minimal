//! The named face's own rules: how tall its capitals are, and where it
//! stands on the page.

use crate::core::frame::Frame;
use crate::core::geom::{Point, Rgba};
use crate::core::text::{Text, paint};
use crate::core::typeface::Setting;

/// A face the program carries, in the setting a panel may be set in.
fn inter() -> Setting {
    Setting::named("inter", None).expect("a face the program carries")
}

/// A frame with one capital painted in it, at the origin.
fn painted(face: Option<&Setting>, label: &str, scale: u8) -> Frame {
    let mut frame = Frame::new(40, 20).expect("a frame");

    paint(
        &mut frame,
        &Text {
            at: Point::new(0, 0),
            text: label,
            scale,
            spacing: 0,
            colour: Rgba::WHITE,
        },
        face,
    );

    frame
}

/// The rows the frame has a lit pixel in.
fn lit_rows(frame: &Frame) -> Vec<i64> {
    (0..i64::from(frame.height()))
        .filter(|row| {
            (0..i64::from(frame.width())).any(|column| {
                frame.color_at(Point::new(column as i32, *row as i32)) == Some(Rgba::WHITE)
            })
        })
        .collect()
}

#[test]
fn a_named_face_capital_is_as_tall_as_the_built_in_capital() {
    let builtin = lit_rows(&painted(None, "I", 1)).len() as i64;
    let named = lit_rows(&painted(Some(&inter()), "I", 1)).len() as i64;

    assert_eq!(builtin, 7, "the built-in font's capital fills its cell");
    assert!(
        (named - builtin).abs() <= 1,
        "the named face's capital is {named} pixels tall, the built-in's is {builtin}"
    );
}

#[test]
fn a_named_face_stands_on_the_built_in_font_s_baseline() {
    let builtin = lit_rows(&painted(None, "I", 1));
    let named = lit_rows(&painted(Some(&inter()), "I", 1));

    assert_eq!(named.last(), builtin.last(), "both stand on the same row");
}

#[test]
fn painting_a_label_again_reuses_the_stamps_of_the_first_painting() {
    use crate::core::typeface::cache;

    cache::clear();
    let setting = inter();

    let _ = painted(Some(&setting), "OKO", 2);
    let kept = cache::len();
    assert!(kept > 0, "the first painting filled the cache");

    let _ = painted(Some(&setting), "OKO", 2);
    assert_eq!(cache::len(), kept, "the second painting added nothing");
}
