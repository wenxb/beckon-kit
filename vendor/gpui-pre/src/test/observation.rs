//! Completed-frame facts from native identified divs, including cached paint replay.
use crate::{Bounds, ElementId, Pixels, Role, Window, accesskit};

/// Layout and accessibility facts recorded for an identified element.
#[derive(Clone, Debug)]
pub struct ElementSnapshot {
    pub(crate) path: Vec<ElementId>,
    pub(crate) bounds: Bounds<Pixels>,
    pub(crate) visible: bool,
    pub(crate) focused: Option<bool>,
    pub(crate) node: accesskit::Node,
}

impl ElementSnapshot {
    /// Full identity path in the rendered tree.
    pub fn path(&self) -> &[ElementId] {
        &self.path
    }
    /// Laid-out window coordinates.
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.bounds
    }
    /// Whether the element intersects the active content mask.
    pub fn visible(&self) -> bool {
        self.visible
    }
    /// Focus state, when the element tracks a focus handle.
    pub fn focused(&self) -> Option<bool> {
        self.focused
    }
    /// Accessibility role supplied by the rendered element.
    pub fn role(&self) -> Role {
        self.node.role()
    }
    /// Controlled toggle state, if supplied.
    pub fn checked(&self) -> Option<bool> {
        self.node
            .toggled()
            .map(|state| state == accesskit::Toggled::True)
    }
    /// Accessible label, if supplied.
    pub fn label(&self) -> Option<&str> {
        self.node.label()
    }
    /// Accessible value, if supplied.
    pub fn value(&self) -> Option<&str> {
        self.node.value()
    }
    /// Expansion state for popup controls.
    pub fn expanded(&self) -> Option<bool> {
        self.node.is_expanded()
    }
}

/// Snapshots from the last completed frame, including cached paint replay.
pub fn snapshots(window: &Window) -> &[ElementSnapshot] {
    &window.rendered_frame.element_snapshots
}

/// Find one identity beneath a scope; ambiguous identities fail rather than picking one.
pub fn find(window: &Window, scope: &[ElementId], id: &ElementId) -> Option<ElementSnapshot> {
    let mut matches = snapshots(window).iter().filter(|entry| {
        entry.path.len() > scope.len()
            && entry.path.starts_with(scope)
            && entry.path.last() == Some(id)
    });
    let found = matches.next().cloned();
    assert!(
        matches.next().is_none(),
        "ambiguous ElementId {id:?}; use within(...) to select a scope"
    );
    found
}

/// Resolve an identity path to use as a child query scope.
pub fn scope(window: &Window, parent: &[ElementId], id: &ElementId) -> Vec<ElementId> {
    let mut matches = std::collections::HashSet::new();
    for entry in snapshots(window)
        .iter()
        .filter(|entry| entry.path.starts_with(parent))
    {
        for index in parent.len()..entry.path.len() {
            if &entry.path[index] == id {
                matches.insert(entry.path[..=index].to_vec());
            }
        }
    }
    assert_eq!(
        matches.len(),
        1,
        "expected one scope {id:?}; registered paths: {}",
        registered_paths(window)
    );
    matches.into_iter().next().unwrap()
}

/// Whether a completed frame recorded focus within a scope.
pub fn has_observed_focus(window: &Window, scope: &[ElementId]) -> bool {
    snapshots(window)
        .iter()
        .any(|entry| entry.path.starts_with(scope) && entry.focused == Some(true))
}

/// Rendered identities for diagnostics when a query fails.
pub fn registered_paths(window: &Window) -> String {
    snapshots(window)
        .iter()
        .map(|entry| format!("{:?}", entry.path))
        .collect::<Vec<_>>()
        .join(", ")
}
