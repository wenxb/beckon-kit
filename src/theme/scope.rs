use gpui::{
    AnyElement, App, Bounds, Element, ElementId, GlobalElementId, InspectorElementId, IntoElement,
    LayoutId, Pixels, Window,
};
use std::sync::Arc;

use super::{Theme, with_theme};

/// Transparent layout wrapper; siblings and other windows do not inherit it.
pub struct ThemeScope {
    theme: Arc<Theme>,
    child: AnyElement,
}

impl ThemeScope {
    pub fn new(theme: impl Into<Arc<Theme>>, child: impl IntoElement) -> Self {
        Self {
            theme: theme.into(),
            child: child.into_any_element(),
        }
    }
}

impl IntoElement for ThemeScope {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

impl Element for ThemeScope {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let text = self.theme.text_style();
        let layout = with_theme(self.theme.clone(), cx, |cx| {
            window.with_text_style(Some(text), |window| self.child.request_layout(window, cx))
        });
        (layout, ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let text = self.theme.text_style();
        with_theme(self.theme.clone(), cx, |cx| {
            window.with_text_style(Some(text), |window| self.child.prepaint(window, cx))
        });
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        let text = self.theme.text_style();
        with_theme(self.theme.clone(), cx, |cx| {
            window.with_text_style(Some(text), |window| self.child.paint(window, cx))
        });
    }
}
