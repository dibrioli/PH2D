//! [`ColorSwatch`] — color preview chip.
//!
//! Pure data + a11y. Rendering is the shell's job (egui owns paint).

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SwatchState {
    #[default]
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct ColorSwatch {
    pub id: NodeId,
    pub label: String,
    pub rgba: [u8; 4],
    pub state: SwatchState,
}

impl ColorSwatch {
    pub fn new(id: NodeId, label: impl Into<String>, rgba: [u8; 4]) -> Self {
        Self {
            id,
            label: label.into(),
            rgba,
            state: SwatchState::Normal,
        }
    }

    /// AccessKit node — no native ColorSwatch role; Button with the
    /// hex appended so screen readers say something useful.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let label = format!(
            "{} (#{:02X}{:02X}{:02X})",
            self.label, self.rgba[0], self.rgba[1], self.rgba[2]
        );
        NodeBuilder::new(Role::Button)
            .label(label)
            .bounds(x, y, w, h)
            .focusable(self.state != SwatchState::Disabled)
            .action(Action::Click)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_normal_state() {
        let s = ColorSwatch::new(NodeId(1), "Color", [255, 0, 0, 255]);
        assert_eq!(s.state, SwatchState::Normal);
        assert_eq!(s.rgba, [255, 0, 0, 255]);
    }

    #[test]
    fn a11y_label_includes_hex() {
        let s = ColorSwatch::new(NodeId(1), "Color", [0xAB, 0xCD, 0xEF, 0xFF]);
        let n = s.build_a11y(0.0, 0.0, 40.0, 40.0);
        assert_eq!(n.label(), Some("Color (#ABCDEF)"));
    }
}
