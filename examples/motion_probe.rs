//! Forced full-tree redraw diagnostics, not an idle or animation-only FPS benchmark.
//! Prints native callback intervals and CPU phases without changing control motion.
//! `layout_gap_ms` includes root layout and work outside the measured subtree.
//! Run with `cargo run -p beckon-kit --features gallery --example motion_probe -- controls`.
//! Replace `controls` with `gallery` or `materials`; keep the window active while sampling.
use beckon_kit::{
    Root, WindowConfig, WindowRootHandleExt,
    components::{CheckState, Checkbox, Slider, SliderScale, Toggle},
    gallery::{Gallery, Page},
    init,
    window::FrameMode,
};
use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Element, ElementId, Entity, FocusHandle,
    GlobalElementId, InputEvent, InspectorElementId, IntoElement, KeyDownEvent, KeyUpEvent,
    Keystroke, LayoutId, Pixels, Render, Window, WindowOptions, div, prelude::*, px, size,
};
use std::{cell::RefCell, rc::Rc, time::Instant};

#[derive(Default)]
struct Sample {
    request: f64,
    layout: f64,
    prepaint: f64,
    paint: f64,
}
#[derive(Default)]
struct Metrics {
    current: Sample,
    samples: Vec<(f64, Sample)>,
    last: Option<Instant>,
}
struct Measured {
    child: AnyElement,
    metrics: Rc<RefCell<Metrics>>,
    requested_at: Option<Instant>,
}
impl IntoElement for Measured {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for Measured {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let start = Instant::now();
        let layout = self.child.request_layout(window, cx);
        self.metrics.borrow_mut().current.request += start.elapsed().as_secs_f64() * 1000.0;
        self.requested_at = Some(Instant::now());
        (layout, ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let start = Instant::now();
        if let Some(requested_at) = self.requested_at.take() {
            self.metrics.borrow_mut().current.layout +=
                start.duration_since(requested_at).as_secs_f64() * 1000.0;
        }
        self.child.prepaint(window, cx);
        self.metrics.borrow_mut().current.prepaint += start.elapsed().as_secs_f64() * 1000.0;
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let start = Instant::now();
        self.child.paint(window, cx);
        self.metrics.borrow_mut().current.paint += start.elapsed().as_secs_f64() * 1000.0;
    }
}
struct Probe {
    metrics: Rc<RefCell<Metrics>>,
    gallery: Option<Entity<Gallery>>,
    on: bool,
    changes: usize,
    value: f64,
    focus: FocusHandle,
}
impl Render for Probe {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let child = if let Some(gallery) = &self.gallery {
            gallery.clone().into_any_element()
        } else {
            Root::new("probe-root")
                .child(
                    div()
                        .w(px(300.0))
                        .p(px(20.0))
                        .flex()
                        .flex_col()
                        .gap(px(16.0))
                        .child(
                            Toggle::new("probe-toggle", "开关", self.on)
                                .focus_handle(&self.focus)
                                .on_change(cx.listener(|s, on: &bool, _, cx| {
                                    s.on = *on;
                                    s.changes += 1;
                                    cx.notify();
                                })),
                        )
                        .child(Checkbox::new(
                            "probe-check",
                            "选择",
                            if self.on {
                                CheckState::On
                            } else {
                                CheckState::Off
                            },
                        ))
                        .child(Slider::new(
                            "probe-slider",
                            "滑块",
                            SliderScale::default(),
                            self.value,
                        )),
                )
                .into_any_element()
        };
        Measured {
            child,
            metrics: self.metrics.clone(),
            requested_at: None,
        }
    }
}
fn tick(probe: &mut Probe, window: &mut Window, cx: &mut Context<Probe>, frame: usize) {
    let now = Instant::now();
    {
        let mut metrics = probe.metrics.borrow_mut();
        let sample = std::mem::take(&mut metrics.current);
        if let Some(last) = metrics.last.replace(now) {
            metrics
                .samples
                .push(((now - last).as_secs_f64() * 1000.0, sample));
        }
    }
    if frame == 100 {
        let metrics = probe.metrics.borrow();
        let samples = &metrics.samples[10..];
        for (label, mut values) in [
            (
                "callback_ms",
                samples.iter().map(|s| s.0).collect::<Vec<_>>(),
            ),
            ("request_ms", samples.iter().map(|s| s.1.request).collect()),
            (
                "layout_gap_ms",
                samples.iter().map(|s| s.1.layout).collect(),
            ),
            (
                "prepaint_ms",
                samples.iter().map(|s| s.1.prepaint).collect(),
            ),
            ("paint_ms", samples.iter().map(|s| s.1.paint).collect()),
        ] {
            values.sort_by(f64::total_cmp);
            eprintln!(
                "{label}: n={} p50={:.3} p95={:.3} max={:.3}",
                values.len(),
                values[values.len() / 2],
                values[values.len() * 95 / 100],
                values[values.len() - 1]
            );
        }
        eprintln!(
            "active={:?} refresh={:?} scale={} gpu={:?}",
            window.is_window_active(),
            window.refresh_interval(cx),
            window.scale_factor(),
            window.gpu_specs()
        );
        if probe.gallery.is_none() {
            assert!(
                probe.changes >= 3,
                "native keyboard activations reached the toggle"
            );
            eprintln!("toggle_changes={}", probe.changes);
        }
        cx.quit();
        return;
    }
    if probe.gallery.is_none() && frame.is_multiple_of(30) {
        let focus = probe.focus.clone();
        // Input listeners update Probe, so dispatch without holding its entity borrow.
        window.defer(cx, move |window, cx| {
            focus.focus(window, cx);
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: Keystroke::parse("space").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(
                KeyUpEvent {
                    keystroke: Keystroke::parse("space").unwrap(),
                }
                .to_platform_input(),
                cx,
            );
        });
        probe.value = if probe.value < 0.5 { 0.8 } else { 0.2 };
    }
    // Redraw the same complete tree on each real platform frame, not a test clock.
    window.refresh();
    cx.on_next_frame(window, move |probe, window, cx| {
        tick(probe, window, cx, frame + 1)
    });
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mode = args.first().map(String::as_str).unwrap_or("controls");
    let page = match mode {
        "controls" => None,
        "gallery" => Some(Page::Controls),
        "materials" => Some(Page::Materials),
        _ => panic!("expected controls, gallery, or materials"),
    };
    let native = args.iter().any(|a| a == "--native");
    eprintln!("probe={mode} native={native}");
    gpui_platform::application().run(move |cx| {
        init(cx);
        let handle = WindowConfig::new("Beckon motion diagnostics")
            .size(size(px(980.0), px(680.0)))
            .frame(if native {
                FrameMode::Native
            } else {
                FrameMode::Adaptive
            })
            .options(WindowOptions {
                app_id: Some("com.youngwen.beckon.motion-probe".into()),
                ..Default::default()
            })
            .open(cx, |_, cx| {
                cx.new(|cx| Probe {
                    gallery: page.map(|page| {
                        cx.new(|cx| {
                            let mut g = Gallery::new(cx);
                            g.show_page(page, cx);
                            g
                        })
                    }),
                    metrics: Rc::default(),
                    on: false,
                    changes: 0,
                    value: 0.2,
                    focus: cx.focus_handle(),
                })
            })
            .expect("open diagnostic window");
        let content = handle.content(cx).unwrap();
        handle
            .update(cx, |_, window, _| {
                window.on_next_frame(move |window, cx| {
                    content.update(cx, |probe, cx| tick(probe, window, cx, 0));
                });
            })
            .unwrap();
        cx.activate(true);
    });
}
