//! [`LevelMeter`] — a vertical stereo level meter with green/amber/red zones.
//!
//! Non-interactive (output-only, like [`crate::widget::ProgressBar`]). Two
//! quantities per channel: an **RMS** fill (≈ perceived loudness) that reads as
//! the bar, and a **peak-hold** marker — a thin line at the held peak that
//! catches transients the averaged fill misses. The caller feeds already-
//! ballistics-processed values (the peak-hold decay lives on the caller side,
//! e.g. the mixer panel); the widget just paints. Zones use the semantic
//! `Success → Warn → Danger` tokens so the meter reads in every theme (no
//! literal colours).

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// Fraction of full scale where the meter turns Success → Warn (≈ −6 dBFS).
const ZONE_WARN: f32 = 0.5; // LITERAL-PX-OK: meter zone threshold (amplitude fraction, not a UI dimension)
/// Fraction of full scale where the meter turns Warn → Danger (≈ −2 dBFS).
const ZONE_DANGER: f32 = 0.8; // LITERAL-PX-OK: meter zone threshold (amplitude fraction, not a UI dimension)
/// Peak-hold marker thickness.
const HOLD_THICKNESS_PX: f32 = 2.0; // LITERAL-PX-OK: peak-hold marker line thickness (chrome)
/// Clip cap height at the top of the column when a channel has clipped.
const CLIP_CAP_PX: f32 = 3.0; // LITERAL-PX-OK: clip-indicator cap height (chrome)

/// A stereo (or mono) level meter: RMS fill + peak-hold marker + clip cap.
#[derive(Clone, Debug)]
pub struct LevelMeter {
    pub id: NodeId,
    pub label: String,
    /// Per-channel RMS `[left, right]` — the bar fill (values > 1.0 clip).
    pub rms: [f32; 2],
    /// Per-channel held peak `[left, right]` — the marker line.
    pub peak_hold: [f32; 2],
    /// Per-channel latched clip flag — a solid `Danger` cap at the column top.
    pub clipped: [bool; 2],
    /// When false, paints a single (mono) column from channel 0.
    pub stereo: bool,
}

impl LevelMeter {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            rms: [0.0, 0.0],
            peak_hold: [0.0, 0.0],
            clipped: [false, false],
            stereo: true,
        }
    }

    /// Set the per-channel latched clip flags.
    pub fn clipped(mut self, left: bool, right: bool) -> Self {
        self.clipped = [left, right];
        self
    }

    /// Set the RMS fill levels.
    pub fn rms(mut self, left: f32, right: f32) -> Self {
        self.rms = [left, right];
        self
    }

    /// Set the peak-hold marker levels.
    pub fn peak_hold(mut self, left: f32, right: f32) -> Self {
        self.peak_hold = [left, right];
        self
    }

    /// Convenience: set both the fill and the marker to the same levels (a
    /// simple meter with no separate peak-hold — e.g. the widget showcase).
    pub fn levels(mut self, left: f32, right: f32) -> Self {
        self.rms = [left, right];
        self.peak_hold = [left, right];
        self
    }

    /// Single-bar mono meter at `level` (fill and marker equal).
    pub fn mono(mut self, level: f32) -> Self {
        self.rms = [level, level];
        self.peak_hold = [level, level];
        self.stereo = false;
        self
    }

    /// AccessKit node — a progress-indicator surface reporting the louder
    /// channel's held peak (`0..1`).
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let peak = self.peak_hold[0].max(self.peak_hold[1]).clamp(0.0, 1.0);
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
        paint_column(
            scene,
            left,
            meter.rms[0],
            meter.peak_hold[0],
            meter.clipped[0],
            theme,
        );
        paint_column(
            scene,
            right,
            meter.rms[1],
            meter.peak_hold[1],
            meter.clipped[1],
            theme,
        );
    } else {
        paint_column(
            scene,
            rect,
            meter.rms[0],
            meter.peak_hold[0],
            meter.clipped[0],
            theme,
        );
    }
}

