#![cfg(feature = "test-support")]
mod support;

use beckon_kit::{
    Root,
    components::{CheckState, Checkbox, Slider, SliderScale, Toggle},
    theme::{Appearance, Theme},
};
use gpui::{
    AnyWindowHandle, AppContext, Context, Entity, IntoElement, Quad, Render, TestAppContext,
    Window, div, point, prelude::*, px,
};
use std::time::Duration;

struct Controls {
    check: CheckState,
    on: bool,
    value: f64,
    events: usize,
    disabled: bool,
    accept: bool,
}
impl Render for Controls {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Root::new("motion-root").child(
            div()
                .flex()
                .flex_col()
                .w(px(300.0))
                .gap(px(16.0))
                .p(px(20.0))
                .child(
                    Checkbox::new("check", "多选框", self.check)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|s, v: &CheckState, _, cx| {
                            s.check = *v;
                            s.events += 1;
                            cx.notify();
                        })),
                )
                .child(
                    Toggle::new("toggle", "开关", self.on)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|s, v: &bool, _, cx| {
                            s.on = *v;
                            s.events += 1;
                            cx.notify();
                        })),
                )
                .child(
                    Slider::new("slider", "滑块", SliderScale::default(), self.value)
                        .disabled(self.disabled)
                        .on_change(cx.listener(|s, v: &f64, _, cx| {
                            if s.accept {
                                s.value = *v;
                            }
                            s.events += 1;
                            cx.notify();
                        })),
                ),
        )
    }
}
fn fixture(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<Controls>) {
    let handle = support::open(cx, |_, _| Controls {
        check: CheckState::Off,
        on: false,
        value: 0.0,
        events: 0,
        disabled: false,
        accept: true,
    });
    (handle.into(), handle.root(cx).unwrap())
}
fn advance(cx: &mut TestAppContext, window: AnyWindowHandle, millis: u64) -> usize {
    cx.executor().advance_clock(Duration::from_millis(millis));
    let callbacks = cx
        .update_window(window, |_, w, cx| w.simulate_next_frame(cx))
        .unwrap();
    cx.run_until_parked();
    cx.update_window(window, |_, w, cx| w.draw(cx).clear(cx))
        .unwrap();
    callbacks
}
fn quad(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    control: &'static str,
    width: Option<f32>,
) -> Quad {
    let row = support::bounds(cx, window, control);
    support::with_window(cx, window, |w, _| {
        let scale = w.scale_factor();
        w.painted_quads()
            .into_iter()
            .find(|q| {
                let center = q.bounds.center();
                row.contains(&point(
                    px(center.x.as_f32() / scale),
                    px(center.y.as_f32() / scale),
                )) && width.map_or_else(
                    || q.border_color == Theme::default().colors.secondary,
                    |width| (q.bounds.size.width.as_f32() / scale - width).abs() < 0.01,
                )
            })
            .expect("painted control indicator")
    })
}
fn thumb_x(cx: &mut TestAppContext, window: AnyWindowHandle) -> f32 {
    let scale = support::with_window(cx, window, |w, _| w.scale_factor());
    quad(cx, window, "slider", None).bounds.center().x.as_f32() / scale
}
fn knob_x(cx: &mut TestAppContext, window: AnyWindowHandle) -> f32 {
    let scale = support::with_window(cx, window, |w, _| w.scale_factor());
    quad(cx, window, "toggle", Some(14.0))
        .bounds
        .left()
        .as_f32()
        / scale
}

fn assert_color(quad: Quad, expected: gpui::Hsla) {
    let color = quad.background.as_solid().expect("solid indicator color");
    assert!((color.h - expected.h).abs() < 1e-6);
    assert!((color.s - expected.s).abs() < 1e-6);
    assert!((color.l - expected.l).abs() < 1e-6);
    assert!((color.a - expected.a).abs() < 1e-6);
}

