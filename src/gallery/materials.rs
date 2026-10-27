use gpui::{Context, IntoElement, div, prelude::*, px};

use super::Gallery;
use crate::{components::*, theme::Theme};

impl Gallery {
    pub(super) fn materials(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::new(self.appearance);
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(16.0))
                    .child(
                        Toggle::new(
                            "gallery-reduce-transparency",
                            "降低透明度",
                            self.preferences.reduced_transparency,
                        )
                        .on_change(cx.listener(
                            |gallery, enabled: &bool, _, cx| {
                                gallery.preferences.reduced_transparency = *enabled;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Toggle::new(
                            "gallery-contrast",
                            "提高对比度",
                            self.preferences.increased_contrast,
                        )
                        .on_change(cx.listener(
                            |gallery, enabled: &bool, _, cx| {
                                gallery.preferences.increased_contrast = *enabled;
                                cx.notify();
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(16.0))
                    .child(
                        Button::new("gallery-scene-change", "更换玻璃后方内容").on_click(
                            cx.listener(|gallery, _, _, cx| {
                                gallery.advance_scene(cx);
                            }),
                        ),
                    )
                    .child(
                        Slider::new(
                            "gallery-scene-shift",
                            "背景位置",
                            SliderScale::new(0.0, 1.0, 0.01).unwrap(),
                            self.scene_shift,
                        )
                        .w(px(200.0))
                        .on_change(cx.listener(
                            |gallery, value: &f64, _, cx| {
                                gallery.set_scene_shift(*value, cx);
                            },
                        )),
                    ),
            )
            .child(
                div()
                    .relative()
                    .overflow_hidden()
                    .rounded(px(14.0))
                    .w_full()
                    .min_h(px(430.0))
                    .child(self.live_scene())
                    .child(
                        Button::icon("gallery-glass-close", "关闭示例", Icon::new(IconName::X))
                            .tooltip(false)
                            .frosted()
                            .absolute()
                            .top(px(22.0))
                            .right(px(22.0))
                            .rounded_full()
                            .on_click(cx.listener(|gallery, _, _, cx| {
                                gallery.last_event = "已点击磨砂按钮".into();
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .relative()
                            .min_h(px(430.0))
                            .p(px(22.0))
                            .pt(px(105.0))
                            .flex()
                            .items_center()
                            .flex_wrap()
                            .gap(px(18.0))
                            .children(
                                [("card", "卡片"), ("details", "信息"), ("actions", "操作")]
                                    .into_iter()
                                    .map(|(key, title)| {
                                        Surface::new(format!("material-{key}"))
                                            .frosted()
                                            .flex_1()
                                            .min_w(px(180.0))
                                            .min_h(px(185.0))
                                            .child(Badge::new(title))
                                            .child(
                                                div()
                                                    .text_size(px(19.0))
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .child("柔和的边界。"),
                                            )
                                            .child(
                                                div()
                                                    .text_color(theme.colors.secondary)
                                                    .child("背景实时模糊，前景依然清晰。"),
                                            )
                                            .child(
                                                Button::new(
                                                    format!("material-action-{key}"),
                                                    "查看",
                                                )
                                                .variant(ButtonVariant::Secondary)
                                                .on_click(cx.listener(|gallery, _, _, cx| {
                                                    gallery.last_event = "已查看材质".into();
                                                    cx.notify();
                                                })),
                                            )
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(theme.colors.secondary)
                    .child("所有毛玻璃组件共用主题材质 · 实时背景模糊 · 不支持时显示实心材质"),
            )
    }
}