/// One meter column: `Bg2` track + up to three stacked zone segments (RMS fill)
/// + a peak-hold marker line + a clip cap.
fn paint_column(
    scene: &mut VectorScene,
    col: Rect,
    rms: f32,
    peak_hold: f32,
    clipped: bool,
    theme: Theme,
) {
    let radius = Radius::Xs.px();
    fill_rounded_rect(scene, col, radius, resolve(ColorToken::Bg2, theme));
    let rms = rms.clamp(0.0, 1.0);
    draw_segment(scene, col, 0.0, ZONE_WARN, rms, ColorToken::Success, theme);
    draw_segment(
        scene,
        col,
        ZONE_WARN,
        ZONE_DANGER,
        rms,
        ColorToken::Warn,
        theme,
    );
    draw_segment(scene, col, ZONE_DANGER, 1.0, rms, ColorToken::Danger, theme);
    draw_peak_hold(scene, col, peak_hold, theme);
    if clipped {
        // A solid Danger cap at the very top — latched until the caller clears it.
        let cap = Rect::new(col.x, col.y, col.w, CLIP_CAP_PX.min(col.h));
        fill_rounded_rect(
            scene,
            cap,
            radius.min(col.w * 0.5),
            resolve(ColorToken::Danger, theme),
        );
    }
}

/// Fill the `[lo, hi]` fraction of `col` with `token` if `level` reaches into it
/// (bottom-to-top).
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

/// A thin marker line at the held-peak position, `Danger` when it's clipping,
/// else `Text1` (bright over any fill). Skipped at silence.
fn draw_peak_hold(scene: &mut VectorScene, col: Rect, peak_hold: f32, theme: Theme) {
    let ph = peak_hold.clamp(0.0, 1.0);
    if ph <= 0.0 {
        return;
    }
    let token = if ph >= ZONE_DANGER {
        ColorToken::Danger
    } else {
        ColorToken::Text1
    };
    let y = crate::math::safe_clamp(
        col.y + col.h * (1.0 - ph) - HOLD_THICKNESS_PX * 0.5,
        col.y,
        col.y + col.h - HOLD_THICKNESS_PX,
    );
    let mark = Rect::new(col.x, y, col.w, HOLD_THICKNESS_PX);
    fill_rounded_rect(
        scene,
        mark,
        Radius::Xs.px().min(col.w * 0.5),
        resolve(token, theme),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a11y_reports_louder_held_peak() {
        let m = LevelMeter::new(NodeId(1), "Master")
            .rms(0.3, 0.5)
            .peak_hold(0.4, 0.9);
        let node = m.build_a11y(0.0, 0.0, 12.0, 80.0);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert!((node.numeric_value().unwrap() - 0.9).abs() < 1e-5);
    }

    #[test]
    fn a11y_clamps_over_unity() {
        let m = LevelMeter::new(NodeId(1), "x").peak_hold(1.4, 0.2);
        let node = m.build_a11y(0.0, 0.0, 12.0, 80.0);
        assert!((node.numeric_value().unwrap() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn levels_sets_both_fill_and_marker() {
        let m = LevelMeter::new(NodeId(1), "x").levels(0.4, 0.6);
        assert_eq!(m.rms, [0.4, 0.6]);
        assert_eq!(m.peak_hold, [0.4, 0.6]);
    }

    #[test]
    fn mono_sets_single_bar() {
        let m = LevelMeter::new(NodeId(1), "x").mono(0.4);
        assert!(!m.stereo);
        assert_eq!(m.rms, [0.4, 0.4]);
        assert_eq!(m.peak_hold, [0.4, 0.4]);
    }

    fn smoke(rms_l: f32, hold_l: f32, clipped: bool, stereo: bool, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut m = LevelMeter::new(NodeId(1), "M")
            .rms(rms_l, rms_l * 0.9)
            .peak_hold(hold_l, hold_l)
            .clipped(clipped, clipped);
        m.stereo = stereo;
        paint_level_meter(&m, Rect::new(0.0, 0.0, 16.0, 80.0), &mut scene, theme);
    }

    #[test]
    fn paint_smoke_all_zones() {
        smoke(0.2, 0.65, false, true, Theme::Forge); // green fill + amber hold
        smoke(0.95, 1.3, true, true, Theme::Blueprint); // red fill + clip cap
        smoke(0.0, 0.0, false, false, Theme::Sunstone); // silence, mono
    }
}
