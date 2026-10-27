#![cfg(feature = "test-support")]
use beckon_kit::{WindowConfig, components::Surface, test_support};
use gpui::{
    AppContext, Context, Entity, HeadlessAppContext, IntoElement, Render, StyleRefinement, Window,
    div, prelude::*, px, size,
};
use std::sync::Arc;

struct Cached;
impl Render for Cached {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::rgb(0xff0000))
            .child(gpui::deferred(
                div()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .w(px(600.0))
                    .h(px(600.0))
                    .bg(gpui::rgb(0x00ff00)),
            ))
    }
}
struct Content {
    cached: Entity<Cached>,
}
impl Render for Content {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(gpui::rgb(0x0000ff))
            .child(
                self.cached
                    .clone()
                    .cached(StyleRefinement::default().size_full()),
            )
            .child(
                Surface::new("glass")
                    .frosted()
                    .absolute()
                    .left(px(100.0))
                    .top(px(60.0))
                    .w(px(80.0))
                    .h(px(50.0)),
            )
    }
}

#[cfg(target_os = "linux")]
#[test]
fn window_clip_survives_cached_and_deferred_replay_with_scene_blur() {
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(()),
        || {
            Ok(Some(Box::new(
                gpui_wgpu::WgpuHeadlessRenderer::new_with_transparency(true)?,
            )))
        },
    );
    let handle = cx.update(|cx| {
        WindowConfig::new("裁剪")
            .size(size(px(320.0), px(220.0)))
            .titlebar(beckon_kit::window::TitlebarMode::None)
            .client_style(|s| s.radius(px(32.0)))
            .open(cx, |_, cx| {
                let cached = cx.new(|_| Cached);
                cx.new(|_| Content { cached })
            })
            .unwrap()
    });
    cx.run_until_parked();
    for _ in 0..3 {
        let pixels = test_support::render_pixels(&mut cx, handle.into());
        let (bounds, scale) = handle
            .read_with(&cx, |root, _| {
                (
                    root.frame_status().geometry.visible,
                    root.frame_status().scale_factor,
                )
            })
            .unwrap();
        let x = (f32::from(bounds.left()) * scale) as u32;
        let y = (f32::from(bounds.top()) * scale) as u32;
        // The outer shadow may contribute alpha, but the deferred green rectangle must not.
        assert!(
            pixels.get_pixel(x, y)[1] < 20,
            "deferred foreground escaped the rounded group"
        );
        let x = (f32::from(bounds.center().x) * scale) as u32;
        let y = (f32::from(bounds.center().y) * scale) as u32;
        assert!(pixels.get_pixel(x, y)[1] > 200);
    }
}
