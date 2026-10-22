//! The path shape, as the raster paints it.
//!
//! A path is painted by the core's own path painter, and the raster only
//! hands it over: the fill is the non-zero winding rule and the stroke is
//! the edge, and both are the painter's rules.

use crate::core::geom::{Point, Rgba};
use crate::core::motion::Motion;
use crate::core::path::{Path, Segment};
use crate::core::raster::*;
use crate::core::scene::Element;
use crate::core::scene::Scene;
use crate::core::shape::Shape;

/// Put a shape with no motion into a scene.
fn push(scene: &mut Scene, shape: Shape) {
    let element = Element::new(shape, Motion::none()).expect("a shape with no motion");
    scene.push(element);
}

#[test]
fn a_path_paints_its_interior() {
    let mut scene = Scene::new();
    push(
        &mut scene,
        Shape::Path {
            path: Path {
                segments: vec![
                    Segment::Move {
                        to: Point::new(1, 1),
                    },
                    Segment::Line {
                        to: Point::new(9, 1),
                    },
                    Segment::Line {
                        to: Point::new(1, 9),
                    },
                    Segment::Close,
                ],
            },
            fill: Some(Rgba::WHITE),
            stroke: None,
        },
    );

    let frame = render(&scene, 12, 12).expect("a small frame");

    assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::WHITE));
    assert_eq!(frame.color_at(Point::new(9, 9)), Some(Rgba::BLACK));
}

#[test]
fn a_stroked_path_paints_its_edge_and_not_its_inside() {
    let mut scene = Scene::new();
    push(
        &mut scene,
        Shape::Path {
            path: Path {
                segments: vec![
                    Segment::Move {
                        to: Point::new(1, 1),
                    },
                    Segment::Line {
                        to: Point::new(9, 1),
                    },
                    Segment::Line {
                        to: Point::new(1, 9),
                    },
                    Segment::Close,
                ],
            },
            fill: None,
            stroke: Some(Rgba::WHITE),
        },
    );

    let frame = render(&scene, 12, 12).expect("a small frame");

    assert_eq!(frame.color_at(Point::new(5, 1)), Some(Rgba::WHITE));
    assert_eq!(frame.color_at(Point::new(2, 2)), Some(Rgba::BLACK));
}
