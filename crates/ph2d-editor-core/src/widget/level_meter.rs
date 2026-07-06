//! [`LevelMeter`] — a vertical stereo peak-level meter with green/amber/red
//! zones.
//!
//! Non-interactive (output-only, like [`crate::widget::ProgressBar`]). The
//! caller feeds already-ballistics-processed levels (peak-hold / decay live on
//! the caller side, e.g. the mixer bridge); the widget just paints the bars,
//! filling bottom-to-top. Zones use the semantic `Success → Warn → Danger`
//! tokens so the meter reads correctly in every theme (no literal colours).

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Fraction of full scale where the meter turns Success → Warn (≈ −6 dBFS).
const ZONE_WARN: f32 = 0.5; // LITERAL-PX-OK: meter zone threshold (amplitude fraction, not a UI dimension)
/// Fraction of full scale where the meter turns Warn → Danger (≈ −2 dBFS).
const ZONE_DANGER: f32 = 0.8; // LITERAL-PX-OK: meter zone threshold (amplitude fraction, not a UI dimension)

/// A stereo (or mono) peak-level meter.
#[derive(Clone, Debug)]
pub struct LevelMeter {
    pub id: NodeId,
    pub label: String,
    /// Per-channel levels `[left, right]`, each `0.0..` (values > 1.0 clip).
    pub levels: [f32; 2],
    /// When false, paints a single (mono) bar from `levels[0]`.
    pub stereo: bool,
}

impl LevelMeter {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            levels: [0.0, 0.0],
            stereo: true,
        }
    }

    /// Set stereo levels.
    pub fn levels(mut self, left: f32, right: f32) -> Self {
        self.levels = [left, right];
        self
    }

    /// Single-bar mono meter at `level`.
    pub fn mono(mut self, level: f32) -> Self {
        self.levels = [level, level];
        self.stereo = false;
        self
    }

    /// AccessKit node — a progress-indicator surface reporting the louder
    /// channel's level (`0..1`).
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let peak = self.levels[0].max(self.levels[1]).clamp(0.0, 1.0);
        NodeBuilder::new(Role::ProgressIndicator)
            .label(&self.label)
            .bounds(x, y, w, h)
            .numeric_value(peak as f64)
            .numeric_value_min(0.0)
            .numeric_value_max(1.0)
            .build()
    }
}

/// Paint the meter inside `rect`, filling bottom-to-top. Stereo → two columns.
pub fn paint_level_meter(meter: &LevelMeter, rect: Rect, scene: &mut VectorScene, theme: Theme) {
    if meter.stereo {
        let gap = Spacing::Xs.px();
        let col_w = ((rect.w - gap) * 0.5).max(1.0);
        let left = Rect::new(rect.x, rect.y, col_w, rect.h);
        let right = Rect::new(rect.x + col_w + gap, rect.y, col_w, rect.h);
        paint_column(scene, left, meter.levels[0], theme);
        paint_column(scene, right, meter.levels[1], theme);
    } else {
        paint_column(scene, rect, meter.levels[0], theme);
    }
}

/// One meter column: `Bg2` track + up to three stacked zone segments.
fn paint_column(scene: &mut VectorScene, col: Rect, level: f32, theme: Theme) {
    let radius = Radius::Xs.px();
    fill_rounded_rect(scene, col, radius, resolve(ColorToken::Bg2, theme));
    let level = level.clamp(0.0, 1.0);
    draw_segment(
        scene,
        col,
        0.0,
        ZONE_WARN,
        level,
        ColorToken::Success,
        theme,
    );
    draw_segment(
        scene,
        col,
        ZONE_WARN,
        ZONE_DANGER,
        level,
        ColorToken::Warn,
        theme,
    );
    draw_segment(
        scene,
        col,
        ZONE_DANGER,
        1.0,
        level,
        ColorToken::Danger,
        theme,
    );
}

/// Fill the `[lo, hi]` fraction of `col` with `token` if the current `level`
/// reaches into it (bottom-to-top).
fn draw_segment(
    scene: &mut VectorScene,
    col: Rect,
    lo: f32,
    hi: f32,
    level: f32,
    token: ColorToken,
    theme: Theme,
) {
    let top = level.min(hi);
    if top <= lo {
        return;
    }
    let y0 = col.y + col.h * (1.0 - top);
    let y1 = col.y + col.h * (1.0 - lo);
    let seg = Rect::new(col.x, y0, col.w, (y1 - y0).max(0.0));
    let radius = Radius::Xs.px().min(col.w * 0.5);
    fill_rounded_rect(scene, seg, radius, resolve(token, theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a11y_reports_louder_channel() {
        let m = LevelMeter::new(NodeId(1), "Master").levels(0.3, 0.9);
        let node = m.build_a11y(0.0, 0.0, 12.0, 80.0);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert!((node.numeric_value().unwrap() - 0.9).abs() < 1e-5);
    }

    #[test]
    fn a11y_clamps_over_unity() {
        let m = LevelMeter::new(NodeId(1), "x").levels(1.4, 0.2);
        let node = m.build_a11y(0.0, 0.0, 12.0, 80.0);
        assert!((node.numeric_value().unwrap() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn mono_sets_single_bar() {
        let m = LevelMeter::new(NodeId(1), "x").mono(0.4);
        assert!(!m.stereo);
        assert_eq!(m.levels, [0.4, 0.4]);
    }

    fn smoke(level_l: f32, level_r: f32, stereo: bool, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut m = LevelMeter::new(NodeId(1), "M").levels(level_l, level_r);
        m.stereo = stereo;
        paint_level_meter(&m, Rect::new(0.0, 0.0, 16.0, 80.0), &mut scene, theme);
    }

    #[test]
    fn paint_smoke_all_zones() {
        smoke(0.2, 0.65, true, Theme::Forge); // green + amber
        smoke(0.95, 1.3, true, Theme::Blueprint); // red + clip
        smoke(0.0, 0.0, false, Theme::Sunstone); // silence, mono
    }
}
