//! [`RadioGroup`] — single-select among options.
//!
//! Pure data + a11y. Rendering is the shell's job (egui owns paint).

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum RadioOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct RadioOption<T: Clone + PartialEq> {
    pub value: T,
    pub label: String,
    pub id: NodeId,
}

#[derive(Clone, Debug)]
pub struct RadioGroup<T: Clone + PartialEq> {
    pub id: NodeId,
    pub label: String,
    pub options: Vec<RadioOption<T>>,
    pub selected: Option<T>,
    pub orientation: RadioOrientation,
}

impl<T: Clone + PartialEq> RadioGroup<T> {
    pub fn new(id: NodeId, label: impl Into<String>, options: Vec<RadioOption<T>>) -> Self {
        Self {
            id,
            label: label.into(),
            options,
            selected: None,
            orientation: RadioOrientation::Horizontal,
        }
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = RadioOrientation::Vertical;
        self
    }

    /// Select the option whose value equals `v`. No-op if absent.
    pub fn select(&mut self, v: T) {
        if self.options.iter().any(|o| o.value == v) {
            self.selected = Some(v);
        }
    }

    /// Container + per-option AccessKit nodes. Caller flattens the
    /// vec into the Tree.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Vec<Node> {
        let container = NodeBuilder::new(Role::RadioGroup)
            .label(&self.label)
            .bounds(x, y, w, h)
            .children(self.options.iter().map(|o| o.id))
            .build();
        let mut out = vec![container];
        for opt in &self.options {
            let toggled = if Some(&opt.value) == self.selected.as_ref() {
                Toggled::True
            } else {
                Toggled::False
            };
            out.push(
                NodeBuilder::new(Role::RadioButton)
                    .label(&opt.label)
                    .focusable(true)
                    .action(Action::Click)
                    .toggled(toggled)
                    .build(),
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_demo() -> RadioGroup<&'static str> {
        RadioGroup::new(
            NodeId(1),
            "Lock Axis",
            vec![
                RadioOption {
                    value: "none",
                    label: "None".into(),
                    id: NodeId(2),
                },
                RadioOption {
                    value: "x",
                    label: "X".into(),
                    id: NodeId(3),
                },
                RadioOption {
                    value: "y",
                    label: "Y".into(),
                    id: NodeId(4),
                },
            ],
        )
    }

    #[test]
    fn select_only_accepts_listed_values() {
        let mut g = build_demo();
        g.select("z");
        assert_eq!(g.selected, None);
        g.select("x");
        assert_eq!(g.selected, Some("x"));
    }

    #[test]
    fn a11y_emits_container_plus_one_per_option() {
        let g = build_demo();
        let nodes = g.build_a11y(0.0, 0.0, 100.0, 30.0);
        assert_eq!(nodes.len(), 4); // container + 3 options
        assert_eq!(nodes[0].role(), Role::RadioGroup);
        assert_eq!(nodes[1].role(), Role::RadioButton);
    }
}
