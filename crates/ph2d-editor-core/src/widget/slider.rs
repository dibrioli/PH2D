//! [`Slider`] — continuous numeric input (0..=1 normalized).
//!
//! Procreate uses sliders for brush size, opacity, flow. Same pattern
//! as [`crate::widget::Button`]: data + state enum + token-resolved
//! colors + AccessKit `Role::Slider` node + `paint_slider` colocated.
//! Supports horizontal and vertical orientation and an optional set
//! of tick positions for snap-to-grid affordance.

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, StrokeToken, Theme};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug)]
pub struct Slider {
    pub id: NodeId,
    pub label: String,
    /// Normalized 0..=1; UI maps this to whatever the binding wants.
    pub value: f32,
    pub state: SliderState,
    pub orientation: SliderOrientation,
    /// True ⇒ filled bar uses `Accent`; false ⇒ `AccentPress` (a dim
    /// default). Distinct from focus highlight.
    pub accent: bool,
    /// Optional snap positions in [0, 1]. Painted as small marks on
    /// the track. Empty means no ticks.
    pub ticks: Vec<f32>,
}

impl Slider {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            value: 0.5,
            state: SliderState::Normal,
            orientation: SliderOrientation::Horizontal,
            accent: false,
            ticks: Vec::new(),
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

    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn ticks(mut self, ticks: impl Into<Vec<f32>>) -> Self {
        self.ticks = ticks.into();
        self
    }

    /// Clamp + assign. Value is held in [0, 1].
    pub fn set_value(&mut self, v: f32) {
        self.value = v.clamp(0.0, 1.0);
    }

