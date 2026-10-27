use std::sync::Arc;

use gpui::{
    App, Bounds, ContentMask, Font, Hsla, Path, PathBuilder, Pixels, ShapedLine, SharedString,
    TextAlign, TextRun, Window, fill, outline, point, px, size,
};

use super::{
    Node, Point, Rect, Scene, Stroke, Transform,
    hit::{HitShape, arrow_points},
};

pub(super) struct Compiled {
    pub scene: Arc<Scene>,
    pub font: Font,
    pub alternative: SharedString,
    leaves: Vec<Leaf>,
    paint_origin: gpui::Point<Pixels>,
}

struct Leaf {
    id: String,
    clip: Rect,
    hit: HitShape,
    drawing: Drawing,
}

enum Drawing {
    Rect {
        rect: Rect,
        fill: Option<Hsla>,
        stroke: Option<Stroke>,
    },
    Arrow {
        path: Path<Pixels>,
        color: Hsla,
    },
    Text {
        origin: Point,
        line: Box<ShapedLine>,
        height: Pixels,
    },
}

impl Compiled {
    pub fn new(scene: Arc<Scene>, font: Font, window: &mut Window) -> Self {
        let mut compiled = Self {
            scene: scene.clone(),
            font,
            alternative: SharedString::default(),
            leaves: Vec::new(),
            paint_origin: point(px(0.0), px(0.0)),
        };
        let mut alternative = String::new();
        if scene.transform.is_valid() {
            compiled.nodes(
                &scene.children,
                scene.transform,
                scene.viewport(),
                &mut alternative,
                window,
            );
        }
        compiled.alternative = alternative.into();
        compiled
    }

    fn nodes(
        &mut self,
        nodes: &[Node],
        transform: Transform,
        clip: Rect,
        alternative: &mut String,
        window: &mut Window,
    ) {
        for node in nodes {
            match node {
                Node::Group {
                    transform: child,
                    clip: child_clip,
                    children,
                } => {
                    let transform = transform.compose(*child);
                    if !transform.is_valid() {
                        continue;
                    }
                    let clip =
                        child_clip.map_or(clip, |rect| clip.intersection(transform.map_rect(rect)));
                    self.nodes(children, transform, clip, alternative, window);
                }
                Node::Rect {
                    id,
                    rect,
                    fill,
                    stroke,
                } => {
                    let rect = transform.map_rect(*rect);
                    let stroke = stroke.map(|stroke| Stroke {
                        color: stroke.color,
                        width: stroke.width * transform.scale,
                    });
                    self.leaves.push(Leaf {
                        id: id.clone(),
                        clip,
                        hit: HitShape::Rect {
                            rect,
                            filled: fill.is_some(),
                            stroke: stroke.map_or(0.0, |stroke| stroke.width),
                        },
                        drawing: Drawing::Rect {
                            rect,
                            fill: *fill,
                            stroke,
                        },
                    });
                    alternative.push_str(&format!("Rectangle {id}. "));
                }
                Node::Arrow {
                    id,
                    from,
                    to,
                    stroke,
                } => {
                    let Some(points) = arrow_points(*from, *to, stroke.width) else {
                        continue;
                    };
                    let points = points.map(|point| transform.map_point(point));
                    let width = stroke.width * transform.scale;
                    let mut path = PathBuilder::stroke(px(width));
                    let [from, to, left, right] = points.map(gpui_point);
                    path.move_to(from);
                    path.line_to(to);
                    path.move_to(left);
                    path.line_to(to);
                    path.move_to(right);
                    path.line_to(to);
                    self.leaves.push(Leaf {
                        id: id.clone(),
                        clip,
                        hit: HitShape::Arrow {
                            points,
                            tolerance: (width / 2.0).max(3.0),
                        },
                        drawing: Drawing::Arrow {
                            path: path.build().expect("valid canvas arrow"),
                            color: stroke.color,
                        },
                    });
                    alternative.push_str(&format!("Arrow {id}. "));
                }
                Node::Text {
                    id,
                    origin,
                    text,
                    size,
                    color,
                } => {
                    let origin = transform.map_point(*origin);
                    let line = window.text_system().shape_line(
                        text.clone(),
                        px(*size * transform.scale),
                        &[TextRun {
                            len: text.len(),
                            font: self.font.clone(),
                            color: *color,
                            ..Default::default()
                        }],
                        None,
                    );
                    let height = line.ascent + line.descent;
                    let bounds = Rect {
                        x: origin.x,
                        y: origin.y,
                        width: f32::from(line.width()),
                        height: f32::from(height),
                    };
                    self.leaves.push(Leaf {
                        id: id.clone(),
                        clip,
                        hit: HitShape::Text(bounds),
                        drawing: Drawing::Text {
                            origin,
                            line: Box::new(line),
                            height,
                        },
                    });
                    if !text.is_empty() {
                        alternative.push_str(text);
                        alternative.push_str(". ");
                    }
                }
            }
        }
    }

