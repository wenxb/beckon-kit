use gpui::{App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div, px};

use crate::theme::Theme;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(IntoElement)]
pub struct Badge {
    label: SharedString,
    status: Status,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            status: Status::default(),
        }
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }
}

impl RenderOnce for Badge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let color = match self.status {
            Status::Neutral => theme.colors.secondary,
            Status::Info => theme.colors.info,
            Status::Success => theme.colors.success,
            Status::Warning => theme.colors.warning,
            Status::Danger => theme.colors.danger,
        };
        div()
            .flex()
            .items_center()
            .gap(px(5.0))
            .px(px(7.0))
            .py(px(3.0))
            .rounded(px(4.0))
            .bg(color.opacity(0.10))
            .text_color(color)
            .text_size(px(theme.typography.caption))
            .whitespace_nowrap()
            .child(div().size(px(4.0)).rounded_full().bg(color))
            .child(self.label)
    }
}
