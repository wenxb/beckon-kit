//! Thin utilities over GPUI's real test platform, input dispatch and debug bounds.

use gpui::{AnyWindowHandle, AppContext, Bounds, Pixels, TestAppContext, VisualTestContext};

pub fn render(cx: &mut impl AppContext, window: AnyWindowHandle) {
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .expect("render test window");
}

/// Offscreen pixels for numerical assertions. No files or desktop capture are used.
pub fn render_pixels(cx: &mut impl AppContext, window: AnyWindowHandle) -> image::RgbaImage {
    // Draw without refresh so this also exercises normal subtree cache replay.
    cx.update_window(window, |_, window, cx| {
        window.draw(cx).clear(cx);
        window.render_to_image()
    })
    .expect("update offscreen window")
    .expect("render offscreen pixels")
}

/// Debug selectors should be unique in a window; this does not bypass hit testing.
pub fn bounds(
    cx: &TestAppContext,
    window: AnyWindowHandle,
    selector: &'static str,
) -> Bounds<Pixels> {
    VisualTestContext::from_window(window, cx)
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("missing completed-frame debug selector {selector}"))
}

pub fn click(cx: &mut TestAppContext, window: AnyWindowHandle, selector: &'static str) {
    render(cx, window);
    let bounds = bounds(cx, window, selector);
    let mut visual = VisualTestContext::from_window(window, cx);
    visual.simulate_mouse_move(bounds.center(), None, Default::default());
    visual.simulate_click(bounds.center(), Default::default());
    cx.run_until_parked();
    render(cx, window);
}
