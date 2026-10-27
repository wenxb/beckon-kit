use super::{WindowFrameStatus, WindowRootConfig, close::CloseGate, platform::Platform};
use crate::components::{Button, ButtonVariant, ControlSize, Icon, IconName};
use gpui::{
    App, ElementId, IntoElement, MouseButton, ParentElement, RenderOnce, SharedString, Styled,
    Window, WindowControlArea, div, prelude::*, px,
};
use std::{cell::RefCell, rc::Rc};

/// Explicit non-interactive window-move space. Put controls beside it, not inside it.
#[derive(IntoElement)]
pub struct DragRegion {
    id: ElementId,
    label: SharedString,
    style: gpui::StyleRefinement,
}
impl DragRegion {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: "移动窗口".into(),
            style: Default::default(),
        }
    }
}
impl Styled for DragRegion {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}
impl RenderOnce for DragRegion {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        use gpui::Refineable;
        let selector = self.id.to_string();
        let mut element = div()
            .id(self.id)
            .aria_label(self.label)
            .debug_selector(move || selector)
            .flex_1()
            .h_full();
        element.style().refine(&self.style);
        if Platform::current() == Platform::Windows {
            element = element.window_control_area(WindowControlArea::Drag);
        } else {
            element = element.on_mouse_down(MouseButton::Left, |event, window, cx| {
                if event.click_count == 2 {
                    if Platform::current() == Platform::Mac {
                        window.titlebar_double_click();
                    } else if window.is_resizable() && window.window_controls().maximize {
                        window.zoom_window();
                    }
                } else {
                    window.start_window_move();
                }
                cx.stop_propagation();
            });
        }
        if Platform::current() == Platform::Linux {
            element = element.on_mouse_down(MouseButton::Right, |event, window, cx| {
                if window.window_controls().window_menu {
                    window.show_window_menu(event.position);
                    cx.stop_propagation();
                }
            });
        }
        element
    }
}

pub(crate) fn titlebar(
    config: &WindowRootConfig,
    status: &WindowFrameStatus,
    gate: Rc<RefCell<CloseGate>>,
    content: Option<gpui::AnyView>,
) -> impl IntoElement {
    let mut bar = div()
        .id("window-titlebar")
        .debug_selector(|| "window-titlebar".into())
        .h(px(38.0))
        .w_full()
        .flex()
        .items_center()
        .flex_shrink_0()
        .gap(px(4.0))
        .px(px(8.0));
    if Platform::current() == Platform::Mac {
        bar = bar.child(div().w(px(76.0)).flex_shrink_0());
    }
    bar = bar.child(div().text_size(px(12.0)).child(config.title.clone()));
    if let Some(content) = content {
        bar = bar.child(content);
    }
    bar = if config.movable {
        bar.child(DragRegion::new("window-drag"))
    } else {
        bar.child(div().flex_1())
    };
    if Platform::current() == Platform::Mac {
        return bar;
    }
    let minimize = caption_button("window-minimize", "最小化", IconName::Minus)
        .disabled(!config.minimizable || !status.controls.minimize)
        .on_click(|_, window, _| window.minimize_window());
    let mut maximize = caption_button(
        "window-maximize",
        if status.maximized {
            "还原"
        } else {
            "最大化"
        },
        if status.maximized {
            IconName::Copy
        } else {
            IconName::Square
        },
    )
    .disabled(!config.resizable || !status.controls.maximize)
    .on_click(|_, window, _| window.zoom_window());
    if Platform::current() == Platform::Windows {
        // Keep Snap Layout hover; Pressable alone dispatches maximize/restore.
        maximize = maximize.window_control_area(WindowControlArea::Max);
    }
    bar.child(minimize).child(maximize).child(
        caption_button("window-close", "关闭", IconName::X).on_click(move |_, window, cx| {
            if gate.borrow_mut().request(window, cx) {
                window.remove_window();
            }
        }),
    )
}

fn caption_button(id: &'static str, label: &'static str, icon: IconName) -> Button {
    Button::icon(id, label, Icon::new(icon))
        .size(ControlSize::Small)
        .variant(ButtonVariant::Quiet)
        .rounded_full()
}