#[gpui::test]
fn switch_retargets_from_presentation_without_coasting_and_stops_requesting_frames(
    cx: &mut TestAppContext,
) {
    let (window, entity) = fixture(cx);
    let off = knob_x(cx, window);
    let layout = support::bounds(cx, window, "toggle");
    support::click(cx, window, "toggle");
    assert!(entity.read_with(cx, |s, _| s.on));
    assert_eq!(knob_x(cx, window), off);
    assert!(advance(cx, window, 40) > 0);
    let moving = knob_x(cx, window);
    assert!(moving > off && moving < off + 14.0);
    support::press(cx, window, "space");
    assert!(!entity.read_with(cx, |s, _| s.on));
    assert_eq!(
        knob_x(cx, window),
        moving,
        "retarget from presentation, not target"
    );
    advance(cx, window, 16);
    assert!(
        knob_x(cx, window) < moving,
        "a reversal responds immediately instead of coasting in the old direction"
    );
    advance(cx, window, 1000);
    assert_eq!(knob_x(cx, window), off);
    assert_eq!(support::bounds(cx, window, "toggle"), layout);
    assert_eq!(
        advance(cx, window, 1000),
        0,
        "settled controls do not run idle loops"
    );
    assert_eq!(entity.read_with(cx, |s, _| s.events), 2);
}

#[gpui::test]
fn checkbox_fades_both_directions_and_morphs_mixed_state_without_layout_shift(
    cx: &mut TestAppContext,
) {
    let (window, entity) = fixture(cx);
    let idle = quad(cx, window, "check", Some(18.0));
    support::click(cx, window, "check");
    assert_eq!(entity.read_with(cx, |s, _| s.check), CheckState::On);
    assert_eq!(
        quad(cx, window, "check", Some(18.0)).background,
        idle.background
    );
    advance(cx, window, 60);
    let middle = quad(cx, window, "check", Some(18.0));
    assert_ne!(middle.background, idle.background);
    assert_ne!(
        middle.background.as_solid(),
        Some(Theme::default().colors.info)
    );
    advance(cx, window, 1000);
    let selected = quad(cx, window, "check", Some(18.0));
    assert_color(selected, Theme::default().colors.info);
    entity.update(cx, |s, cx| {
        s.check = CheckState::Mixed;
        cx.notify();
    });
    support::frame(cx, window);
    assert!(
        advance(cx, window, 60) > 0,
        "mixed mark is animated even with an already selected fill"
    );
    advance(cx, window, 1000);
    support::press(cx, window, "space"); // Mixed -> On
    support::press(cx, window, "space"); // On -> Off
    advance(cx, window, 60);
    assert_ne!(
        quad(cx, window, "check", Some(18.0)).background,
        idle.background
    );
    advance(cx, window, 1000);
    let end = quad(cx, window, "check", Some(18.0));
    assert_eq!(end.background, idle.background);
    assert_eq!(end.bounds, idle.bounds);
    assert_eq!(entity.read_with(cx, |s, _| s.events), 3);
}

