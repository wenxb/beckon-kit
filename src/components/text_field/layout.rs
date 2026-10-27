use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, AvailableSpace, Bounds, Canvas, Element, ElementId, GlobalElementId, InspectorElementId,
    IntoElement, LayoutId, Pixels, Refineable, SharedString, Style, StyleRefinement, Styled,
    TextRun, Window, canvas, px, size,
};

use super::{display::DisplayText, geometry::FieldGeometry};

pub(super) struct FieldLayout {
    display: DisplayText,
    placeholder: SharedString,
    runs: Vec<TextRun>,
    placeholder_run: TextRun,
    font_size: Pixels,
    line_height: Pixels,
    multiline: bool,
    shaped: Option<ShapedField>,
}

struct ShapedField {
    wrap_width: Option<Pixels>,
    geometry: FieldGeometry,
    placeholder: Option<FieldGeometry>,
}

impl FieldLayout {
    pub fn new(
        display: DisplayText,
        placeholder: SharedString,
        runs: Vec<TextRun>,
        placeholder_run: TextRun,
        font_size: Pixels,
        line_height: Pixels,
        multiline: bool,
    ) -> Self {
        Self {
            display,
            placeholder,
            runs,
            placeholder_run,
            font_size,
            line_height,
            multiline,
            shaped: None,
        }
    }

    fn shape(&mut self, width: Option<Pixels>, window: &mut Window) -> &ShapedField {
        let wrap_width = self
            .multiline
            .then_some(width)
            .flatten()
            .map(|width| (width - px(2.0)).max(px(1.0)));
        if self
            .shaped
            .as_ref()
            .is_none_or(|shaped| shaped.wrap_width != wrap_width)
        {
            let geometry = |display: DisplayText, runs: &[TextRun]| {
                let lines = window
                    .text_system()
                    .shape_text(display.text.clone(), self.font_size, runs, wrap_width, None)
                    .expect("shape field text");
                FieldGeometry::new(Bounds::default(), lines, display, self.line_height)
            };
            self.shaped = Some(ShapedField {
                wrap_width,
                geometry: geometry(self.display.clone(), &self.runs),
                placeholder: (self.display.text.is_empty() && !self.placeholder.is_empty()).then(
                    || {
                        geometry(
                            DisplayText::new(&self.placeholder, false),
                            std::slice::from_ref(&self.placeholder_run),
                        )
                    },
                ),
            });
        }
        self.shaped.as_ref().unwrap()
    }
}

/// Measures and paints the same wrapped geometry, without a second layout frame.
pub(super) struct FieldCanvas<T> {
    canvas: Canvas<T>,
    layout: Rc<RefCell<FieldLayout>>,
}

impl<T: 'static> FieldCanvas<T> {
    pub fn new(
        layout: FieldLayout,
        prepaint: impl FnOnce(
            Bounds<Pixels>,
            FieldGeometry,
            Option<FieldGeometry>,
            &mut Window,
            &mut App,
        ) -> T
        + 'static,
        paint: impl FnOnce(Bounds<Pixels>, T, &mut Window, &mut App) + 'static,
    ) -> Self {
        let layout = Rc::new(RefCell::new(layout));
        let paint_layout = layout.clone();
        Self {
            canvas: canvas(
                move |bounds, window, cx| {
                    let mut layout = paint_layout.borrow_mut();
                    layout.shape(Some(bounds.size.width), window);
                    let mut shaped = layout.shaped.take().unwrap();
                    shaped.geometry.bounds = bounds;
                    shaped.geometry.origin = bounds.origin;
                    prepaint(bounds, shaped.geometry, shaped.placeholder, window, cx)
                },
                paint,
            ),
            layout,
        }
    }
}

impl<T: 'static> IntoElement for FieldCanvas<T> {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl<T> Styled for FieldCanvas<T> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.canvas.style()
    }
}

impl<T: 'static> Element for FieldCanvas<T> {
    type RequestLayoutState = Style;
    type PrepaintState = Option<T>;

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
        _: &mut App,
    ) -> (LayoutId, Style) {
        let mut style = Style::default();
        style.refine(self.canvas.style());
        let layout = self.layout.clone();
        let id =
            window.request_measured_layout(style.clone(), move |known, available, window, _| {
                let width = known.width.or(match available.width {
                    AvailableSpace::Definite(width) => Some(width),
                    _ => None,
                });
                let mut layout = layout.borrow_mut();
                let shaped = layout.shape(width, window);
                let geometry = shaped.placeholder.as_ref().unwrap_or(&shaped.geometry);
                size(geometry.width() + px(2.0), geometry.height())
            });
        (id, style)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Style,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<T> {
        self.canvas
            .prepaint(id, inspector, bounds, layout, window, cx)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        layout: &mut Style,
        prepaint: &mut Option<T>,
        window: &mut Window,
        cx: &mut App,
    ) {
        self.canvas
            .paint(id, inspector, bounds, layout, prepaint, window, cx);
    }
}
