use super::{
    config::ClientFrame,
    geometry::FrameGeometry,
    platform::Platform,
    resolve::{WindowSnapshot, resolve},
    *,
};
use crate::theme::Theme;
use gpui::{Decorations, Tiling, WindowControls, px, size};

fn config() -> WindowRootConfig {
    WindowRootConfig {
        title: "测试".into(),
        frame: FrameMode::Adaptive,
        titlebar: TitlebarMode::Integrated,
        client: ClientFrame::themed(&Theme::default()),
        theme: None,
        min_size: None,
        initial_inset: None,
        resizable: true,
        minimizable: true,
        movable: true,
        close: None,
    }
}
fn snapshot(platform: Platform, decorations: Decorations) -> WindowSnapshot {
    WindowSnapshot {
        platform,
        decorations,
        negotiated: true,
        surface: size(px(1000.0), px(700.0)),
        controls: WindowControls::default(),
        active: false,
        visible: true,
        maximized: false,
        fullscreen: false,
        scale_factor: 1.0,
        rounded_clip: true,
    }
}

#[test]
fn every_tiling_combination_removes_only_affected_edges() {
    let style = config().client;
    for bits in 0..16 {
        let tiling = Tiling {
            top: bits & 1 != 0,
            right: bits & 2 != 0,
            bottom: bits & 4 != 0,
            left: bits & 8 != 0,
        };
        let g = FrameGeometry::new(
            size(px(1000.0), px(700.0)),
            true,
            &style,
            true,
            tiling,
            false,
            true,
        );
        assert_eq!(g.insets.top == px(0.0), tiling.top);
        assert_eq!(g.insets.right == px(0.0), tiling.right);
        assert_eq!(g.insets.bottom == px(0.0), tiling.bottom);
        assert_eq!(g.insets.left == px(0.0), tiling.left);
        assert_eq!(
            g.corner_radii.top_left == px(0.0),
            tiling.top || tiling.left
        );
        assert_eq!(
            g.corner_radii.bottom_right == px(0.0),
            tiling.bottom || tiling.right
        );
        assert_eq!(
            g.visible.size.width + g.insets.left + g.insets.right,
            g.surface.size.width
        );
    }
}

#[test]
fn native_ownership_never_invents_system_radius_and_dpi_does_not_scale_logical_geometry() {
    let config = config();
    for platform in [Platform::Mac, Platform::Windows, Platform::Linux] {
        let mut s = snapshot(platform, Decorations::Server);
        let first = resolve(&config, &s);
        s.scale_factor = 2.0;
        let second = resolve(&config, &s);
        assert_eq!(first.geometry, second.geometry);
        assert_eq!(first.owner, FrameOwner::System);
        assert_eq!(first.state, FrameState::SystemManaged);
        assert_eq!(first.geometry.normal_inset, px(0.0));
    }
}

#[test]
fn maximize_keeps_normal_inset_and_restore_recovers_client_geometry() {
    let config = config();
    let mut s = snapshot(
        Platform::Linux,
        Decorations::Client {
            tiling: Tiling::default(),
        },
    );
    let normal = resolve(&config, &s);
    s.maximized = true;
    let max = resolve(&config, &s);
    assert_eq!(normal.geometry.normal_inset, max.geometry.normal_inset);
    assert_eq!(max.geometry.visible, max.geometry.surface);
    s.maximized = false;
    assert_eq!(normal, resolve(&config, &s));
}

#[test]
fn explicit_requests_report_negotiation_failure() {
    let mut config = config();
    let s = snapshot(Platform::Linux, Decorations::Server);
    assert!(resolve(&config, &s).reason.is_some());
    config.frame = FrameMode::Client;
    assert_eq!(resolve(&config, &s).state, FrameState::Unsupported);
    config.frame = FrameMode::Native;
    let mut s = snapshot(
        Platform::Linux,
        Decorations::Client {
            tiling: Tiling::default(),
        },
    );
    assert_eq!(resolve(&config, &s).state, FrameState::Unsupported);
    s.rounded_clip = false;
    assert!(resolve(&config, &s).reason.unwrap().contains("GPU"));
}

#[test]
fn unacknowledged_decoration_mode_is_not_reported_as_applied() {
    let config = config();
    let mut s = snapshot(
        Platform::Linux,
        Decorations::Client {
            tiling: Tiling::default(),
        },
    );
    s.negotiated = false;
    assert_eq!(resolve(&config, &s).state, FrameState::Negotiating);
    s.negotiated = true;
    assert_eq!(resolve(&config, &s).state, FrameState::ClientApplied);
}

#[test]
fn nonresizable_shadowless_frame_still_has_a_border() {
    let style = config().client.shadows(Vec::new());
    let g = FrameGeometry::new(
        size(px(100.0), px(100.0)),
        true,
        &style,
        false,
        Tiling::default(),
        false,
        false,
    );
    assert_eq!(g.normal_inset, px(0.0));
    assert_eq!(g.border_widths.top, style.border_width);
    assert_eq!(g.resize_width, px(0.0));
}

#[test]
fn shadow_extent_and_resize_band_are_independent() {
    let frame = config().client.shadows(Vec::new());
    assert_eq!(super::geometry::normal_inset(&frame, true), px(6.0));
    assert_eq!(super::geometry::normal_inset(&frame, false), px(0.0));
    let frame = frame.shadows(vec![gpui::BoxShadow {
        offset: gpui::point(px(-10.0), px(5.0)),
        blur_radius: px(12.0),
        spread_radius: px(2.0),
        color: gpui::black(),
        inset: false,
    }]);
    assert_eq!(super::geometry::normal_inset(&frame, true), px(48.0));
}