#[gpui::test]
fn slider_click_and_keyboard_animate_but_drag_is_direct_and_keeps_grab_offset(
    cx: &mut TestAppContext,
) {
    let (window, entity) = fixture(cx);
    let start = thumb_x(cx, window);
    support::click(cx, window, "slider");
    assert_eq!(entity.read_with(cx, |s, _| s.value), 0.5);
    assert!(
        (thumb_x(cx, window) - start).abs() <= 0.5,
        "paint rounding only, no immediate target jump"
    );
    advance(cx, window, 60);
    assert!(thumb_x(cx, window) > start);
    support::press(cx, window, "end");
    assert_eq!(entity.read_with(cx, |s, _| s.value), 1.0);
    advance(cx, window, 1000);
    let right = thumb_x(cx, window);
    support::press(cx, window, "home");
    advance(cx, window, 60);
    let moving = thumb_x(cx, window);
    assert!(moving < right && moving > start);
    let row = support::bounds(cx, window, "slider");
    let grab = point(px(moving + 3.0), row.center().y);
    support::mouse_down(cx, window, grab);
    advance(cx, window, 60);
    assert!(
        quad(cx, window, "slider", None).bounds.size.width.as_f32()
            > 14.0 * support::with_window(cx, window, |w, _| w.scale_factor()),
        "press feedback grows the thumb without changing its layout"
    );
    assert!(
        (thumb_x(cx, window) - moving).abs() < 1.5,
        "grab presentation without snapping to the center"
    );
    let destination = grab + point(px(-20.0), px(0.0));
    gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
        destination,
        Some(gpui::MouseButton::Left),
        Default::default(),
    );
    support::frame(cx, window);
    let dragged = thumb_x(cx, window);
    assert!(
        (dragged - (moving - 20.0)).abs() < 1.5,
        "drag does not lag behind the pointer"
    );
    let value_before_release = entity.read_with(cx, |s, _| s.value);
    support::mouse_up(cx, window, destination);
    assert_eq!(entity.read_with(cx, |s, _| s.value), value_before_release);
    advance(cx, window, 1000);
    assert_eq!(entity.read_with(cx, |s, _| s.value), value_before_release);
    let scale = support::with_window(cx, window, |w, _| w.scale_factor());
    assert!(
        (thumb_x(cx, window) - dragged).abs() <= 0.5 / scale + 1e-4,
        "release must not drift beyond half-device-pixel rounding as the thumb shrinks"
    );
}

#[gpui::test]
fn reduced_motion_snaps_all_three_and_disabled_controls_do_not_emit(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (window, entity) = fixture(cx);
    let off = knob_x(cx, window);
    support::click(cx, window, "toggle");
    assert_eq!(knob_x(cx, window), off + 14.0);
    support::click(cx, window, "check");
    assert_color(
        quad(cx, window, "check", Some(18.0)),
        Theme::default().colors.info,
    );
    support::click(cx, window, "slider");
    support::press(cx, window, "end");
    let row = support::bounds(cx, window, "slider");
    assert!(thumb_x(cx, window) > f32::from(row.right()) - 12.0);
    assert_eq!(advance(cx, window, 1000), 0);
    let events = entity.read_with(cx, |s, _| s.events);
    entity.update(cx, |s, cx| {
        s.disabled = true;
        cx.notify();
    });
    support::frame(cx, window);
    for id in ["check", "toggle", "slider"] {
        support::click(cx, window, id);
    }
    assert_eq!(entity.read_with(cx, |s, _| s.events), events);
}

#[gpui::test]
fn slider_animation_is_presentation_only_when_host_rejects_a_change(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx);
    entity.update(cx, |s, cx| {
        s.accept = false;
        cx.notify();
    });
    support::frame(cx, window);
    let initial = thumb_x(cx, window);
    support::click(cx, window, "slider");
    advance(cx, window, 500);
    assert_eq!(thumb_x(cx, window), initial);
    assert_eq!(entity.read_with(cx, |s, _| s.value), 0.0);
    assert!(entity.read_with(cx, |s, _| s.events) > 0);
}

#[gpui::test]
fn controls_use_front_loaded_ease_out_without_a_lingering_tail(cx: &mut TestAppContext) {
    let (window, _) = fixture(cx);
    let off = knob_x(cx, window);
    let start = thumb_x(cx, window);
    support::click(cx, window, "toggle");
    support::click(cx, window, "check");
    support::click(cx, window, "slider");
    support::press(cx, window, "end");
    let row = support::bounds(cx, window, "slider");
    let end = row.right().as_f32() - Theme::default().metrics.focus_width - 8.0;
    advance(cx, window, 80);
    let progress = (thumb_x(cx, window) - start) / (end - start);
    assert!(
        (0.5..0.7).contains(&progress),
        "cover most of the distance early, without snapping"
    );
    let fill = quad(cx, window, "check", Some(18.0)).background;
    assert_ne!(fill.as_solid(), Some(Theme::default().colors.info));
    advance(cx, window, 80);
    let progress = (thumb_x(cx, window) - start) / (end - start);
    assert!(
        (0.8..0.95).contains(&progress),
        "decelerate through the middle instead of accelerating late"
    );
    advance(cx, window, 160);
    let scale = support::with_window(cx, window, |w, _| w.scale_factor());
    assert!(
        knob_x(cx, window) - off + 0.5 / scale >= 14.0 * 0.97,
        "near target within half-device-pixel rounding"
    );
    assert!(thumb_x(cx, window) - start >= (end - start) * 0.97);
    assert_color(
        quad(cx, window, "check", Some(18.0)),
        Theme::default().colors.info,
    );
    assert_eq!(
        advance(cx, window, 0),
        0,
        "finish at 320ms without requesting tail frames"
    );
}

