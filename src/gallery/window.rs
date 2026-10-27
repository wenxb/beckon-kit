use super::Gallery;
use crate::{
    WindowRoot,
    components::{Button, Slider},
    window::{FrameOwner, WindowFrameStatus},
};
use gpui::{Context, IntoElement, Window, div, prelude::*, px};

impl Gallery {
    pub(super) fn window_settings(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let root = window.root::<WindowRoot<Self>>().flatten();
        let status = root
            .as_ref()
            .map(|root| root.read(cx).frame_status().clone());
        let radius = root.as_ref().map_or(0.0, |root| {
            f32::from(root.read(cx).client_style().corner_radii.top_left)
        });
        let mut content = div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child("窗口 · 请求与实际策略");
        if let Some(status) = status {
            content = content.child(status_text(&status)).child(format!(
                "标题区：{:?}",
                root.as_ref().unwrap().read(cx).titlebar_mode()
            ));
            content = content.child(
                Slider::new(
                    "window-radius",
                    "客户端圆角",
                    crate::components::SliderScale::default(),
                    f64::from(radius / 32.0),
                )
                .on_change(cx.listener(move |_, value: &f64, window, cx| {
                    let root = root.clone();
                    let radius = px((*value * 32.0) as f32);
                    cx.defer_in(window, move |_, window, cx| {
                        if let Some(root) = root {
                            root.update(cx, |root, cx| {
                                if let Err(error) =
                                    root.set_client_style(|s| s.radius(radius), window, cx)
                                {
                                    eprintln!("{error}");
                                }
                            });
                        }
                    });
                })),
            );
            content = content.child(Button::new("window-shadow", "切换客户端投影").on_click(
                cx.listener(|_, _, window, cx| {
                    let root = window.root::<WindowRoot<Self>>().flatten();
                    cx.defer_in(window, move |_, window, cx| {
                        if let Some(root) = root {
                            root.update(cx, |root, cx| {
                                let shadows = if root.client_style().shadows.is_empty() {
                                    vec![gpui::BoxShadow::new(
                                        px(0.0),
                                        px(8.0),
                                        gpui::black().opacity(0.3),
                                    )]
                                } else {
                                    Vec::new()
                                };
                                if let Err(error) =
                                    root.set_client_style(|s| s.shadows(shadows), window, cx)
                                {
                                    eprintln!("{error}");
                                }
                            });
                        }
                    });
                }),
            ));
        } else {
            content = content.child("当前测试 / 嵌入环境没有 WindowRoot");
        }
        content.child("圆角和投影仅影响客户端外框；系统外框的半径与阴影由系统管理。标题区归属为创建属性，不提供伪动态切换。")
    }
}
fn status_text(status: &WindowFrameStatus) -> String {
    format!(
        "请求：{:?} · 实际：{:?} / {:?}\n可见客户区：{:.0} × {:.0} · DPI：{:.2}\n圆角 / 阴影归属：{}\n{}",
        status.requested,
        status.owner,
        status.state,
        f32::from(status.geometry.visible.size.width),
        f32::from(status.geometry.visible.size.height),
        status.scale_factor,
        if status.owner == FrameOwner::System {
            "系统管理（不报告虚构半径）"
        } else {
            "客户端应用"
        },
        status.reason.as_deref().unwrap_or("无降级")
    )
}
