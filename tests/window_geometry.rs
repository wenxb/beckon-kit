use gpui::{Bounds, Corners, RoundedClip, point, px, size};

#[test]
fn application_hit_clip_rejects_hidden_corners_without_rejecting_the_center() {
    let clip = RoundedClip::new(
        Bounds::new(point(px(10.0), px(20.0)), size(px(100.0), px(80.0))),
        Corners::all(px(20.0)),
    );
    assert!(!clip.contains(point(px(10.0), px(20.0))));
    assert!(!clip.contains(point(px(109.0), px(99.0))));
    assert!(clip.contains(point(px(60.0), px(60.0))));
    assert!(clip.contains(point(px(30.0), px(20.0))));
    assert!(!clip.contains(point(px(9.0), px(60.0))));
}

#[test]
fn corner_radii_are_clamped_to_small_client_areas() {
    let clip = RoundedClip::new(
        Bounds::new(point(px(0.0), px(0.0)), size(px(10.0), px(10.0))),
        Corners::all(px(1000.0)),
    );
    assert!(clip.radii.top_left <= px(5.0));
    assert!(clip.contains(point(px(5.0), px(5.0))));
}
