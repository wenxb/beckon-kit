use gpui::{
    A11ySubtreeBuilder, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Role, Window, accesskit::Node,
};

/// Add states that GPUI's div builders do not expose without a parallel a11y tree.
pub(crate) struct Semantic<E> {
    inner: E,
    disabled: bool,
    busy: bool,
    read_only: bool,
    invalid: bool,
    multiline: bool,
}

impl<E> Semantic<E> {
    pub(crate) fn new(inner: E) -> Self {
        Self {
            inner,
            disabled: false,
            busy: false,
            read_only: false,
            invalid: false,
            multiline: false,
        }
    }

    pub(crate) fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub(crate) fn busy(mut self, value: bool) -> Self {
        self.busy = value;
        self
    }

    pub(crate) fn read_only(mut self, value: bool) -> Self {
        self.read_only = value;
        self
    }

    pub(crate) fn invalid(mut self, value: bool) -> Self {
        self.invalid = value;
        self
    }

    pub(crate) fn multiline(mut self, value: bool) -> Self {
        self.multiline = value;
        self
    }
}

impl<E: Element> IntoElement for Semantic<E> {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl<E: Element> Element for Semantic<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        self.inner.id()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.inner
            .paint(id, inspector, bounds, layout, prepaint, window, cx);
    }

    fn a11y_role(&self) -> Option<Role> {
        if self.multiline {
            Some(Role::MultilineTextInput)
        } else {
            self.inner.a11y_role()
        }
    }

    fn write_a11y_info(&self, node: &mut Node) {
        self.inner.write_a11y_info(node);
        if self.disabled {
            node.set_disabled();
        }
        if self.busy {
            node.set_busy();
        }
        if self.read_only {
            node.set_read_only();
        }
        if self.invalid {
            node.set_invalid(gpui::accesskit::Invalid::True);
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{InteractiveElement, StatefulInteractiveElement, div};

    #[test]
    fn semantic_states_augment_the_existing_accessibility_node() {
        let element = Semantic::new(
            div()
                .id("field")
                .role(Role::TextInput)
                .aria_label("Name")
                .aria_value("value"),
        )
        .disabled(true)
        .busy(true)
        .read_only(true)
        .invalid(true)
        .multiline(true);
        let mut node = Node::new(element.a11y_role().unwrap());
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Name"));
        assert_eq!(node.value(), Some("value"));
        assert!(node.is_disabled() && node.is_busy() && node.is_read_only());
        assert_eq!(node.invalid(), Some(gpui::accesskit::Invalid::True));
        assert_eq!(node.role(), Role::MultilineTextInput);
    }
}
