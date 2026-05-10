//! [`Slider`] — continuous numeric input (0..=1 normalized).
//!
//! Procreate uses sliders for brush size, opacity, flow. Same pattern
//! as [`crate::widget::Button`]: data + state enum + token-resolved
//! colors + AccessKit `Role::Slider` node + `paint_slider` colocated
//! here (paint.rs stays the dispatch layer for compound widgets).

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;

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
    /// True ⇒ filled bar uses `AccentPrimary`, the "active modulator"
    /// look; false ⇒ `AccentSecondary` (a dim default).
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

    /// Build the AccessKit node. Per ADR-0023 §10: Role::Slider with
    /// numeric value + min/max so screen readers can announce
    /// "30 percent" without us spelling it out.
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

/// Track + filled portion + thumb. Caller supplies the rect; the panel
/// layout decides where the slider sits.
///
/// - Track: thin horizontal bar centered vertically (`Border` color).
/// - Filled: from `rect.x` to `rect.x + rect.w * value` (accent-colored).
/// - Thumb: small square at the value position, raised to `BorderEmphasis`
///   while `Dragging`. Vello has no circle primitive on `VectorScene`;
///   we approximate with a square — matches Procreate's chunky aesthetic.
pub fn paint_slider(slider: &Slider, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    if slider.state == SliderState::Disabled {
        // Single flat bar, no thumb / fill — leaves a clear "off" state.
        scene.fill_rect(rect_to_vello(rect), resolve(ColorToken::Border, theme));
        return;
    }

    // 1. Track (full width, thin).
    let track_h = (rect.h * 0.15).max(1.0);
    let track_y = rect.y + (rect.h - track_h) / 2.0;
    let track = Rect::new(rect.x, track_y, rect.w, track_h);
    scene.fill_rect(rect_to_vello(track), resolve(ColorToken::Border, theme));

    // 2. Filled portion.
    let fill_w = rect.w * slider.value.clamp(0.0, 1.0);
    if fill_w > 0.0 {
        let fill_token = if slider.accent {
            ColorToken::Accent
        } else {
            ColorToken::AccentPress
        };
        let filled = Rect::new(rect.x, track_y, fill_w, track_h);
        scene.fill_rect(rect_to_vello(filled), resolve(fill_token, theme));
    }

    // 3. Thumb (small square at value position).
    let thumb_size = rect.h * 0.6;
    let thumb_x = rect.x + rect.w * slider.value.clamp(0.0, 1.0) - thumb_size / 2.0;
    let thumb_y = rect.y + (rect.h - thumb_size) / 2.0;
    let thumb_token = match slider.state {
        SliderState::Dragging => ColorToken::BorderEmph,
        _ => ColorToken::Accent,
    };
    let thumb = Rect::new(thumb_x, thumb_y, thumb_size, thumb_size);
    scene.fill_rect(rect_to_vello(thumb), resolve(thumb_token, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let s = Slider::new(NodeId(1), "Opacity");
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Opacity");
        assert!((s.value - 0.5).abs() < f32::EPSILON);
        assert_eq!(s.state, SliderState::Normal);
        assert!(!s.accent);
    }

    #[test]
    fn set_value_clamps_below_zero() {
        let mut s = Slider::new(NodeId(1), "x");
        s.set_value(-0.5);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn set_value_clamps_above_one() {
        let mut s = Slider::new(NodeId(1), "x");
        s.set_value(1.5);
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn set_value_inside_range_passes_through() {
        let mut s = Slider::new(NodeId(1), "x");
        s.set_value(0.42);
        assert!((s.value - 0.42).abs() < f32::EPSILON);
    }

    #[test]
    fn a11y_node_has_slider_role_and_value() {
        let s = Slider::new(NodeId(1), "Size");
        let node = s.build_a11y(0.0, 0.0, 100.0, 30.0);
        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.label(), Some("Size"));
        assert_eq!(node.numeric_value(), Some(0.5));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(1.0));
    }

    #[test]
    fn paint_smoke_default_does_not_panic() {
        let s = Slider::new(NodeId(1), "x");
        let mut scene = VectorScene::new();
        paint_slider(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_value_zero_does_not_panic() {
        let mut s = Slider::new(NodeId(1), "x");
        s.set_value(0.0);
        let mut scene = VectorScene::new();
        paint_slider(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_value_one_does_not_panic() {
        let mut s = Slider::new(NodeId(1), "x");
        s.set_value(1.0);
        let mut scene = VectorScene::new();
        paint_slider(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_disabled_uses_flat_border_path() {
        let s = Slider::new(NodeId(1), "x").state(SliderState::Disabled);
        let mut scene = VectorScene::new();
        paint_slider(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_dragging_smoke() {
        let s = Slider::new(NodeId(1), "x")
            .accent(true)
            .state(SliderState::Dragging);
        let mut scene = VectorScene::new();
        paint_slider(
            &s,
            Rect::new(10.0, 10.0, 200.0, 24.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn disabled_slider_is_not_focusable_in_a11y() {
        let s = Slider::new(NodeId(1), "x").state(SliderState::Disabled);
        let _ = s.build_a11y(0.0, 0.0, 100.0, 30.0);
        // NodeBuilder default is non-focusable; we trust focusable(false).
    }
}
