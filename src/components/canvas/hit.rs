use super::{Point, Rect};

pub(super) enum HitShape {
    Rect {
        rect: Rect,
        filled: bool,
        stroke: f32,
    },
    Arrow {
        points: [Point; 4],
        tolerance: f32,
    },
    Text(Rect),
}

impl HitShape {
    pub fn contains(&self, position: Point) -> bool {
        match self {
            Self::Rect {
                rect,
                filled,
                stroke,
            } => {
                (*filled && rect.contains(position))
                    || (*stroke > 0.0
                        && rect.outset(*stroke / 2.0).contains(position)
                        && !rect.outset(-*stroke / 2.0).contains(position))
            }
            Self::Arrow { points, tolerance } => {
                let [from, to, left, right] = *points;
                [(from, to), (left, to), (right, to)]
                    .into_iter()
                    .any(|(from, to)| segment_distance(position, from, to) <= *tolerance)
            }
            Self::Text(bounds) => bounds.contains(position),
        }
    }
}

pub(super) fn arrow_points(from: Point, to: Point, width: f32) -> Option<[Point; 4]> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let length = dx.hypot(dy);
    if !length.is_finite() || length <= 0.0 {
        return None;
    }
    let head = (width * 4.0).max(8.0).min(length / 2.0);
    let back = Point {
        x: to.x - dx / length * head,
        y: to.y - dy / length * head,
    };
    let side = head * 0.5;
    Some([
        from,
        to,
        Point {
            x: back.x - dy / length * side,
            y: back.y + dx / length * side,
        },
        Point {
            x: back.x + dy / length * side,
            y: back.y - dx / length * side,
        },
    ])
}

fn segment_distance(point: Point, from: Point, to: Point) -> f32 {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return (point.x - from.x).hypot(point.y - from.y);
    }
    let ux = dx / length;
    let uy = dy / length;
    let distance = ((point.x - from.x) * ux + (point.y - from.y) * uy).clamp(0.0, length);
    (point.x - from.x - ux * distance).hypot(point.y - from.y - uy * distance)
}
