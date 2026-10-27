use std::{cell::RefCell, rc::Rc, sync::Arc};

use gpui::{
    App, AppContext, Context, Element, FocusHandle, IntoElement, MouseButton, Render, RenderOnce,
    Role, TestAppContext, VisualTestContext, Window, div, point, prelude::*, px, size,
};

use super::{
    Canvas, Node, Phase, Point, PointerEvent, Rect, Scene, Stroke, Transform, compiled::Compiled,
};

fn rect(id: &str, bounds: Rect, filled: bool) -> Node {
    Node::Rect {
        id: id.into(),
        rect: bounds,
        fill: filled.then(gpui::black),
        stroke: Some(Stroke {
            color: gpui::black(),
            width: 2.0,
        }),
    }
}

fn scene(children: Vec<Node>) -> Arc<Scene> {
    Arc::new(Scene {
        width: 200.0,
        height: 120.0,
        transform: Transform::default(),
        children,
    })
}

struct Fixture {
    scene: Arc<Scene>,
    focus: FocusHandle,
    events: Vec<PointerEvent>,
    adopt_on_down: bool,
    semantics: Rc<RefCell<gpui::accesskit::Node>>,
}

#[derive(IntoElement)]
struct InspectCanvas {
    canvas: Canvas,
    semantics: Rc<RefCell<gpui::accesskit::Node>>,
}

impl RenderOnce for InspectCanvas {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let element = self.canvas.render(window, cx).into_element();
        let mut node = gpui::accesskit::Node::new(element.a11y_role().unwrap());
        element.write_a11y_info(&mut node);
        *self.semantics.borrow_mut() = node;
        element
    }
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        crate::Root::new("canvas-root").child(
            div().p(px(20.0)).child(InspectCanvas {
                canvas: Canvas::new("canvas", self.scene.clone())
                    .focus_handle(&self.focus)
                    .label("Annotation")
                    .on_pointer(cx.listener(|this, event: &PointerEvent, _, cx| {
                        this.events.push(event.clone());
                        if this.adopt_on_down && event.phase == Phase::Down {
                            this.scene = Arc::new(Scene {
                                transform: Transform {
                                    x: 20.0,
                                    y: 20.0,
                                    scale: 1.0,
                                },
                                children: Vec::new(),
                                ..(*this.scene).clone()
                            });
                        }
                        cx.notify();
                    })),
                semantics: self.semantics.clone(),
            }),
        )
    }
}

fn fixture(
    cx: &mut TestAppContext,
    adopt_on_down: bool,
) -> (gpui::AnyWindowHandle, gpui::Entity<Fixture>) {
    cx.update(crate::init);
    let window = cx.open_window(size(px(500.0), px(400.0)), |_, cx| Fixture {
        scene: Arc::new(Scene {
            transform: Transform {
                x: 10.0,
                y: 10.0,
                scale: 2.0,
            },
            ..(*scene(vec![rect(
                "original",
                Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 30.0,
                    height: 20.0,
                },
                true,
            )]))
            .clone()
        }),
        focus: cx.focus_handle(),
        events: Vec::new(),
        adopt_on_down,
        semantics: Rc::new(RefCell::new(gpui::accesskit::Node::new(Role::Image))),
    });
    let entity = window.root(cx).unwrap();
    let window = window.into();
    frame(cx, window);
    (window, entity)
}

fn frame(cx: &mut TestAppContext, window: gpui::AnyWindowHandle) {
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.refresh();
        window.draw(cx).clear(cx);
    })
    .unwrap();
    cx.run_until_parked();
}

#[gpui::test]
fn painter_order_group_clips_root_transform_and_viewport_share_hit_geometry(
    cx: &mut TestAppContext,
) {
    let (window, _) = fixture(cx, false);
    cx.update_window(window, |_, window, _| {
        let scene = Arc::new(Scene {
            transform: Transform {
                x: 10.0,
                y: 5.0,
                scale: 2.0,
            },
            ..(*scene(vec![
                rect(
                    "bottom",
                    Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 100.0,
                        height: 100.0,
                    },
                    true,
                ),
                Node::Group {
                    transform: Transform {
                        x: 5.0,
                        y: 5.0,
                        scale: 2.0,
                    },
                    clip: Some(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 10.0,
                        height: 10.0,
                    }),
                    children: vec![rect(
                        "top",
                        Rect {
                            x: 0.0,
                            y: 0.0,
                            width: 40.0,
                            height: 40.0,
                        },
                        true,
                    )],
                },
            ]))
            .clone()
        });
        let compiled = Compiled::new(scene, gpui::Font::default(), window);
        assert_eq!(compiled.hit_test(Point { x: 30.0, y: 25.0 }), Some("top"));
        assert_eq!(
            compiled.hit_test(Point { x: 70.0, y: 25.0 }),
            Some("bottom")
        );
        assert_eq!(compiled.hit_test(Point { x: 205.0, y: 30.0 }), None);
        assert_eq!(compiled.hit_test(Point { x: 40.0, y: -1.0 }), None);
    })
    .unwrap();
}

