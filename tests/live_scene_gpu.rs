#![cfg(target_os = "linux")]

use beckon_kit::{
    components::{Button, Surface},
    test_support,
    theme::{Theme, ThemeScope},
};
use gpui::{
    AppContext, Context, Entity, HeadlessAppContext, IntoElement, Render, StyleRefinement, Window,
    div, prelude::*, px, size,
};
use std::{cell::Cell, rc::Rc, sync::Arc};

struct CachedGlass {
    renders: Rc<Cell<usize>>,
}

impl Render for CachedGlass {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        self.renders.set(self.renders.get() + 1);
        Surface::new("cached-glass")
            .frosted()
            .size_full()
            .p(px(0.0))
            .rounded(px(0.0))
            .shadow(Vec::new())
            .child(
                div()
                    .absolute()
                    .left(px(44.0))
                    .top(px(124.0))
                    .size(px(16.0))
                    .bg(gpui::rgb(0x00ff00)),
            )
    }
}

struct LiveScene {
    color: u32,
    word: &'static str,
    glass: Entity<CachedGlass>,
}

impl Render for LiveScene {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut theme = Theme::default();
        theme.colors.surface = gpui::rgb(0).into();
        let text = |left| {
            div()
                .absolute()
                .left(px(left))
                .top(px(30.0))
                .w(px(140.0))
                .text_size(px(18.0))
                .text_color(gpui::rgb(0xffffff))
                .child(self.word)
        };
        ThemeScope::new(
            theme,
            div()
                .relative()
                .size_full()
                .bg(gpui::rgb(self.color))
                .child(text(10.0))
                .child(text(170.0))
                .child(
                    Button::new("live-background-control", "Background control")
                        .absolute()
                        .left(px(172.0))
                        .top(px(72.0))
                        .w(px(140.0)),
                )
                .child(
                    self.glass.clone().cached(
                        StyleRefinement::default()
                            .absolute()
                            .left(px(160.0))
                            .top(px(0.0))
                            .w(px(160.0))
                            .h(px(192.0)),
                    ),
                ),
        )
    }
}

fn runtime() -> HeadlessAppContext {
    let mut cx = HeadlessAppContext::with_platform(
        gpui_platform::current_platform(true).text_system(),
        Arc::new(()),
        gpui_platform::current_headless_renderer,
    );
    cx.update(beckon_kit::init);
    cx
}

#[test]
fn real_text_and_controls_are_filtered_and_cached_glass_reads_the_current_frame() {
    let mut cx = runtime();
    let renders = Rc::new(Cell::new(0));
    let window = cx
        .open_window(size(px(320.0), px(192.0)), |_, cx| {
            let glass = cx.new(|_| CachedGlass {
                renders: renders.clone(),
            });
            cx.new(|_| LiveScene {
                color: 0,
                word: "||||||||||||||||",
                glass,
            })
        })
        .unwrap();
    cx.run_until_parked();
    let initial = test_support::render_pixels(&mut cx, window.into());
    let (region, foreground, scale) = cx
        .update_window(window.into(), |_, window, _| {
            assert!(window.is_backdrop_blur_supported());
            let foreground = window
                .painted_quads()
                .into_iter()
                .find(|quad| quad.background.as_solid() == Some(gpui::rgb(0x00ff00).into()))
                .expect("foreground quad")
                .bounds;
            (
                window.painted_backdrop_blurs()[0],
                foreground,
                window.scale_factor(),
            )
        })
        .unwrap();
    assert_eq!(region.bounds.origin.x.0, 160.0 * scale);
    let variation = |start: f32| {
        let left = ((start + 12.0) * scale) as u32;
        let right = ((start + 135.0) * scale) as u32;
        ((32.0 * scale) as u32..(66.0 * scale) as u32)
            .flat_map(|y| (left..right).map(move |x| (x, y)))
            .map(|(x, y)| {
                u32::from(initial.get_pixel(x, y)[0].abs_diff(initial.get_pixel(x + 1, y)[0]))
            })
            .sum::<u32>()
    };
    let sharp = variation(0.0);
    let filtered = variation(160.0);
    assert!(sharp > 1000, "real glyphs must be rasterized");
    assert!(
        filtered < sharp / 8,
        "text behind glass must lose high-frequency detail: sharp={sharp}, filtered={filtered}"
    );
    let center = foreground.center();
    assert_eq!(
        initial.get_pixel(center.x.0 as u32, center.y.0 as u32).0,
        [0, 255, 0, 255]
    );
    let cached_renders = renders.get();
    let host = window.root(&mut cx).unwrap();
    for color in [0xff0000, 0x0000ff] {
        host.update(&mut cx, |host, cx| {
            host.color = color;
            host.word = "Current frame";
            cx.notify();
        });
        cx.run_until_parked();
        let pixels = test_support::render_pixels(&mut cx, window.into());
        assert_eq!(
            renders.get(),
            cached_renders,
            "background changes must not invalidate an unrelated cached subtree"
        );
        let pixel = pixels.get_pixel((290.0 * scale) as u32, (168.0 * scale) as u32);
        let channel = if color == 0xff0000 { 0 } else { 2 };
        assert!(
            pixel[channel] > 20 && pixel[(channel + 1) % 3] == 0 && pixel[(channel + 2) % 3] == 0,
            "cached glass samples the new scene, not a cached texture: {pixel:?}"
        );
        assert_eq!(
            pixels.get_pixel(center.x.0 as u32, center.y.0 as u32).0,
            [0, 255, 0, 255]
        );
    }
}
