use std::{cell::RefCell, ops::Range, rc::Rc};

use gpui::{
    AnyWindowHandle, Context, Entity, IntoElement, Render, ScrollStrategy, TestAppContext,
    UniformListScrollHandle, Window, div, prelude::*, px, size,
};

use super::VirtualList;
use crate::{Root, components::Button, test_support};

struct Fixture {
    scroll: UniformListScrollHandle,
    rendered: Rc<RefCell<Vec<Range<usize>>>>,
    clicked: Option<usize>,
    custom_height: bool,
}

impl Render for Fixture {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rendered = self.rendered.clone();
        let entity = cx.entity();
        let list = VirtualList::new("virtual-rows", 10_000, move |range, _, cx| {
            rendered.borrow_mut().push(range.clone());
            entity.update(cx, |_, cx| {
                range
                    .map(|index| {
                        Button::new(format!("virtual-row-{index}"), format!("Row {index}"))
                            .h(px(32.0))
                            .min_h(px(32.0))
                            .w_full()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.clicked = Some(index);
                                cx.notify();
                            }))
                    })
                    .collect::<Vec<_>>()
            })
        })
        .label("Rows")
        .track_scroll(&self.scroll);
        let list = if self.custom_height {
            list.h(px(96.0))
        } else {
            list
        };
        Root::new("root").child(div().size_full().child(list))
    }
}

fn fixture(cx: &mut TestAppContext, custom_height: bool) -> (AnyWindowHandle, Entity<Fixture>) {
    cx.update(crate::init);
    let window = cx.open_window(size(px(400.0), px(500.0)), |_, _| Fixture {
        scroll: UniformListScrollHandle::new(),
        rendered: Rc::default(),
        clicked: None,
        custom_height,
    });
    let entity = window.root(cx).unwrap();
    let window = window.into();
    test_support::render(cx, window);
    (window, entity)
}

#[gpui::test]
fn virtual_list_builds_only_visible_rows_and_styled_height_overrides_default(
    cx: &mut TestAppContext,
) {
    let (window, entity) = fixture(cx, true);
    let rendered = entity.read_with(cx, |this, _| this.rendered.clone());
    assert!(!rendered.borrow().is_empty());
    assert!(rendered.borrow().iter().all(|range| range.end <= 4));
    let first = test_support::bounds(cx, window, "virtual-row-0");
    assert_eq!(first.size.height, px(32.0));
    let scroll = entity.read_with(cx, |this, _| this.scroll.clone());
    assert_eq!(scroll.0.borrow().base_handle.bounds().size.height, px(96.0));

    entity.update(cx, |this, cx| {
        this.custom_height = false;
        cx.notify();
    });
    test_support::render(cx, window);
    assert_eq!(
        scroll.0.borrow().base_handle.bounds().size.height,
        px(320.0)
    );
}

#[gpui::test]
fn virtual_list_scroll_reuses_identity_and_clicks_the_visible_row_id(cx: &mut TestAppContext) {
    let (window, entity) = fixture(cx, true);
    let (scroll, rendered) =
        entity.read_with(cx, |this, _| (this.scroll.clone(), this.rendered.clone()));
    rendered.borrow_mut().clear();
    scroll.scroll_to_item_strict(9000, ScrollStrategy::Top);
    test_support::render(cx, window);
    assert!(rendered.borrow().iter().any(|range| range.contains(&9000)));
    assert!(rendered.borrow().iter().all(|range| range.len() <= 4));
    test_support::click(cx, window, "virtual-row-9000");
    assert_eq!(entity.read_with(cx, |this, _| this.clicked), Some(9000));
    let offset = scroll.0.borrow().base_handle.offset();
    test_support::render(cx, window);
    assert_eq!(scroll.0.borrow().base_handle.offset(), offset);

    let bounds = test_support::bounds(cx, window, "virtual-row-9000");
    gpui::VisualTestContext::from_window(window, cx).simulate_event(gpui::ScrollWheelEvent {
        position: bounds.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-64.0))),
        ..Default::default()
    });
    cx.run_until_parked();
    test_support::render(cx, window);
    test_support::click(cx, window, "virtual-row-9002");
    assert_eq!(entity.read_with(cx, |this, _| this.clicked), Some(9002));
}
