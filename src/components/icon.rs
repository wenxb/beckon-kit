use gpui::{
    App, IntoElement, Refineable, RenderOnce, SharedString, StyleRefinement, Styled, Window, px,
    svg,
};
use std::sync::Arc;

macro_rules! lucide_catalog {
    ($($name:ident => $file:literal),+ $(,)?) => {
        /// Curated Lucide/Feather names, independent of any host asset registry.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum IconName { $($name),+ }

        impl IconName {
            pub const ALL: &'static [Self] = &[$(Self::$name),+];

            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$name => $file),+ }
            }

            pub const fn svg(self) -> &'static [u8] {
                match self {
                    $(Self::$name => include_bytes!(concat!("../../assets/lucide/", $file, ".svg"))),+
                }
            }
        }
    };
}

lucide_catalog! {
    Component => "component",
    PanelsTopLeft => "panels-top-left",
    Layers => "layers",
    AppWindow => "app-window",
    Palette => "palette",
    Shapes => "shapes",
    Check => "check",
    X => "x",
    Minus => "minus",
    Square => "square",
    Plus => "plus",
    Search => "search",
    Copy => "copy",
    ChevronLeft => "chevron-left",
    ChevronRight => "chevron-right",
    ChevronDown => "chevron-down",
    ChevronUp => "chevron-up",
    CircleCheck => "circle-check",
    Info => "info",
    TriangleAlert => "triangle-alert",
    CircleX => "circle-x",
    Sun => "sun",
    Moon => "moon",
    Settings2 => "settings-2",
    ExternalLink => "external-link",
    PanelLeft => "panel-left",
    RotateCcw => "rotate-ccw",
    Github => "github",
    Settings => "settings",
    Calculator => "calculator",
    Terminal => "terminal",
    Globe => "globe",
    SlidersHorizontal => "sliders-horizontal",
    Keyboard => "keyboard",
    Blocks => "blocks",
    ArrowLeft => "arrow-left",
    ArrowUpRight => "arrow-up-right",
    BookOpen => "book-open",
    Sparkles => "sparkles",
    LayoutGrid => "layout-grid",
}

#[derive(Clone)]
enum Source {
    Named(IconName),
    Data(Arc<[u8]>),
    Path(SharedString),
}

/// A decorative 16px icon inheriting text color. Its parent owns the accessible label.
#[derive(Clone, IntoElement)]
pub struct Icon {
    source: Source,
    style: StyleRefinement,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self::from_source(Source::Named(name))
    }

    /// Owned bytes for a custom monochrome SVG; clones share the source.
    pub fn data(bytes: &[u8]) -> Self {
        Self::from_source(Source::Data(Arc::from(bytes)))
    }

    /// A path supplied by the host's GPUI AssetSource, not a filesystem path.
    pub fn path(path: impl Into<SharedString>) -> Self {
        Self::from_source(Source::Path(path.into()))
    }

    fn from_source(source: Source) -> Self {
        Self {
            source,
            style: StyleRefinement::default(),
        }
    }
}

impl Styled for Icon {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Icon {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let mut element = match self.source {
            Source::Named(name) => svg().data(name.svg()),
            Source::Data(bytes) => svg().data(&bytes),
            Source::Path(path) => svg().path(path),
        }
        .size(px(16.0))
        .flex_shrink_0()
        // GPUI's SVG painter needs an explicit color, unlike text elements.
        .text_color(window.text_style().color);
        element.style().refine(&self.style);
        element
    }
}
