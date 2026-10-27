mod support;

use beckon_kit::gallery::{Gallery, Page};
#[cfg(target_os = "linux")]
use beckon_kit::{
    test_support,
    theme::{Appearance, Density, Preferences},
};
#[cfg(target_os = "linux")]
use gpui::{AppContext, HeadlessAppContext};
use gpui::{TestAppContext, VisualTestContext, point, px, size};
#[cfg(target_os = "linux")]
use std::sync::Arc;

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
fn real_gpu_renders_light_dark_materials_and_narrow_layouts() {
    let mut cx = runtime();
    for (width, height, page, appearance, density, preferences, name) in [
        (
            980.0,
            680.0,
            Page::Controls,
            Appearance::Dark,
            Density::Comfortable,
            Preferences::default(),
            "controls-dark",
        ),
        (
            980.0,
            680.0,
            Page::Controls,
            Appearance::Light,
            Density::Comfortable,
            Preferences::default(),
            "controls-light",
        ),
        (
            980.0,
            680.0,
            Page::Controls,
            Appearance::Dark,
            Density::Compact,
            Preferences::default(),
            "controls-compact",
        ),
        (
            420.0,
            640.0,
            Page::Controls,
            Appearance::Light,
            Density::Comfortable,
            Preferences::default(),
            "controls-narrow",
        ),
        (
            980.0,
            680.0,
            Page::Colors,
            Appearance::Light,
            Density::Comfortable,
            Preferences::default(),
            "colors-light",
        ),
        (
            420.0,
            640.0,
            Page::Icons,
            Appearance::Dark,
            Density::Comfortable,
            Preferences::default(),
            "icons-narrow",
        ),
        (
            980.0,
            680.0,
            Page::Materials,
            Appearance::Dark,
            Density::Comfortable,
            Preferences::default(),
            "materials-dark",
        ),
        (
            980.0,
            680.0,
            Page::Materials,
            Appearance::Light,
            Density::Comfortable,
            Preferences::default(),
            "materials-light",
        ),
        (
            980.0,
            680.0,
            Page::Materials,
            Appearance::Dark,
            Density::Comfortable,
            Preferences {
                reduced_transparency: true,
                increased_contrast: false,
            },
            "materials-opaque",
        ),
        (
            420.0,
            640.0,
            Page::Materials,
            Appearance::Dark,
            Density::Comfortable,
            Preferences::default(),
            "materials-narrow",
        ),
    ] {
        let window = cx
            .open_window(size(px(width), px(height)), |_, cx| {
                cx.new(|cx| {
                    let mut gallery = Gallery::new(cx);
                    gallery.show_page(page, cx);
                    gallery.set_appearance(appearance, cx);
                    gallery.set_density(density, cx);
                    gallery.set_preferences(preferences, cx);
                    gallery
                })
            })
            .unwrap();
        cx.run_until_parked();
        test_support::render(&mut cx, window.into());
        cx.run_until_parked();
        test_support::render(&mut cx, window.into());
        let pixels = test_support::render_pixels(&mut cx, window.into());
        let scale = cx
            .update_window(window.into(), |_, window, _| window.scale_factor())
            .unwrap();
        assert_eq!(
            pixels.dimensions(),
            (
                (width * scale).round() as u32,
                (height * scale).round() as u32
            )
        );
        let top_left = pixels.get_pixel(0, 0);
        assert!(
            pixels.pixels().any(|pixel| pixel != top_left),
            "{name}: gallery must not be blank"
        );
        let (supported, blurs) = cx
            .update_window(window.into(), |_, window, _| {
                (
                    window.is_backdrop_blur_supported(),
                    window.painted_backdrop_blurs(),
                )
            })
            .unwrap();
        let should_blur = supported && page == Page::Materials && !preferences.reduced_transparency;
        assert_eq!(
            !blurs.is_empty(),
            should_blur,
            "{name}: actual capability must match the scene"
        );
        cx.update_window(window.into(), |_, window, _| window.remove_window())
            .unwrap();
    }
}

