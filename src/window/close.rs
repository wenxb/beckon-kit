use gpui::{App, Window};
use std::rc::Rc;

gpui::actions!(beckon_kit, [CloseWindow]);

/// Opaque, window-local request identity. Only the matching pending request may resume.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseRequest(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseDecision {
    Allow,
    Cancel,
    Pending,
}

pub(crate) type CloseHandler = Rc<dyn Fn(CloseRequest, &mut Window, &mut App) -> CloseDecision>;

pub(crate) struct CloseGate {
    pending: Option<CloseRequest>,
    handler: Option<CloseHandler>,
}
impl CloseGate {
    pub fn new(handler: Option<CloseHandler>) -> Self {
        Self {
            pending: None,
            handler,
        }
    }
    pub fn request(&mut self, window: &mut Window, cx: &mut App) -> bool {
        if self.pending.is_some() {
            return false;
        }
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let request = CloseRequest(NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
        self.pending = Some(request);
        let decision = self
            .handler
            .as_ref()
            .map_or(CloseDecision::Allow, |handler| handler(request, window, cx));
        if decision != CloseDecision::Pending {
            self.pending = None;
        }
        decision == CloseDecision::Allow
    }
    pub fn resume(&mut self, request: CloseRequest, allow: bool) -> bool {
        if self.pending != Some(request) {
            return false;
        }
        self.pending = None;
        allow
    }
}
