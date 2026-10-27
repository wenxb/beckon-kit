use gpui::{Bounds, Pixels};

use super::{Phase, Point, PointerEvent, Transform};

#[derive(Default)]
pub(super) struct Input {
    pub bounds: Bounds<Pixels>,
    pub transform: Transform,
    drag: Option<Drag>,
}

struct Drag {
    origin: Point,
    target: Option<String>,
    last: Point,
}

impl Input {
    pub fn active(&self) -> bool {
        self.drag.is_some()
    }

    pub fn local(&self, position: gpui::Point<Pixels>) -> Point {
        Point {
            x: f32::from(position.x - self.bounds.origin.x),
            y: f32::from(position.y - self.bounds.origin.y),
        }
    }

    pub fn down(
        &mut self,
        position: gpui::Point<Pixels>,
        target: Option<String>,
    ) -> Option<PointerEvent> {
        let position = self.transform.inverse_point(self.local(position))?;
        self.drag = Some(Drag {
            origin: position,
            target: target.clone(),
            last: position,
        });
        Some(PointerEvent {
            phase: Phase::Down,
            position,
            origin: position,
            target,
        })
    }

    pub fn next(&mut self, phase: Phase, position: gpui::Point<Pixels>) -> Option<PointerEvent> {
        let mapped = self.transform.inverse_point(self.local(position));
        let drag = self.drag.as_mut()?;
        let phase = if mapped.is_some() {
            phase
        } else {
            Phase::Cancel
        };
        let position = mapped.unwrap_or(drag.last);
        drag.last = position;
        let event = PointerEvent {
            phase,
            position,
            origin: drag.origin,
            target: drag.target.clone(),
        };
        if matches!(phase, Phase::Up | Phase::Cancel) {
            self.drag = None;
        }
        Some(event)
    }
}
