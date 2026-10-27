use gpui::{Bounds, point, px, size};

use super::{
    Phase, Point, Rect, Transform,
    hit::{HitShape, arrow_points},
    input::Input,
};
#[test]
fn inverse_coordinates_compose_without_losing_small_positions() {
    let root = Transform {
        x: 12.0,
        y: 7.0,
        scale: 2.0,
    };
    let child = Transform {
        x: 4.0,
        y: 3.0,
        scale: 0.5,
    };
    let point = Point { x: 0.125, y: 0.25 };
    let composed = root.compose(child);
    assert_eq!(
        composed.map_point(point),
        root.map_point(child.map_point(point))
    );
    assert_eq!(
        composed.inverse_point(composed.map_point(point)),
        Some(point)
    );
    assert_eq!(
        Transform {
            scale: 0.001,
            ..Transform::default()
        }
        .inverse_point(Point {
            x: 0.000125,
            y: 0.00025
        }),
        Some(point)
    );
    assert_eq!(Transform { scale: 0.0, ..root }.inverse_point(point), None);
    assert_eq!(
        root.inverse_point(Point {
            x: f32::NAN,
            y: 0.0
        }),
        None
    );
}

#[test]
fn stroke_only_rectangles_do_not_hit_their_empty_interior() {
    let bounds = Rect {
        x: 10.0,
        y: 10.0,
        width: 40.0,
        height: 20.0,
    };
    let shape = HitShape::Rect {
        rect: bounds,
        filled: false,
        stroke: 2.0,
    };
    assert!(shape.contains(Point { x: 9.5, y: 20.0 }));
    assert!(!shape.contains(Point { x: 8.5, y: 20.0 }));
    assert!(!shape.contains(Point { x: 30.0, y: 20.0 }));
    assert!(
        HitShape::Rect {
            rect: bounds,
            filled: true,
            stroke: 2.0
        }
        .contains(Point { x: 30.0, y: 20.0 })
    );
}

#[test]
fn arrow_hits_follow_the_line_and_both_head_segments_not_the_bounding_box() {
    let points = arrow_points(Point { x: 0.0, y: 0.0 }, Point { x: 100.0, y: 100.0 }, 2.0).unwrap();
    let shape = HitShape::Arrow {
        points,
        tolerance: 3.0,
    };
    assert!(shape.contains(Point { x: 50.0, y: 52.0 }));
    assert!(!shape.contains(Point { x: 30.0, y: 70.0 }));
    for corner in [points[2], points[3]] {
        assert!(shape.contains(corner));
        assert!(shape.contains(Point {
            x: (corner.x + 100.0) / 2.0,
            y: (corner.y + 100.0) / 2.0
        }));
    }
    let tiny = arrow_points(Point::default(), Point { x: 1e-30, y: 0.0 }, 0.001).unwrap();
    assert!(
        tiny.iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
    );
    assert!(
        HitShape::Arrow {
            points: tiny,
            tolerance: 3.0
        }
        .contains(Point::default())
    );
    assert!(arrow_points(Point::default(), Point::default(), 1.0).is_none());
}

#[test]
fn capture_preserves_down_target_and_origin_after_adoption_and_cancels_once() {
    let mut input = Input::default();
    input.bounds = Bounds::new(point(px(100.0), px(50.0)), size(px(200.0), px(120.0)));
    input.transform = Transform {
        x: 10.0,
        y: 10.0,
        scale: 2.0,
    };
    assert!(
        input
            .next(Phase::Move, point(px(150.0), px(100.0)))
            .is_none()
    );
    let down = input
        .down(point(px(150.0), px(100.0)), Some("original".into()))
        .unwrap();
    assert_eq!(down.position, Point { x: 20.0, y: 20.0 });
    input.transform = Transform {
        x: 20.0,
        y: 20.0,
        scale: 1.0,
    };
    let moved = input
        .next(Phase::Move, point(px(400.0), px(250.0)))
        .unwrap();
    assert_eq!(moved.origin, down.origin);
    assert_eq!(moved.target, down.target);
    assert_eq!(moved.position, Point { x: 280.0, y: 180.0 });
    let cancelled = input
        .next(Phase::Cancel, point(px(410.0), px(260.0)))
        .unwrap();
    assert_eq!(cancelled.phase, Phase::Cancel);
    assert!(input.next(Phase::Up, point(px(410.0), px(260.0))).is_none());
}
