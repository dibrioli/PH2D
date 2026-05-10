//! [`ColorSwatch`] — single color preview chip.
//!
//! Same pattern as [`crate::widget::Button`] for the data + state +
//! a11y skeleton, but the visual fill is the user's chosen RGBA — we
//! deliberately bypass the token system here. Tokens describe chrome;
//! the swatch IS the user's content.
//!
//! Variants by size: `Sm` (24), `Md` (32), `Lg` (48). When the
//! swatch's alpha < 255 we paint a 4x4 transparency checkerboard
//! beneath the fill so users can see what's translucent.

use crate::paint::{fill_rounded_rect, resolve, stroke_rounded_rect};
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, Theme};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SwatchSize {
    /// 24 px — palette grids.
    Sm,
    /// 32 px — inspector / panel default.
    #[default]
    Md,
    /// 48 px — hero swatches (current foreground/background pair).
    Lg,
}

impl SwatchSize {
    pub const fn px(self) -> f32 {
        match self {
            Self::Sm => 24.0,
            Self::Md => 32.0,
            Self::Lg => 48.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorSwatch {
    pub id: NodeId,
    pub label: String,
    /// RGBA bytes of the swatch fill — the user's chosen color.
    pub rgba: [u8; 4],
    pub state: SwatchState,
    pub size: SwatchSize,
}

impl ColorSwatch {
    pub fn new(id: NodeId, label: impl Into<String>, rgba: [u8; 4]) -> Self {
        Self {
            id,
            label: label.into(),
            rgba,
            state: SwatchState::Normal,
            size: SwatchSize::Md,
        }
    }

    pub fn state(mut self, state: SwatchState) -> Self {
        self.state = state;
        self
    }

    pub fn size(mut self, size: SwatchSize) -> Self {
        self.size = size;
        self
    }

    /// Suggested edge length for this swatch in DIPs (driven by
    /// `size`). Callers may still hand any rect to [`paint_color_swatch`].
    pub fn px(&self) -> f32 {
        self.size.px()
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

const CHECKER_CELL_PX: f32 = 4.0;
const CHECKER_LIGHT: VelloColor = VelloColor::from_rgba8(220, 220, 220, 255);
const CHECKER_DARK: VelloColor = VelloColor::from_rgba8(170, 170, 170, 255);

/// Fill rect with a small checkerboard pattern. Used to communicate
/// translucency under low-alpha fills.
fn paint_checker(scene: &mut VectorScene, rect: Rect) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let cols = (rect.w / CHECKER_CELL_PX).ceil() as i32;
    let rows = (rect.h / CHECKER_CELL_PX).ceil() as i32;
    for row in 0..rows {
        for col in 0..cols {
            let dark = (row + col) % 2 == 0;
            let cx = rect.x + col as f32 * CHECKER_CELL_PX;
            let cy = rect.y + row as f32 * CHECKER_CELL_PX;
            let cw = CHECKER_CELL_PX.min(rect.x + rect.w - cx);
            let ch = CHECKER_CELL_PX.min(rect.y + rect.h - cy);
            if cw <= 0.0 || ch <= 0.0 {
                continue;
            }
            let cell = Rect::new(cx, cy, cw, ch);
            let color = if dark { CHECKER_DARK } else { CHECKER_LIGHT };
            // Sharp fills (no rounding) for the checker cells —
            // rounding the inside of a clipped rounded rect needs
            // clip stacks we can add later if anyone notices.
            scene.fill_rect(crate::paint::rect_to_vello(cell), color);
        }
    }
}

/// Rounded outer frame + transparency checker (when alpha < 255) +
/// the swatch's actual RGBA on top. Focused state swaps the frame
/// from `Border` to `BorderEmph` (3:1 contrast vs. Bg1).
pub fn paint_color_swatch(swatch: &ColorSwatch, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    let radius = Radius::Sm.px();
    let border_token = if swatch.state == SwatchState::Focused {
        ColorToken::BorderEmph
    } else {
        ColorToken::Border
    };
    fill_rounded_rect(scene, rect, radius, resolve(border_token, theme));

    let pad = 2.0_f32.min(rect.w * 0.5).min(rect.h * 0.5);
    let inner = Rect::new(
        rect.x + pad,
        rect.y + pad,
        (rect.w - 2.0 * pad).max(0.0),
        (rect.h - 2.0 * pad).max(0.0),
    );
    let inner_radius = (radius - pad).max(0.0);

    if swatch.rgba[3] < 255 {
        paint_checker(scene, inner);
    }
    let [r, g, b, a] = swatch.rgba;
    fill_rounded_rect(
        scene,
        inner,
        inner_radius,
        VelloColor::from_rgba8(r, g, b, a),
    );

    if swatch.state == SwatchState::Hovered {
        stroke_rounded_rect(
            scene,
            rect,
            radius,
            1.0,
            resolve(ColorToken::AccentSoft, theme),
        );
    }
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
        assert_eq!(s.size, SwatchSize::Md);
        assert_eq!(s.px(), 32.0);
    }

    #[test]
    fn size_chain_changes_px() {
        assert_eq!(
            ColorSwatch::new(NodeId(1), "x", [0; 4])
                .size(SwatchSize::Sm)
                .px(),
            24.0
        );
        assert_eq!(
            ColorSwatch::new(NodeId(1), "x", [0; 4])
                .size(SwatchSize::Lg)
                .px(),
            48.0
        );
    }

    #[test]
    fn a11y_node_uses_color_well_role() {
        let s = ColorSwatch::new(NodeId(1), "Foreground", [255, 0, 0, 255]);
        let node = s.build_a11y(0.0, 0.0, 32.0, 32.0);
        assert_eq!(node.role(), Role::ColorWell);
        assert_eq!(node.label(), Some("Foreground"));
    }

    fn smoke(swatch: ColorSwatch, rect: Rect, theme: Theme) {
        let mut scene = VectorScene::new();
        paint_color_swatch(&swatch, rect, &mut scene, theme);
    }

    #[test]
    fn paint_smoke_sm_opaque() {
        smoke(
            ColorSwatch::new(NodeId(1), "x", [200, 100, 50, 255]).size(SwatchSize::Sm),
            Rect::new(0.0, 0.0, 24.0, 24.0),
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_md_alpha_zero_paints_checker() {
        smoke(
            ColorSwatch::new(NodeId(1), "x", [0, 0, 0, 0]),
            Rect::new(0.0, 0.0, 32.0, 32.0),
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_lg_alpha_half() {
        smoke(
            ColorSwatch::new(NodeId(1), "x", [50, 200, 100, 128]).size(SwatchSize::Lg),
            Rect::new(0.0, 0.0, 48.0, 48.0),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_hovered_outlines() {
        smoke(
            ColorSwatch::new(NodeId(1), "x", [10, 10, 10, 255]).state(SwatchState::Hovered),
            Rect::new(0.0, 0.0, 32.0, 32.0),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_focused_emphasizes_border() {
        smoke(
            ColorSwatch::new(NodeId(1), "x", [255, 0, 0, 255]).state(SwatchState::Focused),
            Rect::new(0.0, 0.0, 32.0, 32.0),
            Theme::PaintStudio,
        );
    }
}
