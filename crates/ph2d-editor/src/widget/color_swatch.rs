//! [`ColorSwatch`] — single color preview chip.
//!
//! Same pattern as [`crate::widget::Button`] for the data + state +
//! a11y skeleton, but the visual fill is the user's chosen RGBA — we
//! deliberately bypass the token system here. Tokens describe chrome;
//! the swatch IS the user's content.

use crate::paint::{rect_to_vello, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::{Color as VelloColor, VectorScene};

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
    /// RGBA bytes of the swatch fill — the user's chosen color.
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

    pub fn state(mut self, state: SwatchState) -> Self {
        self.state = state;
        self
    }

    /// Build the AccessKit node. `Role::ColorWell` is the AccessKit
    /// canonical for "color picker swatch"; falls back to a labeled
    /// clickable so screen readers say "color, blue" or similar.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        NodeBuilder::new(Role::ColorWell)
            .label(&self.label)
            .bounds(x, y, w, h)
            .focusable(self.state != SwatchState::Disabled)
            .action(Action::Click)
            .build()
    }
}

/// 2-px border (`BorderEmphasis` if Focused else `Border`) framing
/// an inner rect filled with the swatch's actual RGBA — token system
/// is bypassed for the inner fill since it's user content, not chrome.
pub fn paint_color_swatch(swatch: &ColorSwatch, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    // 1. Outer frame — two-pixel border via outer fill + inner inset.
    let border_token = if swatch.state == SwatchState::Focused {
        ColorToken::BorderEmph
    } else {
        ColorToken::Border
    };
    scene.fill_rect(rect_to_vello(rect), resolve(border_token, theme));

    // 2. Inner fill: the actual RGBA color, 2 px inset.
    let pad = 2.0_f32.min(rect.w * 0.5).min(rect.h * 0.5);
    let inner = Rect::new(
        rect.x + pad,
        rect.y + pad,
        (rect.w - 2.0 * pad).max(0.0),
        (rect.h - 2.0 * pad).max(0.0),
    );
    let [r, g, b, a] = swatch.rgba;
    scene.fill_rect(rect_to_vello(inner), VelloColor::from_rgba8(r, g, b, a));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        let s = ColorSwatch::new(NodeId(1), "Brush color", [10, 20, 30, 255]);
        assert_eq!(s.id, NodeId(1));
        assert_eq!(s.label, "Brush color");
        assert_eq!(s.rgba, [10, 20, 30, 255]);
        assert_eq!(s.state, SwatchState::Normal);
    }

    #[test]
    fn state_chain_works() {
        let s = ColorSwatch::new(NodeId(1), "x", [0, 0, 0, 255]).state(SwatchState::Focused);
        assert_eq!(s.state, SwatchState::Focused);
    }

    #[test]
    fn a11y_node_uses_color_well_role() {
        let s = ColorSwatch::new(NodeId(1), "Foreground", [255, 0, 0, 255]);
        let node = s.build_a11y(0.0, 0.0, 32.0, 32.0);
        assert_eq!(node.role(), Role::ColorWell);
        assert_eq!(node.label(), Some("Foreground"));
    }

    #[test]
    fn paint_smoke_default() {
        let s = ColorSwatch::new(NodeId(1), "x", [128, 64, 200, 255]);
        let mut scene = VectorScene::new();
        paint_color_swatch(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_focused_emphasizes_border() {
        let s = ColorSwatch::new(NodeId(1), "x", [0, 255, 0, 255]).state(SwatchState::Focused);
        let mut scene = VectorScene::new();
        paint_color_swatch(
            &s,
            Rect::new(0.0, 0.0, 100.0, 30.0),
            &mut scene,
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_alpha_translucent() {
        let s = ColorSwatch::new(NodeId(1), "x", [255, 0, 0, 64]);
        let mut scene = VectorScene::new();
        paint_color_swatch(
            &s,
            Rect::new(0.0, 0.0, 50.0, 50.0),
            &mut scene,
            Theme::ForgeSdf,
        );
    }
}
