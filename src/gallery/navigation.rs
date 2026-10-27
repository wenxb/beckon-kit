use gpui::{Context, FocusHandle, IntoElement, Role, Window, div, prelude::*, px};

use super::{Gallery, Page};
use crate::{
    components::{Icon, IconName},
    theme::Theme,
};

impl Page {
    pub(super) const ALL: [Self; 7] = [
        Self::Colors,
        Self::Icons,
        Self::Controls,
        Self::VirtualList,
        Self::Canvas,
        Self::Materials,
        Self::Window,
    ];

    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Colors => "颜色",
            Self::Icons => "图标",
            Self::Controls => "基础控件",
            Self::VirtualList => "虚拟列表",
            Self::Canvas => "画布",
            Self::Materials => "材质",
            Self::Window => "窗口",
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Self::Colors => "中性色负责层级，语义色表达状态。",
            Self::Icons => "Lucide 线性图标，统一尺寸与前景色。",
            Self::Controls => "按钮、输入、选择与状态反馈。",
            Self::VirtualList => "固定行高、按可见范围构建与持久滚动身份。",
            Self::Canvas => "原生路径、文字、场景变换、裁剪与指针捕获。",
            Self::Materials => "实时背景模糊、材质层级与无障碍降级。",
            Self::Window => "窗口外框、圆角与平台能力。",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Self::Colors => "colors",
            Self::Icons => "icons",
            Self::Controls => "controls",
            Self::VirtualList => "virtual-list",
            Self::Canvas => "canvas",
            Self::Materials => "materials",
            Self::Window => "window",
        }
    }

    fn icon(self) -> IconName {
        match self {
            Self::Colors => IconName::Palette,
            Self::Icons => IconName::Shapes,
            Self::Controls => IconName::PanelsTopLeft,
            Self::VirtualList => IconName::PanelsTopLeft,
            Self::Canvas => IconName::Shapes,
            Self::Materials => IconName::Layers,
            Self::Window => IconName::AppWindow,
        }
    }
}

impl Gallery {
    pub(super) fn sidebar(
        &self,
        narrow: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let theme = Theme::new(self.appearance);
        let colors = theme.colors;
        let focus: FocusHandle = window
            .use_keyed_state("gallery-navigation-focus", cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        div()
            .id("gallery-sidebar")
            .debug_selector(|| "gallery-sidebar".into())
            .w(px(if narrow { 112.0 } else { 192.0 }))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .gap(px(24.0))
            .p(px(if narrow { 10.0 } else { 16.0 }))
            .border_r(px(1.0))
            .border_color(colors.border)
            .bg(colors.canvas)
            .child(
                div()
                    .h(px(40.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(Icon::new(IconName::Component))
                    .child(
                        div()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Beckon Kit"),
                    ),
            )
            .child(
                div()
                    .id("page")
                    .role(Role::TabList)
                    .aria_orientation(gpui::Orientation::Vertical)
                    .aria_label("组件文档")
                    .aria_description("使用上下方向键切换页面")
                    .track_focus(&focus)
                    .rounded(px(6.0))
                    .focus_visible(move |style| {
                        style.shadow(vec![
                            gpui::BoxShadow::new(px(0.0), px(0.0), colors.focus)
                                .spread_radius(px(1.0)),
                        ])
                    })
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .on_key_down(cx.listener(|gallery, event: &gpui::KeyDownEvent, _, cx| {
                        if event.keystroke.modifiers.modified() {
                            return;
                        }
                        let index = Page::ALL
                            .iter()
                            .position(|page| *page == gallery.page)
                            .unwrap();
                        let next = match event.keystroke.key.as_str() {
                            "up" | "left" => (index + Page::ALL.len() - 1) % Page::ALL.len(),
                            "down" | "right" => (index + 1) % Page::ALL.len(),
                            "home" => 0,
                            "end" => Page::ALL.len() - 1,
                            _ => return,
                        };
                        gallery.show_page(Page::ALL[next], cx);
                        cx.stop_propagation();
                    }))
                    .children(Page::ALL.into_iter().map(|page| {
                        let selected = self.page == page;
                        let selector = format!("page-{}", page.key());
                        let focus = focus.clone();
                        div()
                            .id(gpui::SharedString::from(selector.clone()))
                            .debug_selector(move || selector.clone())
                            .role(Role::Tab)
                            .aria_label(page.name())
                            .aria_selected(selected)
                            .when(selected, |item| item.aria_active_descendant())
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_h(px(34.0))
                            .px(px(8.0))
                            .rounded(px(6.0))
                            .bg(if selected {
                                colors.selection
                            } else {
                                gpui::transparent_black()
                            })
                            .text_color(if selected {
                                colors.info
                            } else {
                                colors.secondary
                            })
                            .cursor_pointer()
                            .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                                focus.focus(window, cx)
                            })
                            .on_click(cx.listener(move |gallery, _, _, cx| {
                                gallery.show_page(page, cx);
                                cx.stop_propagation();
                            }))
                            .child(Icon::new(page.icon()))
                            .child(page.name())
                    })),
            )
    }
}
