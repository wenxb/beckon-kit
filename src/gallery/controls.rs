use gpui::{Context, IntoElement, div, prelude::*, px};

use super::Gallery;
use crate::{components::*, theme::Theme};

impl Gallery {
    pub(super) fn controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::new(self.appearance);
        let group = |title, subtitle| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(15.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(theme.colors.secondary)
                        .child(subtitle),
                )
        };
        div()
            .flex()
            .flex_wrap()
            .items_start()
            .gap(px(18.0))
            .child(
                Surface::new("action-specimen")
                    .shadow(Vec::new())
                    .flex_1()
                    .min_w(px(260.0))
                    .child(group("操作", "实心、柔和、描边与文字，明确不同层级。"))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.0))
                            .child(
                                Button::new("gallery-primary", "执行命令")
                                    .variant(ButtonVariant::Primary)
                                    .leading(Icon::new(IconName::Plus))
                                    .on_click(cx.listener(|gallery, _, _, cx| {
                                        gallery.clicks += 1;
                                        gallery.last_event =
                                            format!("已执行 {} 次", gallery.clicks).into();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("gallery-secondary", "复制").on_click(cx.listener(
                                    |gallery, _, _, cx| {
                                        gallery.last_event = "已请求复制".into();
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                Button::new("gallery-quiet", "更多")
                                    .variant(ButtonVariant::Quiet)
                                    .on_click(cx.listener(|gallery, _, _, cx| {
                                        gallery.last_event = "已请求更多操作".into();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div().flex().flex_wrap().gap(px(8.0)).children(
                            [
                                ("gallery-soft", "柔和", ButtonVariant::Soft),
                                ("gallery-outline", "描边", ButtonVariant::Outline),
                                ("gallery-link", "了解详情", ButtonVariant::Link),
                            ]
                            .into_iter()
                            .map(|(id, label, variant)| {
                                Button::new(id, label)
                                    .variant(variant)
                                    .on_click(cx.listener(move |gallery, _, _, cx| {
                                        gallery.last_event = format!("已点击{label}").into();
                                        cx.notify();
                                    }))
                            }),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(8.0))
                            .child(
                                Button::new("gallery-danger", "移除")
                                    .variant(ButtonVariant::Danger)
                                    .on_click(cx.listener(|gallery, _, _, cx| {
                                        gallery.last_event = "已请求移除".into();
                                        cx.notify();
                                    })),
                            )
                            .child(Button::new("gallery-disabled", "不可用").disabled(true))
                            .child(Button::new("gallery-busy", "处理中").busy(true)),
                    )
                    .child(
                        div()
                            .pt(px(10.0))
                            .text_color(theme.colors.secondary)
                            .child("命令"),
                    )
                    .child(self.input.clone())
                    .child(group("多行说明", "Enter 换行，Ctrl/Cmd+Enter 提交；长文自动折行并滚动。"))
                    .child(self.textarea.clone())
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(theme.colors.secondary)
                            .child(self.last_event.clone()),
                    )
                    .child(ProgressBar::new(
                        "gallery-progress",
                        "命令进度",
                        self.progress as f32,
                    ))
                    .child(Markdown::new(
                        "gallery-markdown",
                        "**只读内容**支持 *强调*、`代码` 和 [链接](https://example.test)。\n\n- 链接默认不执行\n- 图片只展示替代文字",
                    )),
            )
            .child(
                Surface::new("selection-specimen")
                    .shadow(Vec::new())
                    .flex_1()
                    .min_w(px(260.0))
                    .child(group("选择", "开关使用绿色，勾选与激活使用蓝色。"))
                    .child(
                        Toggle::new("gallery-toggle", "显示最近命令", self.on).on_change(
                            cx.listener(|gallery, on: &bool, _, cx| {
                                gallery.on = *on;
                                gallery.last_event = if *on {
                                    "最近命令：已显示"
                                } else {
                                    "最近命令：已隐藏"
                                }
                                .into();
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        Checkbox::new("gallery-check", "保持面板打开", self.checks).on_change(
                            cx.listener(|gallery, state: &CheckState, _, cx| {
                                gallery.checks = *state;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(
                        Checkbox::new("gallery-mixed", "已选择部分来源", CheckState::Mixed)
                            .disabled(true),
                    )
                    .child(
                        SegmentedControl::new(
                            "gallery-segments",
                            "结果布局",
                            [
                                Segment::new("list", "列表"),
                                Segment::new("grid", "网格"),
                                Segment::new("detail", "详情"),
                            ],
                            self.mode.clone(),
                        )
                        .on_change(cx.listener(
                            |gallery, mode: &gpui::SharedString, _, cx| {
                                gallery.mode = mode.clone();
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Select::new(
                            "gallery-select",
                            "结果布局",
                            [
                                SelectOption::new("list", "列表"),
                                SelectOption::new("grid", "网格"),
                                SelectOption::new("detail", "详情"),
                            ],
                            Some(self.mode.clone()),
                        )
                        .on_change(cx.listener(|gallery, mode: &gpui::SharedString, _, cx| {
                            gallery.mode = mode.clone();
                            cx.notify();
                        })),
                    )
                    .children(
                        [
                            ("gallery-pills", TabVariant::Pills),
                            ("gallery-underline", TabVariant::Underline),
                        ]
                        .into_iter()
                        .map(|(id, variant)| {
                            Tabs::new(
                                id,
                                "标签变体",
                                [
                                    Segment::new("list", "列表"),
                                    Segment::new("grid", "网格"),
                                    Segment::new("detail", "详情"),
                                ],
                                self.mode.clone(),
                            )
                            .variant(variant)
                            .on_change(cx.listener(
                                |gallery, mode: &gpui::SharedString, _, cx| {
                                    gallery.mode = mode.clone();
                                    cx.notify();
                                },
                            ))
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .text_color(theme.colors.secondary)
                            .child("强度")
                            .child(format!("{:.0}%", self.progress * 100.0)),
                    )
                    .child(
                        Slider::new(
                            "gallery-slider",
                            "强度",
                            SliderScale::default(),
                            self.progress,
                        )
                        .on_change(cx.listener(
                            |gallery, value: &f64, _, cx| {
                                gallery.progress = *value;
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(6.0))
                            .child(Badge::new("提示").status(Status::Info))
                            .child(Badge::new("就绪").status(Status::Success))
                            .child(Badge::new("待处理").status(Status::Warning))
                            .child(Badge::new("失败").status(Status::Danger)),
                    ),
            )
    }
}
