use super::{PaintOperation, Scene};
use crate::{Bounds, Corners, Pixels, Point, ScaledPixels};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

/// Identity-bearing GPU group clip. Reuse the same value in prepaint and paint.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundedClip {
    /// Stable within a rendered subtree and its deferred draws.
    pub id: u64,
    /// Logical-pixel bounds; layout is unaffected.
    pub bounds: Bounds<Pixels>,
    /// Logical-pixel corner radii.
    pub radii: Corners<Pixels>,
}
impl RoundedClip {
    /// Create a scope. Maximum nesting is eight groups per window.
    pub fn new(bounds: Bounds<Pixels>, radii: Corners<Pixels>) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            bounds,
            radii: radii.clamp_radii_for_quad_size(bounds.size),
        }
    }
    /// Exact application hit clipping, independently of native input regions.
    pub fn contains(&self, p: Point<Pixels>) -> bool {
        if !self.bounds.contains(&p) {
            return false;
        }
        let b = self.bounds;
        for (r, x, y, in_corner) in [
            (
                self.radii.top_left,
                b.left() + self.radii.top_left,
                b.top() + self.radii.top_left,
                p.x < b.left() + self.radii.top_left && p.y < b.top() + self.radii.top_left,
            ),
            (
                self.radii.top_right,
                b.right() - self.radii.top_right,
                b.top() + self.radii.top_right,
                p.x > b.right() - self.radii.top_right && p.y < b.top() + self.radii.top_right,
            ),
            (
                self.radii.bottom_left,
                b.left() + self.radii.bottom_left,
                b.bottom() - self.radii.bottom_left,
                p.x < b.left() + self.radii.bottom_left
                    && p.y > b.bottom() - self.radii.bottom_left,
            ),
            (
                self.radii.bottom_right,
                b.right() - self.radii.bottom_right,
                b.bottom() - self.radii.bottom_right,
                p.x > b.right() - self.radii.bottom_right
                    && p.y > b.bottom() - self.radii.bottom_right,
            ),
        ] {
            if in_corner {
                let dx = f32::from(p.x - x);
                let dy = f32::from(p.y - y);
                if dx * dx + dy * dy > f32::from(r).powi(2) {
                    return false;
                }
            }
        }
        true
    }
}

/// Ordered render barrier. All primitives, including text and blur, lie between these barriers.
#[derive(Debug, Clone, Copy)]
pub struct RoundedClipBoundary {
    /// Scene order.
    pub order: u32,
    /// Device-space clip rectangle.
    pub bounds: Bounds<ScaledPixels>,
    /// Device-space radii.
    pub radii: Corners<ScaledPixels>,
    /// Begin saves background; end applies coverage once to the whole group.
    pub begin: bool,
}

impl Scene {
    pub(crate) fn push_rounded_clip(&mut self, clip: &RoundedClip, scale: f32) {
        self.paint_operations
            .push(PaintOperation::StartRoundedClip(clip.clone(), scale));
    }
    pub(crate) fn pop_rounded_clip(&mut self) {
        self.paint_operations.push(PaintOperation::EndRoundedClip);
    }

    /// Deferred content rejoins its original group before coverage is applied. Keeping
    /// identity in paint operations also makes cached subtree replay independent of indices.
    pub(super) fn finish_rounded_clips(&mut self) {
        if !self
            .paint_operations
            .iter()
            .any(|o| matches!(o, PaintOperation::StartRoundedClip(..)))
        {
            return;
        }
        enum Item {
            Operation(PaintOperation),
            Group(u64),
        }
        struct Group {
            clip: RoundedClip,
            scale: f32,
            items: Vec<Item>,
        }
        let operations = std::mem::take(&mut self.paint_operations);
        let mut groups: HashMap<u64, Group> = HashMap::new();
        let mut root = Vec::new();
        let mut stack = Vec::new();
        for op in operations.iter().cloned() {
            match op {
                PaintOperation::StartRoundedClip(clip, scale) => {
                    let id = clip.id;
                    if !groups.contains_key(&id) {
                        let item = Item::Group(id);
                        if let Some(parent) = stack.last() {
                            groups.get_mut(parent).unwrap().items.push(item);
                        } else {
                            root.push(item);
                        }
                        groups.insert(
                            id,
                            Group {
                                clip,
                                scale,
                                items: Vec::new(),
                            },
                        );
                    }
                    stack.push(id);
                }
                PaintOperation::EndRoundedClip => {
                    stack.pop().expect("balanced rounded clip");
                }
                op => {
                    let item = Item::Operation(op);
                    if let Some(parent) = stack.last() {
                        groups.get_mut(parent).unwrap().items.push(item);
                    } else {
                        root.push(item);
                    }
                }
            }
        }
        assert!(stack.is_empty(), "balanced rounded clip");
        fn append(items: Vec<Item>, groups: &mut HashMap<u64, Group>, scene: &mut Scene) {
            for item in items {
                match item {
                    Item::Operation(PaintOperation::Primitive(p)) => scene.insert_primitive(p),
                    Item::Operation(PaintOperation::StartLayer(b)) => scene.push_layer(b),
                    Item::Operation(PaintOperation::EndLayer) => scene.pop_layer(),
                    Item::Operation(_) => unreachable!(),
                    Item::Group(id) => {
                        let group = groups.remove(&id).unwrap();
                        let boundary = RoundedClipBoundary {
                            order: 0,
                            bounds: group.clip.bounds.scale(group.scale),
                            radii: group.clip.radii.scale(group.scale),
                            begin: true,
                        };
                        scene.insert_clip_boundary(boundary);
                        append(group.items, groups, scene);
                        scene.insert_clip_boundary(RoundedClipBoundary {
                            begin: false,
                            ..boundary
                        });
                    }
                }
            }
        }
        self.clear();
        append(root, &mut groups, self);
        self.paint_operations = operations;
    }
    fn insert_clip_boundary(&mut self, mut boundary: RoundedClipBoundary) {
        boundary.order = self.maximum_order + 1;
        self.maximum_order = boundary.order;
        self.minimum_order = boundary.order + 1;
        self.rounded_clips.push(boundary);
    }
}
