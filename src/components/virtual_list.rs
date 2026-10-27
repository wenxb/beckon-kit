use std::ops::Range;

use gpui::{
    App, ElementId, IntoElement, ListSizingBehavior, Role, SharedString, StyleRefinement, Styled,
    UniformList, UniformListScrollHandle, Window, prelude::*, px, uniform_list,
};

/// Lazy, fixed-height rows backed by GPUI's uniform list.
///
/// The renderer receives only the measured row and visible ranges, not the entire collection.
/// Every row must have the same height. Keep the scroll handle in the owning view and reuse
/// the element id across renders. The default viewport is at most 320px tall; `Styled`
/// overrides apply directly to the list, including explicit height, padding and colors.
/// Row content, activation and accessible names remain the caller's responsibility.
pub struct VirtualList {
    inner: UniformList,
}

impl VirtualList {
    pub fn new<R: IntoElement>(
        id: impl Into<ElementId>,
        item_count: usize,
        render: impl Fn(Range<usize>, &mut Window, &mut App) -> Vec<R> + 'static,
    ) -> Self {
        Self {
            inner: uniform_list(id, item_count, render)
                .role(Role::List)
                .with_sizing_behavior(ListSizingBehavior::Infer)
                .w_full()
                .max_h(px(320.0))
                .overflow_x_hidden(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.inner = self.inner.aria_label(label);
        self
    }

    /// Bind persistent GPUI scroll state, including programmatic `scroll_to_item` requests.
    pub fn track_scroll(mut self, handle: &UniformListScrollHandle) -> Self {
        self.inner = self.inner.track_scroll(handle);
        self
    }
}

impl Styled for VirtualList {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl IntoElement for VirtualList {
    type Element = UniformList;

    fn into_element(self) -> Self::Element {
        self.inner
    }
}

#[cfg(all(test, feature = "test-support"))]
#[path = "virtual_list_tests.rs"]
mod tests;
