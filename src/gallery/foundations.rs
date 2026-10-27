use gpui::{IntoElement, div, prelude::*, px};

use super::Gallery;
use crate::{
    components::{Icon, IconName},
    theme::Theme,
};

impl Gallery {
    pub(super) fn colors(&self) -> impl IntoElement {
        let colors = Theme::new(self.appearance).colors;
        let palette = [
            ("Primary · 主要操作", colors.primary),
            ("Info · 选择 / 激活", colors.info),
            ("Success · 开启 / 成功", colors.success),
            ("Warning · 需注意", colors.warning),
            ("Danger · 错误 / 危险", colors.danger),
            ("Canvas · 底层", colors.canvas),
            ("Surface · 内容", colors.surface),
            ("Elevated · 控件", colors.elevated),
        ];
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(div().text_color(colors.secondary).child(
                "Primary 是中性墨色；深色模式反转明暗以保持对比度。选中与焦点默认使用 Info，不绑定品牌色。",
            ))
            .children(palette.into_iter().map(|(name, color)| {
                let rgb = color.to_rgb();
                let hex = format!(
                    "#{:02X}{:02X}{:02X}",
                    (rgb.r * 255.0).round() as u8,
                    (rgb.g * 255.0).round() as u8,
                    (rgb.b * 255.0).round() as u8,
                );
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .py(px(8.0))
                    .child(
                        div()
                            .size(px(36.0))
                            .flex_shrink_0()
                            .rounded(px(6.0))
                            .bg(color)
                            .border(px(1.0))
                            .border_color(colors.border),
                    )
                    .child(div().flex_1().child(name))
                    .child(div().text_color(colors.secondary).child(hex))
            }))
    }

    pub(super) fn icons(&self) -> impl IntoElement {
        let colors = Theme::new(self.appearance).colors;
        div()
            .flex()
            .flex_col()
            .gap(px(20.0))
            .child(
                div()
                    .text_color(colors.secondary)
                    .child("默认 16px，继承文字颜色。图标不自带交互，操作语义由按钮或导航项提供。"),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(12.0))
                    .children(IconName::ALL.iter().map(|name| {
                        div()
                            .w(px(140.0))
                            .min_h(px(82.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(10.0))
                            .rounded(px(6.0))
                            .border(px(1.0))
                            .border_color(colors.border)
                            .child(Icon::new(*name).size(px(20.0)))
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(colors.secondary)
                                    .child(name.as_str()),
                            )
                    })),
            )
            .child(
                div().text_color(colors.secondary).child(
                    "Icon::new(IconName::Search) · Icon::data(bytes) · Icon::path(asset_path)",
                ),
            )
    }
}