#[gpui::test]
fn selection_curve_approaches_both_targets_without_overshoot(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx);
    let off = knob_x(cx, window);
    let start = thumb_x(cx, window);
    let middle = support::bounds(cx, window, "slider").center().x.as_f32();
    support::click(cx, window, "toggle");
    support::click(cx, window, "check");
    support::click(cx, window, "slider");
    let (mut previous_knob, mut previous_thumb) = (off, start);
    for _ in 0..36 {
        advance(cx, window, 16);
        let knob = knob_x(cx, window);
        let thumb = thumb_x(cx, window);
        assert!(
            knob >= previous_knob && knob <= off + 14.0,
            "switch must not rebound"
        );
        assert!(
            thumb >= previous_thumb && thumb <= middle + 0.5,
            "interior slider target must not overshoot"
        );
        (previous_knob, previous_thumb) = (knob, thumb);
    }
    assert_eq!(previous_knob, off + 14.0);
    assert!((previous_thumb - middle).abs() <= 0.5);
    assert_eq!(advance(cx, window, 0), 0);
    support::click(cx, window, "toggle");
    support::click(cx, window, "check");
    support::click(cx, window, "slider");
    support::press(cx, window, "home");
    for _ in 0..36 {
        advance(cx, window, 16);
        let knob = knob_x(cx, window);
        let thumb = thumb_x(cx, window);
        assert!(
            knob <= previous_knob && knob >= off,
            "reverse path must not rebound"
        );
        assert!(thumb <= previous_thumb && thumb >= start - 0.5);
        (previous_knob, previous_thumb) = (knob, thumb);
    }
    assert_eq!(previous_knob, off);
    assert!((previous_thumb - start).abs() <= 0.5);
    assert_eq!(advance(cx, window, 0), 0);
    assert_eq!(
        entity.read_with(cx, |s, _| (s.check, s.value, s.events)),
        (CheckState::Off, 0.0, 6)
    );
}

#[gpui::test]
fn reduced_motion_can_be_enabled_mid_flight_without_more_frames(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx);
    assert_eq!(
        advance(cx, window, 100),
        0,
        "mount at the initial state, without an entrance animation"
    );
    let off = knob_x(cx, window);
    support::click(cx, window, "toggle");
    support::click(cx, window, "check");
    support::click(cx, window, "slider");
    advance(cx, window, 40);
    assert!(knob_x(cx, window) > off && knob_x(cx, window) < off + 14.0);
    let events = entity.read_with(cx, |s, _| s.events);
    cx.update(|cx| cx.set_reduce_motion(true));
    support::frame(cx, window);
    assert_eq!(knob_x(cx, window), off + 14.0);
    assert_color(
        quad(cx, window, "check", Some(18.0)),
        Theme::default().colors.info,
    );
    assert_eq!(entity.read_with(cx, |s, _| s.events), events);
    advance(cx, window, 1000); // Drain a frame queued before the preference changed.
    assert_eq!(advance(cx, window, 1000), 0);
}

