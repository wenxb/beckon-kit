use std::sync::Arc;

use gpui::{Context, IntoElement, Window, div, prelude::*, px};

use super::Gallery;
use crate::{
    components::{
        Canvas, Surface,
        canvas::{Node, Point, Rect, Scene, Stroke, Transform},
    },
    theme::Theme,
};

impl Gallery {
    pub(super) fn canvas(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::new(self.appearance).colors;
        let appearance = self.appearance;
        let cached = window.use_keyed_state("gallery-canvas-scene", cx, |_, _| {
            (appearance, specimen(&colors))
        });
        if cached.read(cx).0 != appearance {
            cached.update(cx, |cached, _| *cached = (appearance, specimen(&colors)));
        }
        let scene = cached.read(cx).1.clone();
        Surface::new("canvas-specimen")
            .shadow(Vec::new())
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child("矩形、箭头与原生文字 · 组变换与矩形裁剪")
            .child("点击或拖动图形，查看场景坐标与捕获目标；文字编辑由外部 TextField 承担。")
            .child(
                Canvas::new("gallery-canvas", scene)
                    .label("画布组件示例")
                    .border_1()
                    .border_color(colors.border)
                    .on_pointer(cx.listener(
                        |gallery, event: &crate::components::canvas::PointerEvent, _, cx| {
                            gallery.last_event = format!(
                                "{:?} · ({:.1}, {:.1}) · {}",
                                event.phase,
                                event.position.x,
                                event.position.y,
                                event.target.as_deref().unwrap_or("空白")
                            )
                            .into();
                            cx.notify();
                        },
                    )),
            )
            .child(
                div()
                    .text_color(colors.secondary)
                    .child(self.last_event.clone()),
            )
    }
}

fn specimen(colors: &crate::theme::Colors) -> Arc<Scene> {
    Arc::new(Scene {
        width: 440.0,
        height: 240.0,
        transform: Transform {
            x: 16.0,
            y: 16.0,
            scale: 1.0,
        },
        children: vec![Node::Group {
            transform: Transform {
                x: 8.0,
                y: 8.0,
                scale: 1.2,
            },
            clip: Some(Rect {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 160.0,
            }),
            children: vec![
                Node::Rect {
                    id: "rectangle".into(),
                    rect: Rect {
                        x: 10.0,
                        y: 15.0,
                        width: 100.0,
                        height: 55.0,
                    },
                    fill: Some(colors.info.opacity(0.2)),
                    stroke: Some(Stroke {
                        color: colors.info,
                        width: 2.0,
                    }),
                },
                Node::Arrow {
                    id: "arrow".into(),
                    from: Point { x: 120.0, y: 45.0 },
                    to: Point { x: 225.0, y: 90.0 },
                    stroke: Stroke {
                        color: colors.success,
                        width: 3.0,
                    },
                },
                Node::Text {
                    id: "text".into(),
                    origin: Point { x: 12.0, y: 105.0 },
                    text: "中文标注 · Native text".into(),
                    size: 18.0,
                    color: colors.text,
                },
            ],
        }],
    })
}
