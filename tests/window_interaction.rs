#![cfg(all(feature = "test-support", target_os = "linux"))]
use beckon_kit::{
    WindowConfig, WindowRoot, WindowRootHandleExt, test_support,
    window::{FrameOwner, TitlebarMode},
};
use gpui::{
    AppContext, Context, Decorations, IntoElement, MouseButton, Render, ResizeEdge, Tiling,
    VisualTestContext, Window, WindowControls, WindowFrameRequest, WindowHandle, div, point,
    prelude::*, px, size,
};

struct Content;
impl Render for Content {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
fn open(cx: &mut gpui::TestAppContext, config: WindowConfig) -> WindowHandle<WindowRoot<Content>> {
    cx.update(|cx| config.open(cx, |_, cx| cx.new(|_| Content)).unwrap())
}

#[gpui::test]
fn explicit_drag_and_eight_resize_directions_dispatch_native_requests(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(cx, WindowConfig::new("交互"));
    test_support::click(cx, handle.into(), "window-drag");
    assert!(
        cx.window_frame_requests(handle.into())
            .contains(&WindowFrameRequest::Move)
    );
    let g = handle
        .read_with(cx, |root, _| root.frame_status().geometry.clone())
        .unwrap();
    let b = g.visible;
    for (edge, position) in [
        (ResizeEdge::Top, point(b.center().x, b.top() - px(2.0))),
        (
            ResizeEdge::Bottom,
            point(b.center().x, b.bottom() + px(2.0)),
        ),
        (ResizeEdge::Left, point(b.left() - px(2.0), b.center().y)),
        (ResizeEdge::Right, point(b.right() + px(2.0), b.center().y)),
        (ResizeEdge::TopLeft, b.origin - point(px(2.0), px(2.0))),
        (
            ResizeEdge::TopRight,
            point(b.right() + px(2.0), b.top() - px(2.0)),
        ),
        (
            ResizeEdge::BottomLeft,
            point(b.left() - px(2.0), b.bottom() + px(2.0)),
        ),
        (
            ResizeEdge::BottomRight,
            b.bottom_right() + point(px(2.0), px(2.0)),
        ),
    ] {
        let mut visual = VisualTestContext::from_window(handle.into(), cx);
        visual.simulate_mouse_move(position, None, Default::default());
        visual.simulate_mouse_down(position, MouseButton::Left, Default::default());
        // Native move/resize is allowed to take over without a mouse-up reply.
        assert!(
            cx.window_frame_requests(handle.into())
                .contains(&WindowFrameRequest::Resize(edge))
        );
    }
}

#[gpui::test]
fn negotiation_removes_client_chrome_without_restoring_creation_size(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(
        cx,
        WindowConfig::new("协商").size(size(px(980.0), px(680.0))),
    );
    cx.simulate_window_resize(handle.into(), size(px(800.0), px(600.0)));
    let visible = handle
        .read_with(cx, |r, _| r.frame_status().geometry.visible.size)
        .unwrap();
    cx.simulate_window_frame_state(
        handle.into(),
        Decorations::Server,
        false,
        false,
        WindowControls::default(),
    );
    test_support::render(cx, handle.into());
    handle
        .update(cx, |root, window, _| {
            assert_eq!(root.frame_status().owner, FrameOwner::System);
            assert_eq!(window.viewport_size(), visible);
            assert_eq!(root.frame_status().geometry.normal_inset, px(0.0));
        })
        .unwrap();
}

#[gpui::test]
fn style_updates_coalesce_and_keep_visible_size_and_minimum(cx: &mut gpui::TestAppContext) {
    let minimum = size(px(300.0), px(200.0));
    let handle = open(cx, WindowConfig::new("阴影").min_size(minimum));
    let before = handle
        .read_with(cx, |r, _| r.frame_status().geometry.visible.size)
        .unwrap();
    handle
        .update(cx, |root, window, cx| {
            root.set_client_style(|s| s.shadows(Vec::new()), window, cx)
                .unwrap();
            root.set_client_style(
                |s| s.shadows(vec![gpui::BoxShadow::new(px(0.0), px(12.0), gpui::black())]),
                window,
                cx,
            )
            .unwrap();
        })
        .unwrap();
    cx.run_until_parked();
    let g = handle
        .read_with(cx, |r, _| r.frame_status().geometry.clone())
        .unwrap();
    assert_eq!(g.visible.size, before);
    assert!(
        cx.window_frame_requests(handle.into())
            .contains(&WindowFrameRequest::Minimum(
                minimum + size(g.normal_inset * 2.0, g.normal_inset * 2.0)
            ))
    );
}

#[gpui::test]
fn no_titlebar_and_maximized_windows_have_no_resize_hit_layer(cx: &mut gpui::TestAppContext) {
    let handle = open(cx, WindowConfig::new("无标题").titlebar(TitlebarMode::None));
    cx.simulate_window_frame_state(
        handle.into(),
        Decorations::Client {
            tiling: Tiling::tiled(),
        },
        true,
        false,
        WindowControls::default(),
    );
    test_support::render(cx, handle.into());
    let before = cx.window_frame_requests(handle.into()).len();
    let mut visual = VisualTestContext::from_window(handle.into(), cx);
    visual.simulate_mouse_move(point(px(1.0), px(1.0)), None, Default::default());
    visual.simulate_click(point(px(1.0), px(1.0)), Default::default());
    assert_eq!(cx.window_frame_requests(handle.into()).len(), before);
}

#[gpui::test]
fn platform_replies_refresh_insets_and_minimum_without_double_scaling(
    cx: &mut gpui::TestAppContext,
) {
    let minimum = size(px(300.0), px(200.0));
    let handle = open(cx, WindowConfig::new("状态回复").min_size(minimum));
    let inset = handle
        .read_with(cx, |r, _| r.frame_status().geometry.normal_inset)
        .unwrap();
    let before = cx.window_frame_requests(handle.into()).len();
    cx.simulate_window_frame_state(
        handle.into(),
        Decorations::Client {
            tiling: Tiling::tiled(),
        },
        true,
        false,
        WindowControls::default(),
    );
    test_support::render(cx, handle.into());
    let requests = cx.window_frame_requests(handle.into());
    assert!(requests[before..].contains(&WindowFrameRequest::Inset(inset)));
    handle
        .update(cx, |root, _, _| {
            assert_eq!(root.frame_status().geometry.normal_inset, inset);
            assert_eq!(
                root.frame_status().geometry.visible,
                root.frame_status().geometry.surface
            );
        })
        .unwrap();
    cx.simulate_window_scale_factor_change(handle.into(), 2.0);
    test_support::render(cx, handle.into());
    let requests = cx.window_frame_requests(handle.into());
    assert!(requests.contains(&WindowFrameRequest::Minimum(
        minimum + size(inset * 2.0, inset * 2.0)
    )));
    assert!(!requests.contains(&WindowFrameRequest::Minimum(
        size(minimum.width * 2.0, minimum.height * 2.0) + size(inset * 4.0, inset * 4.0)
    )));
}

#[gpui::test]
fn drag_region_preserves_native_menu_and_double_click(cx: &mut gpui::TestAppContext) {
    let handle = open(cx, WindowConfig::new("标题交互"));
    test_support::render(cx, handle.into());
    let position = test_support::bounds(cx, handle.into(), "window-drag").center();
    let mut visual = VisualTestContext::from_window(handle.into(), cx);
    visual.simulate_mouse_move(position, None, Default::default());
    visual.simulate_mouse_down(position, MouseButton::Right, Default::default());
    visual.simulate_event(gpui::MouseDownEvent {
        position,
        button: MouseButton::Left,
        modifiers: Default::default(),
        click_count: 2,
        first_mouse: false,
    });
    let requests = cx.window_frame_requests(handle.into());
    assert!(requests.contains(&WindowFrameRequest::Menu(position)));
    assert!(requests.contains(&WindowFrameRequest::Zoom));
}

struct HitContent(usize);
impl Render for HitContent {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().on_mouse_down(
            MouseButton::Left,
            cx.listener(|view, _, _, cx| {
                view.0 += 1;
                cx.notify();
            }),
        )
    }
}

