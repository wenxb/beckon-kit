#![cfg(feature = "test-support")]
use beckon_kit::{
    WindowConfig, WindowRoot, WindowRootHandleExt,
    components::Button,
    test_support,
    window::{CloseDecision, CloseRequest},
};
use gpui::{
    AppContext, Context, Entity, IntoElement, Render, Window, WindowFrameRequest, WindowHandle,
    div, prelude::*, px, size,
};
use std::{cell::RefCell, rc::Rc};

struct Content {
    clicks: usize,
}
impl Render for Content {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(
            Button::new("content-button", "内容按钮").on_click(cx.listener(|s, _, _, cx| {
                s.clicks += 1;
                cx.notify();
            })),
        )
    }
}
fn open(cx: &mut gpui::TestAppContext, config: WindowConfig) -> WindowHandle<WindowRoot<Content>> {
    cx.update(|cx| {
        config
            .open(cx, |_, cx| cx.new(|_| Content { clicks: 0 }))
            .unwrap()
    })
}

#[gpui::test]
fn ergonomic_handle_preserves_entity_type_and_click_is_not_window_drag(
    cx: &mut gpui::TestAppContext,
) {
    let handle = open(cx, WindowConfig::new("窗口"));
    let content: Entity<Content> = handle.content(cx).unwrap();
    test_support::click(cx, handle.into(), "content-button");
    assert_eq!(content.read_with(cx, |c, _| c.clicks), 1);
    assert!(
        !cx.window_frame_requests(handle.into())
            .contains(&WindowFrameRequest::Move)
    );
    handle.update_content(cx, |c, _, _| c.clicks += 1).unwrap();
    assert_eq!(content.read_with(cx, |c, _| c.clicks), 2);
}

#[gpui::test]
fn controls_record_requests_without_faking_state(cx: &mut gpui::TestAppContext) {
    let handle = open(cx, WindowConfig::new("窗口"));
    test_support::click(cx, handle.into(), "window-minimize");
    test_support::click(cx, handle.into(), "window-maximize");
    let requests = cx.window_frame_requests(handle.into());
    assert!(requests.contains(&WindowFrameRequest::Minimize));
    assert!(requests.contains(&WindowFrameRequest::Zoom));
    handle
        .update(cx, |root, _, _| assert!(!root.frame_status().maximized))
        .unwrap();
}

#[gpui::test]
fn native_button_and_pending_resume_share_one_close_gate(cx: &mut gpui::TestAppContext) {
    let pending = Rc::new(RefCell::new(None::<CloseRequest>));
    let seen = pending.clone();
    let handle = open(
        cx,
        WindowConfig::new("待确认").on_close(move |request, _, _| {
            *seen.borrow_mut() = Some(request);
            CloseDecision::Pending
        }),
    );
    test_support::click(cx, handle.into(), "window-close");
    let first = pending.borrow().unwrap();
    assert!(!gpui::VisualTestContext::from_window(handle.into(), cx).simulate_close());
    assert_eq!(*pending.borrow(), Some(first));
    handle
        .update(cx, |root, window, cx| {
            root.resume_close(first, false, window, cx)
        })
        .unwrap();
    test_support::click(cx, handle.into(), "window-close");
    let second = pending.borrow().unwrap();
    assert_ne!(first, second);
    handle
        .update(cx, |root, window, cx| {
            root.resume_close(first, true, window, cx)
        })
        .unwrap();
    assert!(handle.content(cx).is_ok());
    handle
        .update(cx, |root, window, cx| {
            root.resume_close(second, true, window, cx)
        })
        .unwrap();
    assert!(handle.content(cx).is_err());
}

#[gpui::test]
fn keyboard_and_native_close_can_be_cancelled(cx: &mut gpui::TestAppContext) {
    let calls = Rc::new(std::cell::Cell::new(0));
    let observed = calls.clone();
    let handle = open(
        cx,
        WindowConfig::new("保留窗口").on_close(move |_, _, _| {
            observed.set(observed.get() + 1);
            CloseDecision::Cancel
        }),
    );
    cx.simulate_keystrokes(
        handle.into(),
        if cfg!(target_os = "macos") {
            "cmd-w"
        } else {
            "ctrl-w"
        },
    );
    assert!(!gpui::VisualTestContext::from_window(handle.into(), cx).simulate_close());
    assert_eq!(calls.get(), 2);
    assert!(handle.content(cx).is_ok());
}

#[gpui::test]
fn pending_close_tokens_cannot_cross_windows(cx: &mut gpui::TestAppContext) {
    let a_token = Rc::new(RefCell::new(None));
    let b_token = Rc::new(RefCell::new(None));
    let a_seen = a_token.clone();
    let b_seen = b_token.clone();
    let a = open(
        cx,
        WindowConfig::new("甲").on_close(move |request, _, _| {
            *a_seen.borrow_mut() = Some(request);
            CloseDecision::Pending
        }),
    );
    let b = open(
        cx,
        WindowConfig::new("乙").on_close(move |request, _, _| {
            *b_seen.borrow_mut() = Some(request);
            CloseDecision::Pending
        }),
    );
    test_support::click(cx, a.into(), "window-close");
    test_support::click(cx, b.into(), "window-close");
    let token = a_token.borrow().unwrap();
    assert_ne!(Some(token), *b_token.borrow());
    b.update(cx, |root, window, cx| {
        root.resume_close(token, true, window, cx)
    })
    .unwrap();
    assert!(b.content(cx).is_ok());
}

#[gpui::test]
fn style_rejection_is_atomic_and_windows_are_isolated(cx: &mut gpui::TestAppContext) {
    let a = open(cx, WindowConfig::new("甲"));
    let b = open(cx, WindowConfig::new("乙"));
    a.update(cx, |root, window, cx| {
        let before = root.client_style().clone();
        assert!(
            root.set_client_style(|s| s.radius(px(f32::NAN)), window, cx)
                .is_err()
        );
        assert_eq!(root.client_style(), &before);
        root.set_title("新标题", window, cx);
    })
    .unwrap();
    b.update(cx, |root, _, _| assert_eq!(root.title().as_ref(), "乙"))
        .unwrap();
}

#[gpui::test]
fn user_resize_is_not_reset_by_activation_or_theme(cx: &mut gpui::TestAppContext) {
    let handle = open(
        cx,
        WindowConfig::new("大小").size(size(px(980.0), px(680.0))),
    );
    cx.simulate_window_resize(handle.into(), size(px(800.0), px(600.0)));
    handle
        .update(cx, |root, window, cx| {
            root.set_theme(beckon_kit::theme::Theme::default(), cx);
            assert_eq!(window.viewport_size(), size(px(800.0), px(600.0)));
        })
        .unwrap();
}