#[gpui::test]
fn text_hits_use_native_shaped_width_and_ascent_descent_not_character_counts(
    cx: &mut TestAppContext,
) {
    let (window, _) = fixture(cx, false);
    cx.update_window(window, |_, window, _| {
        let font = gpui::Font::default();
        let text: gpui::SharedString = "WWW ffi 中文".into();
        let line = window.text_system().shape_line(
            text.clone(),
            px(20.0),
            &[gpui::TextRun {
                len: text.len(),
                font: font.clone(),
                color: gpui::black(),
                ..Default::default()
            }],
            None,
        );
        let width = f32::from(line.width());
        let height = f32::from(line.ascent + line.descent);
        let compiled = Compiled::new(
            Arc::new(Scene {
                width: width + 40.0,
                height: height + 40.0,
                transform: Transform::default(),
                children: vec![Node::Text {
                    id: "text".into(),
                    origin: Point { x: 10.0, y: 10.0 },
                    text,
                    size: 20.0,
                    color: gpui::black(),
                }],
            }),
            font,
            window,
        );
        assert_eq!(
            compiled.hit_test(Point {
                x: 10.0 + width / 2.0,
                y: 10.0 + height / 2.0
            }),
            Some("text")
        );
        assert_eq!(
            compiled.hit_test(Point {
                x: 11.0 + width,
                y: 10.0 + height / 2.0
            }),
            None
        );
        assert_eq!(
            compiled.hit_test(Point {
                x: 11.0,
                y: 11.0 + height
            }),
            None
        );
        assert!(compiled.alternative.contains("WWW ffi 中文"));
    })
    .unwrap();
}

#[gpui::test]
fn window_capture_survives_scene_adoption_and_releases_outside_viewport(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx, true);
    let bounds = VisualTestContext::from_window(window, cx)
        .debug_bounds("canvas")
        .unwrap();
    let down = bounds.origin + point(px(50.0), px(50.0));
    VisualTestContext::from_window(window, cx).simulate_mouse_down(
        down,
        MouseButton::Left,
        Default::default(),
    );
    frame(cx, window);
    let outside = bounds.origin + point(px(300.0), px(200.0));
    VisualTestContext::from_window(window, cx).simulate_mouse_move(
        outside,
        Some(MouseButton::Left),
        Default::default(),
    );
    frame(cx, window);
    VisualTestContext::from_window(window, cx).simulate_mouse_up(
        outside,
        MouseButton::Left,
        Default::default(),
    );
    frame(cx, window);
    entity.read_with(cx, |this, _| {
        assert_eq!(
            this.events
                .iter()
                .map(|event| event.phase)
                .collect::<Vec<_>>(),
            [Phase::Down, Phase::Move, Phase::Up]
        );
        for event in &this.events {
            assert_eq!(event.origin, Point { x: 20.0, y: 20.0 });
            assert_eq!(event.target.as_deref(), Some("original"));
        }
        assert_eq!(this.events[2].position, Point { x: 280.0, y: 180.0 });
    });
}

#[gpui::test]
fn lost_primary_button_cancels_capture_and_canvas_has_focus_and_alternative(
    cx: &mut TestAppContext,
) {
    let (window, entity) = fixture(cx, false);
    let bounds = VisualTestContext::from_window(window, cx)
        .debug_bounds("canvas")
        .unwrap();
    let position = bounds.origin + point(px(50.0), px(50.0));
    VisualTestContext::from_window(window, cx).simulate_mouse_down(
        position,
        MouseButton::Left,
        Default::default(),
    );
    frame(cx, window);
    let focus = entity.read_with(cx, |this, _| this.focus.clone());
    assert!(
        cx.update_window(window, |_, window, _| focus.is_focused(window))
            .unwrap()
    );
    VisualTestContext::from_window(window, cx).simulate_mouse_move(
        position,
        None,
        Default::default(),
    );
    frame(cx, window);
    VisualTestContext::from_window(window, cx).simulate_mouse_up(
        position,
        MouseButton::Left,
        Default::default(),
    );
    frame(cx, window);
    entity.read_with(cx, |this, _| {
        assert_eq!(
            this.events
                .iter()
                .map(|event| event.phase)
                .collect::<Vec<_>>(),
            [Phase::Down, Phase::Cancel]
        );
        assert_eq!(this.semantics.borrow().label(), Some("Annotation"));
        assert!(
            this.semantics
                .borrow()
                .description()
                .unwrap()
                .contains("Rectangle original")
        );
    });
}
