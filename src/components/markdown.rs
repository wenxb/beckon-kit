use std::{ops::Range, rc::Rc, sync::Arc};

use gpui::{
    AnyElement, App, ElementId, FontStyle, FontWeight, HighlightStyle, InteractiveText,
    IntoElement, ParentElement, Refineable, RenderOnce, SharedString, StrikethroughStyle,
    StyleRefinement, Styled, StyledText, UnderlineStyle, Window, div, prelude::*, px,
};
use markdown::{ParseOptions, mdast::Node};

use crate::theme::Theme;

type LinkHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// Read-only CommonMark/GFM content. Links are inert unless the host supplies a handler.
/// HTML stays literal, and image references are rendered as alt text without fetching.
#[derive(IntoElement)]
pub struct Markdown {
    id: ElementId,
    source: SharedString,
    callback: Option<LinkHandler>,
    style: StyleRefinement,
}

impl Markdown {
    pub fn new(id: impl Into<ElementId>, source: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            source: source.into(),
            callback: None,
            style: StyleRefinement::default(),
        }
    }
    pub fn on_link(mut self, callback: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.callback = Some(Rc::new(callback));
        self
    }
}
impl Styled for Markdown {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

struct Parsed {
    source: SharedString,
    tree: Arc<Node>,
}
impl Parsed {
    fn new(source: SharedString) -> Self {
        let tree = markdown::to_mdast(&source, &ParseOptions::gfm())
            .expect("CommonMark/GFM without MDX has no syntax errors");
        Self {
            source,
            tree: Arc::new(tree),
        }
    }
}

impl RenderOnce for Markdown {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let cache = window.use_keyed_state((self.id.clone(), "markdown"), cx, |_, _| {
            Parsed::new(self.source.clone())
        });
        if cache.read(cx).source != self.source {
            cache.update(cx, |parsed, _| *parsed = Parsed::new(self.source));
        }
        let tree = cache.read(cx).tree.clone();
        let theme = Theme::current(cx);
        let mut index = 0;
        let selector = self.id.to_string();
        let mut root = div()
            .id(self.id.clone())
            .debug_selector(move || selector)
            .flex()
            .flex_col()
            .gap(px(8.0))
            .w_full()
            .text_color(theme.colors.text)
            .text_size(px(theme.typography.body));
        for node in tree.children().into_iter().flatten() {
            root = root.child(block(
                node,
                &self.id,
                &mut index,
                &theme,
                self.callback.clone(),
            ));
        }
        root.style().refine(&self.style);
        root
    }
}

fn block(
    node: &Node,
    id: &ElementId,
    index: &mut usize,
    theme: &Theme,
    callback: Option<LinkHandler>,
) -> AnyElement {
    *index += 1;
    let current = *index;
    let children = |index: &mut usize| {
        node.children()
            .into_iter()
            .flatten()
            .map(|child| block(child, id, index, theme, callback.clone()))
            .collect::<Vec<_>>()
    };
    match node {
        Node::Definition(_) => div().into_any_element(),
        Node::Heading(heading) => div()
            .text_size(px(match heading.depth {
                1 => 24.0,
                2 => 20.0,
                _ => 16.0,
            }))
            .font_weight(FontWeight::SEMIBOLD)
            .child(inline(
                node,
                (id.clone(), SharedString::from(format!("text-{current}"))).into(),
                theme,
                callback,
            ))
            .into_any_element(),
        Node::Paragraph(_) | Node::TableCell(_) => div()
            .child(inline(
                node,
                (id.clone(), SharedString::from(format!("text-{current}"))).into(),
                theme,
                callback,
            ))
            .into_any_element(),
        Node::Code(code) => div()
            .p(px(10.0))
            .rounded(px(6.0))
            .bg(theme.colors.elevated)
            .font_family("monospace")
            .whitespace_normal()
            .child(code.value.clone())
            .into_any_element(),
        Node::Blockquote(_) => div()
            .border_l_2()
            .border_color(theme.colors.border)
            .pl(px(12.0))
            .flex()
            .flex_col()
            .gap(px(6.0))
            .children(children(index))
            .into_any_element(),
        Node::ThematicBreak(_) => div()
            .h(px(1.0))
            .w_full()
            .bg(theme.colors.border)
            .into_any_element(),
        Node::List(list) => {
            let mut result = div().flex().flex_col().gap(px(4.0));
            for (offset, child) in list.children.iter().enumerate() {
                let marker = if list.ordered {
                    format!("{}.", list.start.unwrap_or(1) as usize + offset)
                } else {
                    "•".to_owned()
                };
                let marker = if let Node::ListItem(item) = child {
                    item.checked
                        .map(|checked| if checked { "☑" } else { "☐" }.to_owned())
                        .unwrap_or(marker)
                } else {
                    marker
                };
                result = result.child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .child(div().w(px(24.0)).flex_shrink_0().child(marker))
                        .child(div().flex_1().min_w_0().child(block(
                            child,
                            id,
                            index,
                            theme,
                            callback.clone(),
                        ))),
                );
            }
            result.into_any_element()
        }
        Node::TableRow(_) => div()
            .flex()
            .gap(px(12.0))
            .children(children(index))
            .into_any_element(),
        Node::Html(html) => div().child(html.value.clone()).into_any_element(),
        _ => div()
            .flex()
            .flex_col()
            .gap(px(6.0))
            .children(children(index))
            .into_any_element(),
    }
}

