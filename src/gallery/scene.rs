use gpui::{IntoElement, div, prelude::*, px};

use super::Gallery;
use crate::{
    components::{Badge, Status},
    theme::Theme,
};

impl Gallery {
    pub(super) fn live_scene(&self) -> impl IntoElement {
        let theme = Theme::new(self.appearance);
        let revision = self.scene_revision;
        let palette = if revision.is_multiple_of(2) {
            [0x44797a, 0xa35e3b, 0x796491]
        } else {
            [0x596fab, 0x55835b, 0x9d4767]
        };
        // Real GPUI text, badges and quads, not a prepared raster asset.
        div().absolute().inset_0().bg(theme.colors.canvas).child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .top(px(-(self.scene_shift as f32) * 120.0))
                .flex()
                .flex_col()
                .gap(px(8.0))
                .p(px(12.0))
                .children((0..16).map(move |index| {
                    div()
                        .id(format!("live-scene-row-{index}"))
                        .debug_selector(move || format!("live-scene-row-{index}"))
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(12.0))
                        .h(px(52.0))
                        .flex_shrink_0()
                        .px(px(14.0))
                        .rounded(px(6.0))
                        .bg(gpui::rgb(palette[index % palette.len()]))
                        .text_color(gpui::rgb(0xffffff))
                        .child(format!("背景 / {:02} · 第 {revision} 次更新", index + 1))
                        .child(
                            Badge::new(if (index + revision).is_multiple_of(2) {
                                "已更新"
                            } else {
                                "就绪"
                            })
                            .status(Status::Neutral),
                        )
                        .child(
                            div()
                                .w(px(64.0))
                                .h(px(4.0))
                                .bg(gpui::rgb(0xffffff).opacity(0.65)),
                        )
                })),
        )
    }
}
