#![cfg(feature = "test-support")]
use beckon_kit::{WindowConfig, WindowRoot, WindowRootHandleExt, components::Button, test_support};
use gpui::{AppContext, Context, IntoElement, Render, Window, div, prelude::*, px, size};

struct Content {
    clicks: usize,
}
impl Render for Content {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            Button::new("overlay-button", "浮层按钮").on_click(cx.listener(|view, _, _, cx| {
                view.clicks += 1;
                cx.notify();
            })),
        )
    }
}

#[gpui::test]
fn prepared_options_interoperate_with_gpui_and_overlay_is_in_visible_client_area(
    cx: &mut gpui::TestAppContext,
) {
    let handle = cx.update(|cx| {
        let prepared = WindowConfig::new("低层组合")
            .size(size(px(500.0), px(400.0)))
            .prepare(cx)
            .unwrap();
        cx.try_open_window(prepared.options, |window, cx| {
            WindowRoot::mount(prepared.root, window, cx, |_, cx| {
                cx.new(|_| Content { clicks: 0 })
            })
        })
        .unwrap()
    });
    let overlay = cx.new(|_| Content { clicks: 0 });
    handle
        .update(cx, |root, _, cx| {
            root.set_overlay(Some(overlay.clone().into()), cx)
        })
        .unwrap();
    test_support::click(cx, handle.into(), "overlay-button");
    assert_eq!(overlay.read_with(cx, |content, _| content.clicks), 1);
    assert_eq!(
        handle
            .content(cx)
            .unwrap()
            .read_with(cx, |content, _| content.clicks),
        0
    );
    handle
        .update(cx, |root, _, cx| root.set_overlay(None, cx))
        .unwrap();
    test_support::click(cx, handle.into(), "overlay-button");
    assert_eq!(
        handle
            .content(cx)
            .unwrap()
            .read_with(cx, |content, _| content.clicks),
        1
    );
}

struct TitlebarControls(usize);
impl Render for TitlebarControls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Button::new("titlebar-button", "标题按钮").on_click(cx.listener(|view, _, _, cx| {
            view.0 += 1;
            cx.notify();
        }))
    }
}

#[cfg(target_os = "linux")]
#[gpui::test]
fn titlebar_slot_controls_do_not_begin_a_window_drag(cx: &mut gpui::TestAppContext) {
    let handle = cx.update(|cx| {
        WindowConfig::new("标题区")
            .open(cx, |_, cx| cx.new(|_| Content { clicks: 0 }))
            .unwrap()
    });
    let controls = cx.new(|_| TitlebarControls(0));
    handle
        .update(cx, |root, _, cx| {
            root.set_titlebar_content(Some(controls.clone().into()), cx);
        })
        .unwrap();
    test_support::click(cx, handle.into(), "titlebar-button");
    assert_eq!(controls.read_with(cx, |content, _| content.0), 1);
    assert!(
        !cx.window_frame_requests(handle.into())
            .contains(&gpui::WindowFrameRequest::Move)
    );
}

#[gpui::test]
fn failed_root_assembly_does_not_leave_a_window_identity(cx: &mut gpui::TestAppContext) {
    cx.update(|cx| {
        let before = cx.windows().len();
        let failed = cx.try_open_window::<Content>(Default::default(), |_, _| {
            Err(std::io::Error::other("root assembly failed").into())
        });
        assert!(failed.is_err());
        assert_eq!(cx.windows().len(), before);
    });
}
