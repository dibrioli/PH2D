//! [`Toggle`] — binary on/off switch (snap-to-grid, lock-axis, etc).
//!
//! Same pattern as [`crate::widget::Button`]: data + state enum +
//! token-resolved colors + AccessKit `Role::Switch` node + paint
//! helper colocated. Maps the boolean `on` to `Toggled::True/False`
//! so screen readers announce "on" / "off".

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role, Toggled};
use ph2d_tokens::{ColorToken, Radius, Theme};
use ph2d_vector::{Affine, Brush, Circle, Fill, Point, VectorScene};

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

/// Pill body + circular thumb (left=off, right=on). Focus ring is
/// drawn as a true stroked outer rounded rect — no more "re-fill
/// inside the ring" inversion hack.
pub fn paint_toggle(toggle: &Toggle, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Full.px();
    let body_token = match (toggle.on, toggle.state) {
        // Disabled + on stays on a muted accent so users can still
        // read the current value at a glance; disabled + off keeps
        // the neutral Border fill. Previously both collapsed to
        // Border which read as "off, locked" regardless of the
        // actual `on` flag.
        (true, ToggleState::Disabled) => ColorToken::AccentSoft,
        (false, ToggleState::Disabled) => ColorToken::Border,
        (true, _) => ColorToken::Accent,
        (false, _) => ColorToken::Bg2,
    };
    fill_rounded_rect(scene, rect, radius, resolve(body_token, theme));

    // Subtle border emphasis on Hover/Focus so the off-state pill
    // doesn't look static when the cursor is over it.
    if matches!(toggle.state, ToggleState::Hovered | ToggleState::Focused) {
        let stroke_w = if toggle.state == ToggleState::Focused {
            2.0
        } else {
            1.0
        };
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            stroke_w,
            resolve(ColorToken::BorderEmph, theme),
        );
    }

    // Circular thumb. Diameter = body height - 2*pad. Off → left,
    // On → right. Disabled tints to Text3 so it stays visible but mute.
    let pad = (rect.h * 0.15).clamp(2.0, 6.0);
    let diameter = (rect.h - pad * 2.0).max(4.0);
    let r = diameter * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let cx = if toggle.on {
        rect.x + rect.w - pad - r
    } else {
        rect.x + pad + r
    };
    let thumb_token = if toggle.state == ToggleState::Disabled {
        ColorToken::Text3
    } else if toggle.on {
        ColorToken::AccentFg
    } else {
        ColorToken::Text2
    };
    let thumb = Circle::new(Point::new(cx as f64, cy as f64), r as f64);
    scene.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(resolve(thumb_token, theme)),
        None,
        &thumb,
    );
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

    fn smoke(t: Toggle, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_toggle(&t, Rect::new(0.0, 0.0, 60.0, 24.0), &mut scene, theme);
    }

    #[test]
    fn paint_smoke_off() {
        smoke(Toggle::new(NodeId(1), "x"), Theme::Forge);
    }

    #[test]
    fn paint_smoke_on() {
        smoke(Toggle::new(NodeId(1), "x").on(true), Theme::Forge);
    }

    #[test]
    fn paint_smoke_hovered_on() {
        smoke(
            Toggle::new(NodeId(1), "x")
                .on(true)
                .state(ToggleState::Hovered),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_focused() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Focused),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_pressed() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Pressed),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            Toggle::new(NodeId(1), "x").state(ToggleState::Disabled),
            Theme::Forge,
        );
    }
}
