//! [`ProgressBar`] — determinate or indeterminate progress.
//!
//! Determinate fills `Bg2` track up to `value` with `Accent`. Indeterminate
//! paints a fixed-width sliver (the live shell animates its position via
//! `Affine::translate` against the track each frame).

use crate::paint::{fill_rounded_rect, paint_text_centered, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum ProgressMode {
    /// Show `value` as proportion of the track. `value` is clamped to
    /// 0..=1.
    Determinate(f32),
    /// Width of the track is unknown; paint a fixed sliver. The shell
    /// animates its position over time.
    #[default]
    Indeterminate,
}

#[derive(Clone, Debug)]
pub struct ProgressBar {
    pub id: NodeId,
    pub label: String,
    pub mode: ProgressMode,
    /// When true, paints `value%` centered on the bar.
    pub show_percent: bool,
}

impl ProgressBar {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
            mode: ProgressMode::Indeterminate,
            show_percent: false,
        }
    }

    pub fn determinate(mut self, value: f32) -> Self {
        self.mode = ProgressMode::Determinate(value.clamp(0.0, 1.0));
        self
    }

    pub fn indeterminate(mut self) -> Self {
        self.mode = ProgressMode::Indeterminate;
        self
    }

    pub fn show_percent(mut self, yes: bool) -> Self {
        self.show_percent = yes;
        self
    }

    /// Build the AccessKit node. Determinate mode publishes the
    /// numeric value (0..=1); indeterminate omits it.
    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut builder = NodeBuilder::new(Role::ProgressIndicator)
            .label(&self.label)
            .bounds(x, y, w, h);
        if let ProgressMode::Determinate(v) = self.mode {
            builder = builder
                .numeric_value(v as f64)
                .numeric_value_min(0.0)
                .numeric_value_max(1.0);
        }
        builder.build()
    }
}

const INDETERMINATE_FRACTION: f32 = 0.30;

pub fn paint_progress_bar(
    bar: &ProgressBar,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let radius = Radius::Full.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::Bg2, theme));
    let fill_w = match bar.mode {
        ProgressMode::Determinate(v) => rect.w * v.clamp(0.0, 1.0),
        ProgressMode::Indeterminate => rect.w * INDETERMINATE_FRACTION,
    };
    if fill_w > 0.0 {
        let fill_rect = Rect::new(rect.x, rect.y, fill_w, rect.h);
        fill_rounded_rect(scene, fill_rect, radius, resolve(ColorToken::Accent, theme));
    }
    if bar.show_percent
        && let ProgressMode::Determinate(v) = bar.mode
    {
        let pct = (v.clamp(0.0, 1.0) * 100.0).round() as i32;
        let text = format!("{pct}%");
        paint_text_centered(
            text_system,
            scene,
            &text,
            rect,
            TypeToken::Xs.px(),
            resolve(ColorToken::Text1, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_indeterminate() {
        let p = ProgressBar::new(NodeId(1), "Loading");
        assert!(matches!(p.mode, ProgressMode::Indeterminate));
    }

    #[test]
    fn determinate_clamps_value() {
        let p = ProgressBar::new(NodeId(1), "x").determinate(2.0);
        assert!(matches!(p.mode, ProgressMode::Determinate(v) if (v - 1.0).abs() < f32::EPSILON));
        let p = ProgressBar::new(NodeId(1), "x").determinate(-0.5);
        assert!(matches!(p.mode, ProgressMode::Determinate(v) if v == 0.0));
    }

    #[test]
    fn a11y_indeterminate_omits_value() {
        let p = ProgressBar::new(NodeId(1), "x");
        let node = p.build_a11y(0.0, 0.0, 100.0, 8.0);
        assert_eq!(node.role(), Role::ProgressIndicator);
        assert_eq!(node.numeric_value(), None);
    }

    #[test]
    fn a11y_determinate_publishes_value() {
        let p = ProgressBar::new(NodeId(1), "x").determinate(0.42);
        let node = p.build_a11y(0.0, 0.0, 100.0, 8.0);
        let v = node.numeric_value().unwrap();
        assert!((v - 0.42).abs() < 1e-5);
    }

    fn smoke(bar: ProgressBar, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::new();
        paint_progress_bar(
            &bar,
            Rect::new(0.0, 0.0, 200.0, 8.0),
            &mut scene,
            &mut text,
            theme,
        );
    }

    #[test]
    fn paint_smoke_determinate_half_with_percent() {
        smoke(
            ProgressBar::new(NodeId(1), "x")
                .determinate(0.5)
                .show_percent(true),
            Theme::ForgeSdf,
        );
    }

    #[test]
    fn paint_smoke_determinate_zero() {
        smoke(
            ProgressBar::new(NodeId(1), "x").determinate(0.0),
            Theme::Sunstone,
        );
    }

    #[test]
    fn paint_smoke_determinate_full() {
        smoke(
            ProgressBar::new(NodeId(1), "x").determinate(1.0),
            Theme::Blueprint,
        );
    }

    #[test]
    fn paint_smoke_indeterminate() {
        smoke(
            ProgressBar::new(NodeId(1), "x").indeterminate(),
            Theme::PaintStudio,
        );
    }
}