    pub fn hit_test(&self, point: Point) -> Option<&str> {
        if !self.scene.viewport().contains(point) {
            return None;
        }
        self.leaves
            .iter()
            .rev()
            .find(|leaf| leaf.clip.contains(point) && leaf.hit.contains(point))
            .map(|leaf| leaf.id.as_str())
    }

    pub fn place(&mut self, origin: gpui::Point<Pixels>) {
        let offset = origin - self.paint_origin;
        if offset == point(px(0.0), px(0.0)) {
            return;
        }
        for leaf in &mut self.leaves {
            if let Drawing::Arrow { path, .. } = &mut leaf.drawing {
                path.bounds.origin += offset;
                for vertex in &mut path.vertices {
                    vertex.xy_position += offset;
                }
            }
        }
        self.paint_origin = origin;
    }

    pub fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let viewport = self.scene.viewport();
        let visible = Rect {
            width: f32::from(bounds.size.width).min(viewport.width),
            height: f32::from(bounds.size.height).min(viewport.height),
            ..viewport
        };
        for leaf in &self.leaves {
            let clip = visible.intersection(leaf.clip);
            if clip.width <= 0.0 || clip.height <= 0.0 {
                continue;
            }
            window.with_content_mask(
                Some(ContentMask {
                    bounds: gpui_bounds(clip, bounds.origin),
                }),
                |window| {
                    // Parent surfaces batch primitive kinds; each leaf needs its own draw order.
                    window.paint_layer(gpui_bounds(clip, bounds.origin), |window| {
                        match &leaf.drawing {
                            Drawing::Rect {
                                rect,
                                fill: color,
                                stroke,
                            } => {
                                if let Some(color) = color {
                                    window.paint_quad(fill(
                                        gpui_bounds(*rect, bounds.origin),
                                        *color,
                                    ));
                                }
                                if let Some(stroke) = stroke {
                                    window.paint_quad(
                                        outline(
                                            gpui_bounds(
                                                rect.outset(stroke.width / 2.0),
                                                bounds.origin,
                                            ),
                                            stroke.color,
                                            gpui::BorderStyle::default(),
                                        )
                                        .border_widths(px(stroke.width)),
                                    );
                                }
                            }
                            Drawing::Arrow { path, color } => {
                                // GPUI consumes paths; the cached tessellation is only copied for submission.
                                window.paint_path(path.clone(), *color);
                            }
                            Drawing::Text {
                                origin,
                                line,
                                height,
                            } => {
                                line.paint(
                                    bounds.origin + gpui_point(*origin),
                                    *height,
                                    TextAlign::Left,
                                    None,
                                    window,
                                    cx,
                                )
                                .expect("paint canvas text");
                            }
                        }
                    });
                },
            );
        }
    }
}

fn gpui_point(value: Point) -> gpui::Point<Pixels> {
    point(px(value.x), px(value.y))
}

fn gpui_bounds(rect: Rect, origin: gpui::Point<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        origin + point(px(rect.x), px(rect.y)),
        size(px(rect.width), px(rect.height)),
    )
}
