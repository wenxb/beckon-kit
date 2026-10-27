#![cfg(all(feature = "test-support", target_os = "linux"))]
mod support;

use beckon_kit::{
    WindowConfig, WindowRoot,
    theme::{Appearance, Density, Theme},
    window::CloseDecision,
};
use gpui::{
    AppContext, Context, Decorations, IntoElement, Render, TestAppContext, Tiling,
    VisualTestContext, Window, WindowControls, WindowFrameRequest, WindowHandle, div, prelude::*,
    px,
};
use std::{cell::Cell, rc::Rc};

struct Content;
impl Render for Content {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}

fn open(cx: &mut TestAppContext, config: WindowConfig) -> WindowHandle<WindowRoot<Content>> {
    let handle = cx.update(|cx| config.open(cx, |_, cx| cx.new(|_| Content)).unwrap());
    support::frame(cx, handle.into());
    handle
}

#[gpui::test]
fn caption_buttons_paint_full_circles_in_both_themes_and_densities(cx: &mut TestAppContext) {
    for appearance in [Appearance::Light, Appearance::Dark] {
        for density in [Density::Comfortable, Density::Compact] {
            let theme = Theme::new(appearance).density(density);
            let handle = open(cx, WindowConfig::new("圆形按钮").theme(theme.clone()));
            let window = handle.into();
            let titlebar = support::bounds(cx, window, "window-titlebar");
            let mut previous_right = titlebar.left();
            for selector in ["window-minimize", "window-maximize", "window-close"] {
                let bounds = support::bounds(cx, window, selector);
                assert_eq!(bounds.size.width, bounds.size.height);
                assert!(bounds.size.width >= px(27.0));
                assert!(bounds.left() >= previous_right && bounds.right() <= titlebar.right());
                assert!(bounds.top() >= titlebar.top() && bounds.bottom() <= titlebar.bottom());
                previous_right = bounds.right();
                VisualTestContext::from_window(window, cx).simulate_mouse_move(
                    bounds.center(),
                    None,
                    Default::default(),
                );
                support::frame(cx, window);
                support::with_window(cx, window, |window, _| {
                    let bounds = bounds.scale(window.scale_factor());
                    let quad = window
                        .painted_quads()
                        .into_iter()
                        .find(|quad| {
                            quad.bounds == bounds
                                && quad.background.as_solid() == Some(theme.colors.hover)
                        })
                        .expect("hover must paint the circular button surface");
                    let radius = bounds.size.width / 2.0;
                    for corner in [
                        quad.corner_radii.top_left,
                        quad.corner_radii.top_right,
                        quad.corner_radii.bottom_left,
                        quad.corner_radii.bottom_right,
                    ] {
                        assert!(
                            corner >= radius,
                            "rounded_full must reach half the diameter"
                        );
                    }
                });
                assert_eq!(support::bounds(cx, window, selector), bounds);
            }
            assert!(
                !cx.window_frame_requests(window)
                    .contains(&WindowFrameRequest::Move)
            );
        }
    }
}

#[gpui::test]
fn circular_caption_actions_keep_native_requests_keyboard_and_close_gate(cx: &mut TestAppContext) {
    let closes = Rc::new(Cell::new(0));
    let seen = closes.clone();
    let handle = open(
        cx,
        WindowConfig::new("原生操作").on_close(move |_, _, _| {
            seen.set(seen.get() + 1);
            CloseDecision::Cancel
        }),
    );
    let window = handle.into();
    support::click(cx, window, "window-minimize");
    assert_eq!(
        cx.window_frame_requests(window)
            .iter()
            .filter(|r| **r == WindowFrameRequest::Minimize)
            .count(),
        1
    );
    support::press(cx, window, "tab");
    support::key_down(cx, window, "enter");
    assert!(
        !cx.window_frame_requests(window)
            .contains(&WindowFrameRequest::Zoom)
    );
    support::key_up(cx, window, "enter");
    assert_eq!(
        cx.window_frame_requests(window)
            .iter()
            .filter(|r| **r == WindowFrameRequest::Zoom)
            .count(),
        1
    );
    assert!(
        !handle
            .read_with(cx, |root, _| root.frame_status().maximized)
            .unwrap()
    );
    cx.simulate_window_frame_state(
        window,
        Decorations::Client {
            tiling: Tiling::tiled(),
        },
        true,
        false,
        WindowControls::default(),
    );
    support::frame(cx, window);
    assert!(
        handle
            .read_with(cx, |root, _| root.frame_status().maximized)
            .unwrap()
    );
    support::click(cx, window, "window-maximize");
    assert_eq!(
        cx.window_frame_requests(window)
            .iter()
            .filter(|r| **r == WindowFrameRequest::Zoom)
            .count(),
        2
    );
    assert!(
        handle
            .read_with(cx, |root, _| root.frame_status().maximized)
            .unwrap()
    );
    support::press(cx, window, "tab");
    support::key_down(cx, window, "space");
    assert_eq!(closes.get(), 0);
    support::key_up(cx, window, "space");
    assert_eq!(closes.get(), 1);
    support::click(cx, window, "window-close");
    assert_eq!(closes.get(), 2);
    assert!(
        !cx.window_frame_requests(window)
            .contains(&WindowFrameRequest::Move)
    );
}

#[gpui::test]
fn unavailable_caption_actions_are_disabled_and_skipped_by_tab(cx: &mut TestAppContext) {
    for platform_restriction in [false, true] {
        let closes = Rc::new(Cell::new(0));
        let seen = closes.clone();
        let mut config = WindowConfig::new("不可用操作").on_close(move |_, _, _| {
            seen.set(seen.get() + 1);
            CloseDecision::Cancel
        });
        if !platform_restriction {
            config = config.options(gpui::WindowOptions {
                is_resizable: false,
                is_minimizable: false,
                ..Default::default()
            });
        }
        let handle = open(cx, config);
        let window = handle.into();
        if platform_restriction {
            cx.simulate_window_frame_state(
                window,
                Decorations::Client {
                    tiling: Tiling::default(),
                },
                false,
                false,
                WindowControls {
                    minimize: false,
                    maximize: false,
                    ..Default::default()
                },
            );
            support::frame(cx, window);
        }
        support::click(cx, window, "window-minimize");
        support::click(cx, window, "window-maximize");
        assert!(
            !cx.window_frame_requests(window)
                .contains(&WindowFrameRequest::Minimize)
        );
        assert!(
            !cx.window_frame_requests(window)
                .contains(&WindowFrameRequest::Zoom)
        );
        support::press(cx, window, "tab");
        support::press(cx, window, "enter");
        assert_eq!(
            closes.get(),
            1,
            "Tab must skip both disabled caption controls"
        );
    }
}
