use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};

use super::{Gallery, Page};
use crate::{
    Root,
    components::{Button, ButtonVariant, ControlSize, Icon, IconName},
    theme::{Appearance, Density, Theme, ThemeScope},
};

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = Theme::new(self.appearance)
            .density(self.density)
            .preferences(self.preferences);
        let colors = &theme.colors;
        let narrow = window.viewport_size().width < px(700.0);
        let sidebar = self.sidebar(narrow, window, cx);
        let header = div()
            .id("gallery-header")
            .debug_selector(|| "gallery-header".into())
            .flex()
            .items_center()
            .justify_between()
            .flex_wrap()
            .gap(px(8.0))
            .min_h(px(56.0))
            .px(px(if narrow { 12.0 } else { 28.0 }))
            .py(px(8.0))
            .flex_shrink_0()
            .border_b(px(1.0))
            .border_color(colors.border)
            .child(div().text_color(colors.secondary).child("组件文档"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .flex_wrap()
                    .child(
                        Button::icon(
                            "gallery-density",
                            "紧凑布局",
                            Icon::new(IconName::Settings2),
                        )
                        .size(ControlSize::Small)
                        .toggled(self.density == Density::Compact)
                        .variant(if self.density == Density::Compact {
                            ButtonVariant::Soft
                        } else {
                            ButtonVariant::Quiet
                        })
                        .on_click(cx.listener(|gallery, _, _, cx| {
                            gallery.set_density(
                                if gallery.density == Density::Compact {
                                    Density::Comfortable
                                } else {
                                    Density::Compact
                                },
                                cx,
                            );
                        })),
                    )
                    .child(
                        Button::icon(
                            "gallery-theme",
                            if self.appearance == Appearance::Dark {
                                "切换到浅色模式"
                            } else {
                                "切换到深色模式"
                            },
                            Icon::new(if self.appearance == Appearance::Dark {
                                IconName::Sun
                            } else {
                                IconName::Moon
                            }),
                        )
                        .size(ControlSize::Small)
                        .variant(ButtonVariant::Quiet)
                        .on_click(cx.listener(|gallery, _, _, cx| {
                            gallery.appearance = if gallery.appearance == Appearance::Dark {
                                Appearance::Light
                            } else {
                                Appearance::Dark
                            };
                            cx.notify();
                        })),
                    ),
            );
        let content = match self.page {
            Page::Controls => self.controls(cx).into_any_element(),
            Page::VirtualList => self.virtual_lists(cx).into_any_element(),
            Page::Canvas => self.canvas(window, cx).into_any_element(),
            Page::Materials => self.materials(cx).into_any_element(),
            Page::Window => self.window_settings(window, cx).into_any_element(),
            Page::Colors => self.colors().into_any_element(),
            Page::Icons => self.icons().into_any_element(),
        };
        ThemeScope::new(
            theme.clone(),
            Root::new("gallery-root")
                .label("Beckon Kit 组件文档")
                .child(
                    div()
                        .size_full()
                        .flex()
                        .overflow_hidden()
                        .bg(colors.surface)
                        .child(sidebar)
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .h_full()
                                .flex()
                                .flex_col()
                                .child(header)
                                .child(
                                    div()
                                        .id("gallery-scroll")
                                        .debug_selector(|| "gallery-scroll".into())
                                        .track_scroll(&self.scroll)
                                        .flex_1()
                                        .min_h_0()
                                        .min_w_0()
                                        .overflow_y_scroll()
                                        .p(px(if narrow { 12.0 } else { 28.0 }))
                                        .child(
                                            div()
                                                .w_full()
                                                .max_w(px(1040.0))
                                                .mx_auto()
                                                .flex()
                                                .flex_col()
                                                .gap(px(24.0))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(6.0))
                                                        .child(
                                                            div()
                                                                .text_size(px(24.0))
                                                                .font_weight(
                                                                    gpui::FontWeight::SEMIBOLD,
                                                                )
                                                                .child(self.page.name()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_color(colors.secondary)
                                                                .child(self.page.description()),
                                                        ),
                                                )
                                                .child(content),
                                        ),
                                ),
                        ),
                ),
        )
    }
}
