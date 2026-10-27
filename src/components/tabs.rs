use gpui::{
    App, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, StyleRefinement, Styled,
    Window,
};

use super::{Segment, SegmentedControl, TabVariant};

/// Controlled page navigation; shares roving-key behavior with segmented choices.
/// The host owns the selected page and its corresponding content.
#[derive(IntoElement)]
pub struct Tabs(SegmentedControl);

impl Tabs {
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        tabs: impl IntoIterator<Item = Segment>,
        selected: impl Into<SharedString>,
    ) -> Self {
        let mut control = SegmentedControl::new(id, label, tabs, selected);
        control.tab_semantics = true;
        Self(control)
    }

    pub fn variant(mut self, variant: TabVariant) -> Self {
        self.0 = self.0.variant(variant);
        self
    }

    pub fn item_style(mut self, style: StyleRefinement) -> Self {
        self.0 = self.0.item_style(style);
        self
    }

    pub fn selected_style(mut self, style: StyleRefinement) -> Self {
        self.0 = self.0.selected_style(style);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0 = self.0.disabled(disabled);
        self
    }

    pub fn focus_handle(mut self, focus: &FocusHandle) -> Self {
        self.0 = self.0.focus_handle(focus);
        self
    }

    pub fn on_change(
        mut self,
        callback: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0 = self.0.on_change(callback);
        self
    }
}

impl Styled for Tabs {
    fn style(&mut self) -> &mut StyleRefinement {
        self.0.style()
    }
}

impl RenderOnce for Tabs {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.0.render(window, cx)
    }
}
