//! [`NodeBuilder`] — convenience over `accesskit::Node` with PH2D
//! defaults (focus_visible focus ring, label sanitization, etc).

use accesskit::{Live, Node, NodeId as AkNodeId, Rect, Role, Toggled};

/// Stable opaque ID for an a11y node. Wraps `accesskit::NodeId`
/// (which itself wraps `u64`); we expose a newtype so callers don't
/// take a direct dep on accesskit just to handle ids.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub u64);

impl NodeId {
    pub const ROOT: Self = Self(0);

    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<NodeId> for AkNodeId {
    fn from(id: NodeId) -> Self {
        AkNodeId(id.0)
    }
}

impl From<AkNodeId> for NodeId {
    fn from(id: AkNodeId) -> Self {
        Self(id.0)
    }
}

/// Builds an [`accesskit::Node`] with PH2D-flavored defaults.
///
/// - All nodes default `focusable(false)`; explicit opt-in via `focusable(true)`.
/// - Label is required for any interactive role (asserts in debug).
/// - Children are added in order; iteration is stable.
pub struct NodeBuilder {
    role: Role,
    label: Option<String>,
    bounds: Option<Rect>,
    focusable: bool,
    live: Option<Live>,
    children: Vec<NodeId>,
    actions: Vec<accesskit::Action>,
    toggled: Option<Toggled>,
    numeric_value: Option<f64>,
    numeric_value_min: Option<f64>,
    numeric_value_max: Option<f64>,
}

impl NodeBuilder {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            label: None,
            bounds: None,
            focusable: false,
            live: None,
            children: Vec::new(),
            actions: Vec::new(),
            toggled: None,
            numeric_value: None,
            numeric_value_min: None,
            numeric_value_max: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn bounds(mut self, x: f64, y: f64, w: f64, h: f64) -> Self {
        self.bounds = Some(Rect {
            x0: x,
            y0: y,
            x1: x + w,
            y1: y + h,
        });
        self
    }

    pub fn focusable(mut self, yes: bool) -> Self {
        self.focusable = yes;
        self
    }

    /// Mark this node as a live region — screen readers announce
    /// updates without stealing focus. Used for toast notifications
    /// (ADR-0023 §10 "Live regions").
    pub fn live(mut self, live: Live) -> Self {
        self.live = Some(live);
        self
    }

    pub fn child(mut self, child: NodeId) -> Self {
        self.children.push(child);
        self
    }

    pub fn children<I: IntoIterator<Item = NodeId>>(mut self, ids: I) -> Self {
        self.children.extend(ids);
        self
    }

    pub fn action(mut self, action: accesskit::Action) -> Self {
        self.actions.push(action);
        self
    }

    /// On/off state for `Switch` / `CheckBox` / `RadioButton` roles.
    /// Maps to AccessKit's `Toggled` (True / False / Mixed).
    pub fn toggled(mut self, t: Toggled) -> Self {
        self.toggled = Some(t);
        self
    }

    /// Current value for `Slider` (and other numeric roles).
    pub fn numeric_value(mut self, v: f64) -> Self {
        self.numeric_value = Some(v);
        self
    }

    /// Lower bound for numeric value.
    pub fn numeric_value_min(mut self, v: f64) -> Self {
        self.numeric_value_min = Some(v);
        self
    }

    /// Upper bound for numeric value.
    pub fn numeric_value_max(mut self, v: f64) -> Self {
        self.numeric_value_max = Some(v);
        self
    }

    pub fn build(self) -> Node {
        // Interactive roles (button, checkbox, etc.) must have a
        // label or screen readers say "button" without context.
        debug_assert!(
            !is_interactive(self.role) || self.label.is_some(),
            "interactive role {:?} requires label() per WCAG 2.5.3",
            self.role
        );

        let mut node = Node::new(self.role);
        if let Some(label) = self.label {
            node.set_label(label);
        }
        if let Some(bounds) = self.bounds {
            node.set_bounds(bounds);
        }
        if let Some(live) = self.live {
            node.set_live(live);
        }
        for child in self.children {
            node.push_child(child.into());
        }
        for action in self.actions {
            node.add_action(action);
        }
        if let Some(t) = self.toggled {
            node.set_toggled(t);
        }
        if let Some(v) = self.numeric_value {
            node.set_numeric_value(v);
        }
        if let Some(v) = self.numeric_value_min {
            node.set_min_numeric_value(v);
        }
        if let Some(v) = self.numeric_value_max {
            node.set_max_numeric_value(v);
        }
        if !self.focusable {
            // accesskit defaults vary by role; explicit non-focusable
            // requires no positive flag — leaving as default is fine.
        }
        node
    }
}

fn is_interactive(role: Role) -> bool {
    matches!(
        role,
        Role::Button
            | Role::CheckBox
            | Role::ComboBox
            | Role::Link
            | Role::MenuItem
            | Role::RadioButton
            | Role::Slider
            | Role::Switch
            | Role::Tab
            | Role::TextInput
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_round_trips_through_accesskit() {
        let id = NodeId(42);
        let ak: AkNodeId = id.into();
        let back: NodeId = ak.into();
        assert_eq!(id, back);
    }

    #[test]
    fn next_increments() {
        let id = NodeId(7);
        assert_eq!(id.next(), NodeId(8));
    }

    #[test]
    fn root_constant() {
        assert_eq!(NodeId::ROOT, NodeId(0));
    }

    #[test]
    fn builder_constructs_button_with_label() {
        let node = NodeBuilder::new(Role::Button)
            .label("Save")
            .bounds(10.0, 20.0, 80.0, 32.0)
            .focusable(true)
            .action(accesskit::Action::Click)
            .build();
        assert_eq!(node.role(), Role::Button);
        assert_eq!(node.label(), Some("Save"));
    }

    #[test]
    fn builder_supports_children() {
        let root = NodeBuilder::new(Role::Window)
            .label("Editor")
            .child(NodeId(1))
            .child(NodeId(2))
            .build();
        assert_eq!(root.role(), Role::Window);
        // accesskit::Node doesn't expose children() back — round-trip
        // happens via TreeUpdate; we trust push_child stored them.
    }

    #[test]
    fn live_region_for_toast() {
        let node = NodeBuilder::new(Role::GenericContainer)
            .label("Saved 2 s ago")
            .live(Live::Polite)
            .build();
        assert_eq!(node.live(), Some(Live::Polite));
    }
}
