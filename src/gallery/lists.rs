use gpui::{Context, IntoElement, ScrollStrategy, div, prelude::*, px};

use super::Gallery;
use crate::{
    components::{Button, ButtonVariant, Surface, VirtualList},
    theme::Theme,
};

impl Gallery {
    pub(super) fn virtual_lists(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Theme::new(self.appearance).colors;
        let entity = cx.entity();
        Surface::new("virtual-list-specimen")
            .shadow(Vec::new())
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child("10,000 行 · 只构建可见行 · 固定 36px 行高")
            .child("默认高度上限 320px；本例通过 Styled 覆盖为 216px，滚动句柄跨帧保留。")
            .child(
                Button::new("gallery-list-jump", "跳到第 5,001 行").on_click(cx.listener(
                    |gallery, _, _, cx| {
                        gallery
                            .list_scroll
                            .scroll_to_item_strict(5000, ScrollStrategy::Top);
                        cx.notify();
                    },
                )),
            )
            .child(
                VirtualList::new("gallery-virtual-list", 10_000, move |range, _, cx| {
                    entity.update(cx, |_, cx| {
                        range
                            .map(|index| {
                                Button::new(
                                    format!("gallery-list-row-{index}"),
                                    format!("第 {} 行 · 固定高度、完整可访问名称", index + 1),
                                )
                                .variant(ButtonVariant::Quiet)
                                .w_full()
                                .h(px(36.0))
                                .rounded(px(4.0))
                                .on_click(cx.listener(
                                    move |gallery, _, _, cx| {
                                        gallery.last_event =
                                            format!("已选择第 {} 行", index + 1).into();
                                        cx.notify();
                                    },
                                ))
                            })
                            .collect::<Vec<_>>()
                    })
                })
                .label("一万行虚拟列表")
                .track_scroll(&self.list_scroll)
                .h(px(216.0))
                .border_1()
                .border_color(colors.border)
                .rounded(px(6.0)),
            )
            .child(
                div()
                    .text_color(colors.secondary)
                    .child(self.last_event.clone()),
            )
    }
}