#[gpui::test]
fn selection_rows_do_not_paint_hover_or_active_backgrounds(cx: &mut TestAppContext) {
    for selector in ["check", "toggle"] {
        let (window, entity) = fixture(cx);
        let fills = |cx: &mut TestAppContext| {
            support::with_window(cx, window, |w, _| {
                let mut fills = w
                    .painted_quads()
                    .iter()
                    // Focus outlines are separate transparent quads, not row fills.
                    .filter(|q| q.background.as_solid().is_some_and(|color| color.a > 0.0))
                    .map(|q| format!("{:?}", (q.bounds, q.background)))
                    .collect::<Vec<_>>();
                // Focus may reorder unchanged fills; only their geometry and color matter.
                fills.sort();
                fills
            })
        };
        let before = fills(cx);
        let row = support::bounds(cx, window, selector);
        gpui::VisualTestContext::from_window(window, cx).simulate_mouse_move(
            row.center(),
            None,
            Default::default(),
        );
        support::frame(cx, window);
        assert_eq!(fills(cx), before, "hover must not add a full-row surface");
        support::mouse_down(cx, window, row.center());
        assert_eq!(
            fills(cx),
            before,
            "mouse active must not add a full-row surface"
        );
        assert_eq!(entity.read_with(cx, |s, _| s.events), 0);
        support::mouse_up(cx, window, point(px(680.0), px(480.0)));
        support::key_down(cx, window, "space");
        assert_eq!(
            fills(cx),
            before,
            "keyboard active must not add a full-row surface"
        );
        assert_eq!(entity.read_with(cx, |s, _| s.events), 0);
        assert_eq!(support::bounds(cx, window, selector), row);
        support::key_up(cx, window, "space");
        assert_eq!(
            entity.read_with(cx, |s, _| s.events),
            1,
            "release still commits selection"
        );
    }
}

#[gpui::test]
fn selection_color_frames_blend_rgb_without_sweeping_other_hues(cx: &mut TestAppContext) {
    for appearance in [Appearance::Dark, Appearance::Light] {
        let theme = Theme::new(appearance);
        let colors = theme.colors.clone();
        cx.update(|cx| theme.install(cx));
        let (window, _) = fixture(cx);
        let off = [
            colors.elevated,
            colors.elevated,
            colors.secondary,
            colors.border,
        ];
        let on = [colors.info, colors.success, colors.on_success, colors.info];
        for (from, to) in [(off, on), (on, off)] {
            support::click(cx, window, "check");
            support::click(cx, window, "toggle");
            for frame in 1..=20 {
                advance(cx, window, 16);
                let row = support::bounds(cx, window, "check");
                let border = support::with_window(cx, window, |w, _| {
                    let scale = w.scale_factor();
                    w.painted_quads()
                        .into_iter()
                        .find(|q| {
                            row.contains(&point(
                                px(q.bounds.center().x.as_f32() / scale),
                                px(q.bounds.center().y.as_f32() / scale),
                            )) && (q.bounds.size.width.as_f32() / scale - 18.0).abs() < 0.01
                                && q.border_widths.top.as_f32() > 0.0
                        })
                        .expect("checkbox edge")
                        .border_color
                });
                let actual = [
                    quad(cx, window, "check", Some(18.0))
                        .background
                        .as_solid()
                        .unwrap(),
                    quad(cx, window, "toggle", Some(34.0))
                        .background
                        .as_solid()
                        .unwrap(),
                    quad(cx, window, "toggle", Some(14.0))
                        .background
                        .as_solid()
                        .unwrap(),
                    border,
                ];
                let t = frame as f32 / 20.0;
                let phase = 1.0 - (1.0 - t).powi(3);
                for ((actual, from), to) in actual.into_iter().zip(from).zip(to) {
                    let (actual, from, to) = (actual.to_rgb(), from.to_rgb(), to.to_rgb());
                    for (actual, from, to) in [
                        (actual.r, from.r, to.r),
                        (actual.g, from.g, to.g),
                        (actual.b, from.b, to.b),
                        (actual.a, from.a, to.a),
                    ] {
                        assert!(
                            (actual - (from + (to - from) * phase)).abs() < 1e-5,
                            "{appearance:?} frame {frame}: every channel follows the endpoint blend, not a hue sweep"
                        );
                    }
                }
            }
        }
    }
}