#[gpui::test]
fn hidden_corners_reject_real_child_input(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        WindowConfig::new("命中裁剪")
            .size(size(px(200.0), px(200.0)))
            .titlebar(TitlebarMode::None)
            .options(gpui::WindowOptions {
                is_resizable: false,
                ..Default::default()
            })
            .client_style(|s| s.radius(px(40.0)).shadows(Vec::new()))
            .open(cx, |_, cx| cx.new(|_| HitContent(0)))
            .unwrap()
    });
    test_support::render(cx, handle.into());
    let mut visual = VisualTestContext::from_window(handle.into(), cx);
    visual.simulate_mouse_move(point(px(1.0), px(1.0)), None, Default::default());
    visual.simulate_click(point(px(1.0), px(1.0)), Default::default());
    assert_eq!(
        handle.content(cx).unwrap().read_with(cx, |view, _| view.0),
        0
    );
    let mut visual = VisualTestContext::from_window(handle.into(), cx);
    visual.simulate_mouse_move(point(px(100.0), px(100.0)), None, Default::default());
    visual.simulate_click(point(px(100.0), px(100.0)), Default::default());
    assert_eq!(
        handle.content(cx).unwrap().read_with(cx, |view, _| view.0),
        1
    );
}

#[gpui::test]
fn duplicate_mount_is_rejected_without_replacing_the_live_root(cx: &mut gpui::TestAppContext) {
    let handle = open(cx, WindowConfig::new("唯一外框"));
    let config = cx.update(|cx| WindowConfig::new("第二外框").prepare(cx).unwrap().root);
    handle
        .update(cx, |_, window, cx| {
            assert!(WindowRoot::mount(config, window, cx, |_, cx| cx.new(|_| Content)).is_err());
        })
        .unwrap();
    assert!(handle.content(cx).is_ok());
}
