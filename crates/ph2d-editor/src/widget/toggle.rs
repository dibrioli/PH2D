//! [`Toggle`] — binary on/off switch (snap-to-grid, lock-axis, etc).
//!
//! Same pattern as [`crate::widget::Button`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Switch` node + paint
//! helper colocated. Maps the boolean `on` to `Toggled::True/False`
//! so screen readers announce "on" / "off".

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

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

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn state(mut self, state: ToggleState) -> Self {
        self.state = state;
        self
    }

    /// Flip the current state. Convenience for click handlers that
    /// don't know or care about the prior value.
    pub fn toggle(&mut self) {
        self.on = !self.on;
    }

    /// Build the AccessKit node. `Role::Switch` is the AccessKit
    /// canonical for binary on/off (vs. CheckBox which is tri-state).
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

/// Pill body + thumb (left=off, right=on).
///
/// - Outer: `SurfaceElevated` so the toggle visually lifts off the
///   panel surface; the focus ring lands as `BorderEmphasis` overlay
///   on a Focused state (drawn as a thin inner border ring fall-back
///   = filled rect 2 px in from the outer pill).
/// - Inner thumb: ~40 % of the rect width, square approximation of
///   a circle. `AccentPrimary` when `on`, `Border` when off.
pub fn paint_toggle(toggle: &Toggle, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    // 1. Outer pill body.
    let body_token = if toggle.state == ToggleState::Disabled {
        ColorToken::Border
    } else {
        ColorToken::BgElev
    };
    scene.fill_rect(rect_to_vello(rect), resolve(body_token, theme));

    // 2. Focus ring fall-back: paint a slightly inset overlay using
    //    BorderEmphasis. We don't have a stroke API on VectorScene
    //    helpers, so a thin border-colored frame stands in.
    if toggle.state == ToggleState::Focused {
        // 2 px inset.
        let pad = 2.0_f32.min(rect.w * 0.1).min(rect.h * 0.1);
        let inner = Rect::new(
            rect.x + pad,
            rect.y + pad,
            (rect.w - 2.0 * pad).max(0.0),
            (rect.h - 2.0 * pad).max(0.0),
        );
        scene.fill_rect(rect_to_vello(inner), resolve(ColorToken::BorderEmph, theme));
        // Re-fill the body inside the ring so the ring reads as a frame.
        let pad2 = pad + 2.0;
        let inner2 = Rect::new(
            rect.x + pad2,
            rect.y + pad2,
            (rect.w - 2.0 * pad2).max(0.0),
            (rect.h - 2.0 * pad2).max(0.0),
        );
        scene.fill_rect(rect_to_vello(inner2), resolve(body_token, theme));
    }

    // 3. Thumb. Square ≈ 40 % of width, vertically centered.
    let thumb_w = (rect.w * 0.4).min(rect.h - 4.0).max(2.0);
    let thumb_y = rect.y + (rect.h - thumb_w) / 2.0;
    let thumb_x = if toggle.on {
        rect.x + rect.w - thumb_w - 2.0
    } else {
        rect.x + 2.0
    };
    let thumb_token = if toggle.state == ToggleState::Disabled {
        ColorToken::TextDisabled
    } else if toggle.on {
        ColorToken::Accent
    } else {
        ColorToken::Border
    };
    let thumb = Rect::new(thumb_x, thumb_y, thumb_w, thumb_w);
    scene.fill_rect(rect_to_vello(thumb), resolve(thumb_token, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let t = Toggle::new(NodeId(1), "Snap");
        assert_eq!(t.id, NodeId(1));
        assert_eq!(t.label, "Snap");
        assert!(!t.on);
        assert_eq!(t.state, ToggleState::Normal);
    }

    #[test]
    fn toggle_flips_state() {
        let mut t = Toggle::new(NodeId(1), "x");
        assert!(!t.on);
        t.toggle();
        assert!(t.on);
        t.toggle();
        assert!(!t.on);
    }

    #[test]
    fn builder_chain_sets_on() {
        let t = Toggle::new(NodeId(1), "x").on(true);
        assert!(t.on);
    }

    #[test]
    fn a11y_node_has_switch_role_and_toggled_off() {
        let t = Toggle::new(NodeId(1), "Snap");
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.role(), Role::Switch);
        assert_eq!(node.label(), Some("Snap"));
        assert_eq!(node.toggled(), Some(Toggled::False));
    }

    #[test]
    fn a11y_node_toggled_true_when_on() {
        let t = Toggle::new(NodeId(1), "Snap").on(true);
        let node = t.build_a11y(0.0, 0.0, 60.0, 24.0);
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    #[test]
    fn paint_smoke_off() {
        let t = Toggle::new(NodeId(1), "x");
        let mut scene = VectorScene::new();
        paint_toggle(
            &t,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_on_focused() {
        let t = Toggle::new(NodeId(1), "x")
            .on(true)
            .state(ToggleState::Focused);
        let mut scene = VectorScene::new();
        paint_toggle(
            &t,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        let t = Toggle::new(NodeId(1), "x").state(ToggleState::Disabled);
        let mut scene = VectorScene::new();
        paint_toggle(
            &t,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }
}
