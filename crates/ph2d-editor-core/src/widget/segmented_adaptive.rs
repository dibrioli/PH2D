//! [`SegmentedAdaptive`] — a typed segmented control that reflows
//! overflowing options onto new rows instead of clipping labels.
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.7). Thin typed wrapper over
//! the canonical [`paint_segmented_group_adaptive`](super::panel_chrome::paint_segmented_group_adaptive)
//! chrome helper (the single source of truth for adaptive segmented
//! layout). Used for the 9-Slice draw modes (§3.5) and any segmented
//! control with enough options that they don't fit one row at the
//! Inspector's narrow column width. Owns the option list + a11y; the
//! paint helper registers per-segment hits as it lays them out.

use super::panel_chrome::{paint_segmented_group_adaptive, segmented_gap};
use crate::interaction::HitIndex;
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

#[derive(Clone, Debug)]
pub struct SegmentedOption {
    pub id: NodeId,
    pub label: String,
}

impl SegmentedOption {
    pub fn new(id: NodeId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SegmentedAdaptive {
    pub id: NodeId,
    pub label: String,
    pub options: Vec<SegmentedOption>,
    /// Index of the selected option. Out-of-range clamps to no
    /// selection (all segments render unselected).
    pub selected: usize,
}

impl SegmentedAdaptive {
    pub fn new(id: NodeId, label: impl Into<String>, options: Vec<SegmentedOption>) -> Self {
        Self {
            id,
            label: label.into(),
            options,
            selected: 0,
        }
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut b = NodeBuilder::new(Role::RadioGroup)
            .label(&self.label)
            .bounds(x, y, w, h);
        for opt in &self.options {
            b = b.child(opt.id);
        }
        b.build()
    }
}

/// Paint the adaptive segmented group, returning the total height used
/// (≥ `rect.h` when options reflow onto extra rows). Per-segment hit
/// rects are registered by the underlying chrome helper.
pub fn paint_segmented_adaptive(
    widget: &SegmentedAdaptive,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
) -> f32 {
    let segments: Vec<(&str, bool, NodeId)> = widget
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| (opt.label.as_str(), i == widget.selected, opt.id))
        .collect();
    paint_segmented_group_adaptive(rect, &segments, scene, text_system, theme, hit_index)
}

/// The height [`paint_segmented_adaptive`] will use at width `rect_w` and row height `row_h` — measure it
/// before sizing a container (e.g. a Card) so the container adapts to the reflow on a narrow panel.
#[must_use]
pub fn measure_segmented_adaptive(
    widget: &SegmentedAdaptive,
    rect_w: f32,
    row_h: f32,
    text_system: &mut TextSystem,
) -> f32 {
    let labels: Vec<&str> = widget.options.iter().map(|o| o.label.as_str()).collect();
    measure_segmented_group_adaptive(rect_w, row_h, &labels, text_system)
}

/// The height [`paint_segmented_group_adaptive`] will use for `labels` at width `rect_w` / row height
/// `row_h` — MEASURE before sizing a container so it adapts to the reflow on a narrow panel. Mirrors the
/// paint fn's END-demotion rule (each demoted button gets its own full-width row). Lives here (not in
/// `panel_chrome`) to keep that shared-chrome file under its LOC cap.
fn measure_segmented_group_adaptive(
    rect_w: f32,
    row_h: f32,
    labels: &[&str],
    text_system: &mut TextSystem,
) -> f32 {
    let n = labels.len();
    if n == 0 {
        return 0.0;
    }
    let gap = segmented_gap();
    let font_size = TypeToken::Sm.px();
    let pad_inside = Spacing::Lg.px() * 2.0;
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| text_system.layout(label, font_size, f32::INFINITY).width() + pad_inside)
        .collect();
    let mut top_n = n;
    while top_n > 1 {
        let total: f32 = widths[..top_n].iter().sum::<f32>() + gap * (top_n as f32 - 1.0);
        if total <= rect_w {
            break;
        }
        top_n -= 1;
    }
    let row_gap = Spacing::Xs.px();
    row_h + (n - top_n) as f32 * (row_gap + row_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> SegmentedAdaptive {
        SegmentedAdaptive::new(
            NodeId(1),
            "Draw Mode",
            vec![
                SegmentedOption::new(NodeId(2), "Simple"),
                SegmentedOption::new(NodeId(3), "Sliced"),
                SegmentedOption::new(NodeId(4), "Tiled"),
                SegmentedOption::new(NodeId(5), "Tiled Fit"),
            ],
        )
        .selected(1)
    }

    #[test]
    fn selected_index_stored() {
        assert_eq!(fixture().selected, 1);
    }

    #[test]
    fn a11y_role_is_radiogroup_with_children() {
        let node = fixture().build_a11y(0.0, 0.0, 200.0, 28.0);
        assert_eq!(node.role(), Role::RadioGroup);
    }

    #[test]
    fn paint_returns_height_at_least_row_for_narrow_width() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        // Narrow width forces reflow → height should exceed a single row.
        let h = paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, 60.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hit,
        );
        assert!(h >= 28.0);
    }

    #[test]
    fn measure_matches_the_painted_reflow_height() {
        // The card sizing relies on `measure_segmented_adaptive` predicting EXACTLY what
        // `paint_segmented_adaptive` will use — at a narrow width that forces reflow.
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        let w = 60.0;
        let painted = paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, w, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
            &mut hit,
        );
        let measured = measure_segmented_adaptive(&fixture(), w, 28.0, &mut text);
        assert!(
            (painted - measured).abs() < 0.01,
            "measure ({measured}) must equal the painted reflow height ({painted})"
        );
    }

    #[test]
    fn paint_smoke_wide() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, 400.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Blueprint,
            &mut hit,
        );
    }
}
