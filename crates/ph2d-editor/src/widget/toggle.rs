//! [`Toggle`] — binary on/off pill.
//!
//! Pure data + a11y. Rendering is the shell's job (egui owns paint).

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ToggleState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Toggle {
    pub id: NodeId,
    pub label: String,
    pub on: bool,
    pub state: ToggleState,
}

impl Toggle {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            on: false,
            state: ToggleState::Normal,
        }
    }

    pub fn on(mut self, yes: bool) -> Self {
        self.on = yes;
        self
    }

    /// AccessKit node — Role::Switch with Toggled true/false.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Switch)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != ToggleState::Disabled)
            .action(Action::Click)
            .toggled(if self.on {
                Toggled::True
            } else {
                Toggled::False
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_off_normal() {
        let t = Toggle::new(NodeId(1), "Snap");
        assert!(!t.on);
        assert_eq!(t.state, ToggleState::Normal);
    }

    #[test]
    fn a11y_has_switch_role_and_toggled_state() {
        let t = Toggle::new(NodeId(2), "Snap").on(true);
        let n = t.build_a11y(0.0, 0.0, 60.0, 30.0);
        assert_eq!(n.role(), Role::Switch);
        assert_eq!(n.label(), Some("Snap"));
        assert_eq!(n.toggled(), Some(Toggled::True));
    }
}
