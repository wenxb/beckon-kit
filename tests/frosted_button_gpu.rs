#![cfg(all(target_os = "linux", feature = "test-support"))]

use beckon_kit::{
    WindowConfig, WindowRootHandleExt,
    components::Button,
    test_support,
    theme::{Appearance, Theme},
    window::TitlebarMode,
};
use gpui::{
    AppContext, Context, HeadlessAppContext, IntoElement, Render, Window, div, prelude::*, px, size,
};
use std::sync::Arc;

struct Glass {
    color: u32,
}
impl Render for Glass {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .relative()
            .bg(gpui::rgb(0xffffff))
            .children((0..100).map(|x| {
                div()
                    .absolute()
                    .left(px(x as f32 * 2.0))
                    .top_0()
                    .w(px(1.0))
                    .h_full()
                    .bg(gpui::rgb(self.color))
            }))
            .child(
                Button::icon(
                    "glass",
                    "Glass",
                    div().size(px(8.0)).bg(gpui::rgb(0xff0011)),
                )
                .tooltip(false)
                .frosted()
                .absolute()
                .left(px(40.0))
                .top(px(30.0))
                .w(px(80.0))
                .h(px(80.0))
                .rounded_full(),
            )
    }
}

#[test]
fn real_button_blurs_current_scene_without_blurring_its_foreground_or_round_corners() {
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
        WindowConfig::new("Glass")
            .size(size(px(200.0), px(160.0)))
            .titlebar(TitlebarMode::None)
            .theme(Theme::new(Appearance::Light))
            .client_style(|style| {
                style
                    .radius(px(0.0))
                    .border(px(0.0), gpui::transparent_black())
                    .shadows(Vec::new())
            })
            .open(cx, |_, cx| cx.new(|_| Glass { color: 0x000000 }))
            .unwrap()
    });
    for color in [0x000000, 0x0000ff] {
        handle
            .update_content(&mut cx, |view, _, cx| {
                view.color = color;
                cx.notify();
            })
            .unwrap();
        let image = test_support::render_pixels(&mut cx, handle.into());
        let (content, scale) = handle
            .read_with(&cx, |root, _| {
                (
                    root.frame_status().geometry.content,
                    root.frame_status().scale_factor,
                )
            })
            .unwrap();
        let pixel = |x: f32, y: f32| {
            image
                .get_pixel(
                    (f32::from(content.left() + px(x)) * scale).round() as u32,
                    (f32::from(content.top() + px(y)) * scale).round() as u32,
                )
                .0
        };
        // Sample content coordinates, not device pixels or the window's resize inset.
        assert_eq!(
            pixel(42.0, 32.0),
            [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255]
        );
        assert_eq!(pixel(43.0, 32.0), [255, 255, 255, 255]);
        let a = pixel(60.0, 60.0);
        let b = pixel(61.0, 60.0);
        assert!(
            a.iter().zip(b).all(|(a, b)| a.abs_diff(b) <= 2),
            "sampled stripes must blur: {a:?} {b:?}"
        );
        assert!(
            a[0] < 255 && a[0] > 230,
            "material tint must remain translucent: {a:?}"
        );
        if color == 0x0000ff {
            assert!(a[2] > a[0], "current scene must contribute to tint: {a:?}");
        }
        assert_eq!(
            pixel(80.0, 70.0),
            [255, 0, 17, 255],
            "foreground remains sharp above the blur"
        );
        let replay = test_support::render_pixels(&mut cx, handle.into());
        assert_eq!(image, replay, "cached replay must preserve layer order");
    }
}
