#![allow(dead_code)]

pub mod scene;

use gpui::{
    AnyWindowHandle, App, AppContext, Bounds, Context, InputEvent, KeyDownEvent, KeyUpEvent,
    Keystroke, MouseButton, Pixels, Point, Render, TestAppContext, VisualTestContext, Window, px,
    size,
};

pub fn frame(cx: &mut TestAppContext, handle: AnyWindowHandle) {
    cx.run_until_parked();
    cx.update_window(handle, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

pub fn open<V: Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> gpui::WindowHandle<V> {
    cx.update(beckon_kit::init);
    let handle = cx.open_window(size(px(700.0), px(500.0)), build);
    frame(cx, handle.into());
    handle
}

pub fn bounds(
    cx: &TestAppContext,
    handle: AnyWindowHandle,
    selector: &'static str,
) -> Bounds<Pixels> {
    VisualTestContext::from_window(handle, cx)
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing selector {selector}"))
}

pub fn click(cx: &mut TestAppContext, handle: AnyWindowHandle, selector: &'static str) {
    let position = bounds(cx, handle, selector).center();
    let mut visual = VisualTestContext::from_window(handle, cx);
    visual.simulate_mouse_move(position, None, Default::default());
    visual.simulate_click(position, Default::default());
    frame(cx, handle);
}

pub fn mouse_down(cx: &mut TestAppContext, handle: AnyWindowHandle, position: Point<Pixels>) {
    let mut visual = VisualTestContext::from_window(handle, cx);
    visual.simulate_mouse_move(position, None, Default::default());
    visual.simulate_mouse_down(position, MouseButton::Left, Default::default());
    frame(cx, handle);
}

pub fn mouse_up(cx: &mut TestAppContext, handle: AnyWindowHandle, position: Point<Pixels>) {
    let mut visual = VisualTestContext::from_window(handle, cx);
    visual.simulate_mouse_move(position, Some(MouseButton::Left), Default::default());
    visual.simulate_mouse_up(position, MouseButton::Left, Default::default());
    frame(cx, handle);
}

pub fn key_down(cx: &mut TestAppContext, handle: AnyWindowHandle, key: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            KeyDownEvent {
                keystroke: Keystroke::parse(key).unwrap(),
                is_held: false,
                prefer_character_input: false,
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx, handle);
}

pub fn key_up(cx: &mut TestAppContext, handle: AnyWindowHandle, key: &str) {
    cx.update_window(handle, |_, window, cx| {
        window.dispatch_event(
            KeyUpEvent {
                keystroke: Keystroke::parse(key).unwrap(),
            }
            .to_platform_input(),
            cx,
        );
    })
    .unwrap();
    frame(cx, handle);
}

pub fn press(cx: &mut TestAppContext, handle: AnyWindowHandle, key: &str) {
    key_down(cx, handle, key);
    key_up(cx, handle, key);
}

pub fn with_window<R>(
    cx: &mut TestAppContext,
    handle: AnyWindowHandle,
    f: impl FnOnce(&mut Window, &mut App) -> R,
) -> R {
    cx.update_window(handle, |_, window, cx| f(window, cx))
        .unwrap()
}
