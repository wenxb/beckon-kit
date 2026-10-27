use gpui::Refineable;
use gpui::{
    App, ElementId, IntoElement, ParentElement, RenderOnce, Role, SharedString, StyleRefinement,
    Styled, Window, div, prelude::*, px, relative,
};

use crate::theme::Theme;

#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    label: SharedString,
    fraction: f32,
    style: StyleRefinement,
}

impl ProgressBar {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, fraction: f32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            fraction: if fraction.is_nan() {
                0.0
            } else {
                fraction.clamp(0.0, 1.0)
            },
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for ProgressBar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = Theme::current(cx);
        let selector = self.id.to_string();
        let mut element = div()
            .id(self.id)
            .role(Role::ProgressIndicator)
            .aria_label(self.label)
            .aria_numeric_value(f64::from(self.fraction))
            .aria_min_numeric_value(0.0)
            .aria_max_numeric_value(1.0)
            .debug_selector(move || selector)
            .w_full()
            .h(px(4.0))
            .rounded_full()
            .bg(theme.colors.border)
            .child(
                div()
                    .h_full()
                    .w(relative(self.fraction))
                    .rounded_full()
                    .bg(theme.colors.info),
            );
        element.style().refine(&self.style);
        element
    }
}
