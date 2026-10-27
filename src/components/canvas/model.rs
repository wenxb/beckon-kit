use gpui::{Hsla, SharedString};

/// A logical pixel position in the coordinate system of its containing node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains(self, point: Point) -> bool {
        self.width > 0.0
            && self.height > 0.0
            && point.x >= self.x
            && point.y >= self.y
            && point.x <= self.x + self.width
            && point.y <= self.y + self.height
    }

    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: (self.x + self.width).min(other.x + other.width) - x,
            height: (self.y + self.height).min(other.y + other.height) - y,
        }
    }

    pub(super) fn outset(self, amount: f32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: self.width + amount * 2.0,
            height: self.height + amount * 2.0,
        }
    }
}

/// Positive uniform scale followed by translation: `local * scale + (x, y)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub scale: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale: 1.0,
        }
    }
}

impl Transform {
    pub fn map_point(self, point: Point) -> Point {
        Point {
            x: point.x * self.scale + self.x,
            y: point.y * self.scale + self.y,
        }
    }

    pub fn inverse_point(self, point: Point) -> Option<Point> {
        if !self.is_valid() || !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let point = Point {
            x: (point.x - self.x) / self.scale,
            y: (point.y - self.y) / self.scale,
        };
        (point.x.is_finite() && point.y.is_finite()).then_some(point)
    }

    /// Compose a child transform with this local-to-parent transform.
    pub fn compose(self, child: Self) -> Self {
        let origin = self.map_point(Point {
            x: child.x,
            y: child.y,
        });
        Self {
            x: origin.x,
            y: origin.y,
            scale: self.scale * child.scale,
        }
    }

    pub fn map_rect(self, rect: Rect) -> Rect {
        let origin = self.map_point(Point {
            x: rect.x,
            y: rect.y,
        });
        Rect {
            x: origin.x,
            y: origin.y,
            width: rect.width * self.scale,
            height: rect.height * self.scale,
        }
    }

    pub(super) fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.scale.is_finite() && self.scale > 0.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub color: Hsla,
    pub width: f32,
}

/// Immutable retained geometry. Hosts validate finite coordinates, positive sizes/scales,
/// unique nonempty leaf ids, single-line text, and application-specific complexity limits.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    pub width: f32,
    pub height: f32,
    pub transform: Transform,
    pub children: Vec<Node>,
}

impl Scene {
    pub fn viewport(&self) -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: self.width,
            height: self.height,
        }
    }

    pub fn viewport_to_scene(&self, point: Point) -> Option<Point> {
        self.transform.inverse_point(point)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Rect {
        id: String,
        rect: Rect,
        fill: Option<Hsla>,
        stroke: Option<Stroke>,
    },
    Arrow {
        id: String,
        from: Point,
        to: Point,
        stroke: Stroke,
    },
    /// `origin` is the top-left of the native shaped line's typographic box.
    Text {
        id: String,
        origin: Point,
        text: SharedString,
        size: f32,
        color: Hsla,
    },
    Group {
        transform: Transform,
        clip: Option<Rect>,
        children: Vec<Node>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerEvent {
    pub phase: Phase,
    pub position: Point,
    pub origin: Point,
    pub target: Option<String>,
}