#[derive(Default)]
struct Inline {
    text: String,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
    monospace: Vec<(Range<usize>, SharedString)>,
    links: Vec<(Range<usize>, String)>,
}
impl Inline {
    fn append(&mut self, text: &str, style: HighlightStyle) {
        let start = self.text.len();
        self.text.push_str(text);
        if self.text.len() > start {
            self.highlights.push((start..self.text.len(), style));
        }
    }
    fn visit(&mut self, node: &Node, mut style: HighlightStyle, theme: &Theme) {
        match node {
            Node::Strong(_) => style.font_weight = Some(FontWeight::BOLD),
            Node::Emphasis(_) => style.font_style = Some(FontStyle::Italic),
            Node::Delete(_) => {
                style.strikethrough = Some(StrikethroughStyle {
                    thickness: px(1.0),
                    color: None,
                })
            }
            Node::InlineCode(_) => style.background_color = Some(theme.colors.elevated),
            Node::Link(_) => {
                style.color = Some(theme.colors.info);
                style.underline = Some(UnderlineStyle {
                    thickness: px(1.0),
                    color: None,
                    wavy: false,
                });
            }
            _ => {}
        }
        let start = self.text.len();
        if let Some(children) = node.children() {
            for child in children {
                self.visit(child, style, theme);
            }
        } else {
            match node {
                Node::Break(_) => self.append("\n", style),
                Node::Image(image) => self.append(&image.alt, style),
                Node::ImageReference(image) => self.append(&image.alt, style),
                _ => self.append(&node.to_string(), style),
            }
        }
        if matches!(node, Node::InlineCode(_)) && self.text.len() > start {
            self.monospace
                .push((start..self.text.len(), "monospace".into()));
        }
        if let Node::Link(link) = node
            && self.text.len() > start
        {
            self.links.push((start..self.text.len(), link.url.clone()));
        }
    }
}

fn inline(node: &Node, id: ElementId, theme: &Theme, callback: Option<LinkHandler>) -> AnyElement {
    let mut text = Inline::default();
    text.visit(node, HighlightStyle::default(), theme);
    let styled = StyledText::new(text.text)
        .with_highlights(text.highlights)
        .with_font_family_overrides(text.monospace);
    if let Some(callback) = callback {
        let (ranges, urls): (Vec<_>, Vec<_>) = text.links.into_iter().unzip();
        InteractiveText::new(id, styled)
            .on_click(ranges, move |index, window, cx| {
                callback(&urls[index], window, cx)
            })
            .into_any_element()
    } else {
        styled.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_styles_are_disjoint_utf8_ranges_and_links_preserve_labels() {
        let parsed = Parsed::new(
            "**粗体 *强调*** 和 [链接](https://example.test) ![alt](file:///never-read)".into(),
        );
        let mut inline = Inline::default();
        inline.visit(&parsed.tree, HighlightStyle::default(), &Theme::default());
        assert_eq!(inline.text, "粗体 强调 和 链接 alt");
        assert!(
            inline
                .highlights
                .windows(2)
                .all(|pair| pair[0].0.end <= pair[1].0.start)
        );
        assert_eq!(&inline.text[inline.links[0].0.clone()], "链接");
        assert_eq!(inline.links[0].1, "https://example.test");
    }
}