#[gpui::test]
fn material_controls_update_the_live_scene_through_click_drag_and_keyboard(
    cx: &mut TestAppContext,
) {
    let handle = support::open(cx, |_, cx| Gallery::new(cx));
    let window = handle.into();
    let gallery = handle.root(cx).unwrap();
    cx.simulate_window_resize(window, size(px(980.0), px(820.0)));
    cx.simulate_backdrop_blur_support(window, true);
    support::frame(cx, window);
    support::click(cx, window, "page-materials");
    assert_eq!(
        support::with_window(cx, window, |window, _| window
            .painted_backdrop_blurs()
            .len()),
        4 // Three material surfaces and the frosted close button.
    );
    support::click(cx, window, "gallery-scene-change");
    assert_eq!(
        gallery.read_with(cx, |gallery, _| gallery.scene_revision()),
        1
    );
    support::with_window(cx, window, |window, _| {
        let first_blur = window.painted_backdrop_blurs()[0].order;
        window
            .painted_quads()
            .into_iter()
            .find(|quad| {
                quad.order < first_blur
                    && quad.background.as_solid() == Some(gpui::rgb(0x596fab).into())
            })
            .expect("new palette must be behind the glass in the current scene")
            .bounds
    });
    let first_top = support::bounds(cx, window, "live-scene-row-3").top();
    let bounds = support::bounds(cx, window, "gallery-scene-shift");
    support::mouse_down(
        cx,
        window,
        point(bounds.left() + px(8.0), bounds.center().y),
    );
    let destination = point(bounds.left() + bounds.size.width * 0.75, bounds.center().y);
    VisualTestContext::from_window(window, cx).simulate_mouse_move(
        destination,
        Some(gpui::MouseButton::Left),
        Default::default(),
    );
    support::frame(cx, window);
    assert!(gallery.read_with(cx, |gallery, _| gallery.scene_shift()) > 0.7);
    support::mouse_up(cx, window, destination);
    support::press(cx, window, "home");
    assert_eq!(
        gallery.read_with(cx, |gallery, _| gallery.scene_shift()),
        0.0
    );
    support::press(cx, window, "end");
    assert_eq!(
        gallery.read_with(cx, |gallery, _| gallery.scene_shift()),
        1.0
    );
    let shifted_top = support::bounds(cx, window, "live-scene-row-3").top();
    assert_eq!(
        first_top - shifted_top,
        px(120.0),
        "background geometry must actually move"
    );
    support::click(cx, window, "gallery-reduce-transparency");
    assert!(
        support::with_window(cx, window, |window, _| window.painted_backdrop_blurs()).is_empty()
    );
    support::click(cx, window, "gallery-reduce-transparency");
    assert_eq!(
        support::with_window(cx, window, |window, _| window
            .painted_backdrop_blurs()
            .len()),
        4
    );
    support::click(cx, window, "gallery-contrast");
    assert!(
        support::with_window(cx, window, |window, _| window.painted_backdrop_blurs()).is_empty()
    );
}

#[gpui::test]
fn gallery_runs_its_controls_not_static_mockups(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, cx| Gallery::new(cx));
    let gallery = handle.root(cx).unwrap();
    let window = handle.into();
    cx.simulate_window_resize(window, size(px(980.0), px(680.0)));
    support::frame(cx, window);
    support::click(cx, window, "gallery-primary");
    assert_eq!(gallery.read_with(cx, |gallery, _| gallery.clicks()), 1);
    support::click(cx, window, "gallery-toggle");
    assert!(!gallery.read_with(cx, |gallery, _| gallery.toggle_value()));
    support::click(cx, window, "gallery-input");
    cx.simulate_input(window, "你好");
    assert_eq!(
        gallery.read_with(cx, |gallery, cx| gallery
            .text_field()
            .read(cx)
            .value()
            .to_owned()),
        "你好"
    );
    support::click(cx, window, "page-materials");
    support::bounds(cx, window, "material-details");
    support::click(cx, window, "gallery-reduce-transparency");
    support::frame(cx, window);
}

#[gpui::test]
fn narrow_gallery_stacks_specimens_without_horizontal_overflow(cx: &mut TestAppContext) {
    let handle = support::open(cx, |_, cx| Gallery::new(cx));
    let window = handle.into();
    cx.simulate_window_resize(window, size(px(420.0), px(640.0)));
    support::frame(cx, window);
    for selector in ["action-specimen", "selection-specimen"] {
        let bounds = support::bounds(cx, window, selector);
        assert!(bounds.left() >= px(0.0));
        assert!(
            bounds.right() <= px(420.0),
            "{selector} overflows: {bounds:?}"
        );
    }
    let sidebar = support::bounds(cx, window, "gallery-sidebar");
    let header = support::bounds(cx, window, "gallery-header");
    let content = support::bounds(cx, window, "gallery-scroll");
    assert!(sidebar.right() <= header.left());
    assert!(header.bottom() <= content.top());
    assert!(content.right() <= px(420.0));
    assert!(content.bottom() <= px(640.0));
    let gallery = handle.root(cx).unwrap();
    gallery.update(cx, |gallery, cx| gallery.show_page(Page::Materials, cx));
    support::frame(cx, window);
    assert_eq!(support::bounds(cx, window, "gallery-sidebar"), sidebar);
    for selector in ["material-card", "material-details", "material-actions"] {
        let bounds = support::bounds(cx, window, selector);
        assert!(
            bounds.right() <= px(420.0),
            "{selector} overflows: {bounds:?}"
        );
    }
}
