//! [`Slider`] — continuous numeric input (0..=1 normalized).
//!
//! Pure data + a11y. Rendering is the shell's job (egui owns widget
//! paint after the egui-migration pivot).

use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SliderState {
    #[default]
    Normal,
    Hovered,
    Dragging,
    Focused,
    Disabled,
}

#[derive(Clone, Debug)]
pub struct Slider {
    pub id: NodeId,
    pub label: String,
    /// Normalized 0..=1; UI maps this to whatever the binding wants.
    pub value: f32,
    pub state: SliderState,
    /// True ⇒ filled bar uses `AccentPrimary` (the "active modulator"
    /// look); false ⇒ `AccentSecondary` (a dim default).
    pub accent: bool,
}

impl Slider {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: 0.5,
            state: SliderState::Normal,
            accent: false,
        }
    }

    pub fn accent(mut self, yes: bool) -> Self {
        self.accent = yes;
        self
    }

    pub fn state(mut self, state: SliderState) -> Self {
        self.state = state;
        self
    }

    /// Clamp + assign. Value is held in [0, 1].
    pub fn set_value(&mut self, v: f32) {
        self.value = v.clamp(0.0, 1.0);
    }

    /// AccessKit node — Role::Slider with NumericValue + Min/Max so
    /// screen readers announce "30 percent" without us spelling it out.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::Slider)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != SliderState::Disabled)
            .action(Action::Click)
            .numeric_value(self.value as f64)
            .numeric_value_min(0.0)
            .numeric_value_max(1.0)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_mid_value_normal_non_accent() {
        let s = Slider::new(NodeId(1), "Size");
        assert!((s.value - 0.5).abs() < 1e-6);
        assert_eq!(s.state, SliderState::Normal);
        assert!(!s.accent);
    }

    #[test]
    fn set_value_clamps_to_unit_range() {
        let mut s = Slider::new(NodeId(1), "X");
        s.set_value(2.0);
        assert!((s.value - 1.0).abs() < 1e-6);
        s.set_value(-0.5);
        assert!(s.value == 0.0);
    }

    #[test]
    fn a11y_has_slider_role_and_numeric_value() {
        let s = Slider::new(NodeId(7), "Opacity");
        let n = s.build_a11y(0.0, 0.0, 100.0, 30.0);
        assert_eq!(n.role(), Role::Slider);
        assert_eq!(n.label(), Some("Opacity"));
        assert_eq!(n.numeric_value(), Some(0.5));
    }
}