    /// Build the AccessKit node. Per ADR-0023 §10: `Role::Slider`
    /// with numeric value + min/max so screen readers can announce
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

/// Canonical slider track: rounded-rect background (`Bg2`) + an
/// `Accent`-filled portion for the current value. **Single source of
/// truth for the slider look** — both the bare [`paint_slider`] and
/// `widget::slider_with_chip::paint_slider_with_chip` render through
/// this, so every slider in the app shares one rectangular appearance.
pub fn paint_slider_track(
    track: Rect,
    value: f32,
    orientation: SliderOrientation,
    scene: &mut VectorScene,
    theme: Theme,
) {
    let r = Radius::Xs.px();
    fill_rounded_rect(scene, track, r, resolve(ColorToken::Bg2, theme));
    let v = value.clamp(0.0, 1.0);
    if v > 0.0 {
        let filled = match orientation {
            SliderOrientation::Horizontal => Rect::new(track.x, track.y, track.w * v, track.h),
            SliderOrientation::Vertical => {
                let h = track.h * v;
                Rect::new(track.x, track.y + track.h - h, track.w, h)
            }
        };
        fill_rounded_rect(scene, filled, r, resolve(ColorToken::Accent, theme));
    }
}

/// Rectangular track + accent fill (no circular thumb — the filled
/// portion is the value readout, matching `paint_slider_with_chip`).
/// `Focused` adds a `BorderEmph` outline on the track; `Disabled`
/// draws a flat `Border` track.
pub fn paint_slider(slider: &Slider, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let track = track_rect(slider.orientation, rect);
    let r = Radius::Xs.px();
    if slider.state == SliderState::Disabled {
        fill_rounded_rect(scene, track, r, resolve(ColorToken::Border, theme));
        return;
    }

    paint_slider_track(track, slider.value, slider.orientation, scene, theme);

    for tick in &slider.ticks {
        let pos = tick.clamp(0.0, 1.0);
        let mark = tick_mark_rect(slider.orientation, rect, pos);
        fill_rounded_rect(scene, mark, r, resolve(ColorToken::Text3, theme));
    }

    if slider.state == SliderState::Focused {
        stroke_rounded_rect(
            scene,
            track,
            r,
            StrokeToken::Default.px(),
            resolve(ColorToken::BorderEmph, theme),
        );
    }
}

fn track_rect(orientation: SliderOrientation, rect: Rect) -> Rect {
    match orientation {
        SliderOrientation::Horizontal => {
            let h = (rect.h * 0.25).clamp(2.0, 8.0); // LITERAL-PX-OK: slider track sized as 25% of widget height (geometry ratio + clamp)
            let y = rect.y + (rect.h - h) / 2.0;
            Rect::new(rect.x, y, rect.w, h)
        }
        SliderOrientation::Vertical => {
            let w = (rect.w * 0.25).clamp(2.0, 8.0); // LITERAL-PX-OK: slider track sized as 25% of widget width (geometry ratio + clamp)
            let x = rect.x + (rect.w - w) / 2.0;
            Rect::new(x, rect.y, w, rect.h)
        }
    }
}

fn tick_mark_rect(orientation: SliderOrientation, rect: Rect, value: f32) -> Rect {
    match orientation {
        SliderOrientation::Horizontal => {
            let x = rect.x + rect.w * value - 1.0;
            let h = (rect.h * 0.5).max(4.0); // LITERAL-PX-OK: tick mark minimum height
            let y = rect.y + (rect.h - h) / 2.0;
            Rect::new(x, y, 2.0, h)
        }
        SliderOrientation::Vertical => {
            let y = rect.y + rect.h - rect.h * value - 1.0;
            let w = (rect.w * 0.5).max(4.0); // LITERAL-PX-OK: tick mark minimum width
            let x = rect.x + (rect.w - w) / 2.0;
            Rect::new(x, y, w, 2.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Slider {
        Slider::new(NodeId(1), "Opacity")
    }

    #[test]
    fn defaults_match_spec() {
        let s = fixture();
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Opacity");
        assert!((s.value - 0.5).abs() < f32::EPSILON);
        assert_eq!(s.state, SliderState::Normal);
        assert_eq!(s.orientation, SliderOrientation::Horizontal);
        assert!(!s.accent);
        assert!(s.ticks.is_empty());
    }

    #[test]
    fn set_value_clamps_below_zero() {
        let mut s = fixture();
        s.set_value(-0.5);
        assert_eq!(s.value, 0.0);
    }

    #[test]
    fn set_value_clamps_above_one() {
        let mut s = fixture();
        s.set_value(1.5);
        assert_eq!(s.value, 1.0);
    }

    #[test]
    fn ticks_setter_round_trips() {
        let s = fixture().ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(s.ticks.len(), 5);
    }

    #[test]
    fn a11y_node_has_slider_role_and_value() {
        let s = fixture();
        let node = s.build_a11y(0.0, 0.0, 100.0, 30.0);
        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.label(), Some("Opacity"));
        assert_eq!(node.numeric_value(), Some(0.5));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(1.0));
    }

    fn smoke(slider: Slider, rect: Rect, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_slider(&slider, rect, &mut scene, theme);
    }

    #[test]
    fn paint_smoke_horizontal_default() {
        smoke(fixture(), Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_horizontal_zero() {
        let mut s = fixture();
        s.set_value(0.0);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_horizontal_one() {
        let mut s = fixture();
        s.set_value(1.0);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Sunstone);
    }

    #[test]
    fn paint_smoke_vertical_half() {
        smoke(
            fixture().orientation(SliderOrientation::Vertical),
            Rect::new(0.0, 0.0, 24.0, 200.0),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_dragging_with_ticks() {
        let s = fixture()
            .accent(true)
            .ticks(vec![0.0, 0.25, 0.5, 0.75, 1.0])
            .state(SliderState::Dragging);
        smoke(s, Rect::new(0.0, 0.0, 200.0, 24.0), Theme::Forge);
    }

    #[test]
    fn paint_smoke_focused_draws_ring() {
        smoke(
            fixture().state(SliderState::Focused),
            Rect::new(0.0, 0.0, 200.0, 24.0),
            Theme::Workshop,
        );
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(
            fixture().state(SliderState::Disabled),
            Rect::new(0.0, 0.0, 200.0, 24.0),
            Theme::Forge,
        );
    }
}
