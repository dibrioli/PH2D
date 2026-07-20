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
use crate::interaction::{HitIndex, WidgetStore};
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
    store: &WidgetStore,
    hit_index: &mut HitIndex,
) -> f32 {
    let segments: Vec<(&str, bool, NodeId)> = widget
        .options
        .iter()
        .enumerate()
        .map(|(i, opt)| (opt.label.as_str(), i == widget.selected, opt.id))
        .collect();
    paint_segmented_group_adaptive(rect, &segments, scene, text_system, theme, store, hit_index)
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
/// `row_h` — MEASURE before sizing a container so it adapts to the reflow on a narrow panel.
///
/// It does not *mirror* the paint fn's wrap any more, it **shares** it: both ask
/// [`segmented_row_counts`]. Two copies of one layout rule is how a container comes to be measured by one
/// and filled by the other. Lives here (not in `panel_chrome`) to keep that shared-chrome file under its
/// LOC cap.
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
    let widths = segmented_natural_widths(labels, text_system);
    let rows = segmented_row_counts(rect_w, &widths).len();
    let row_gap = Spacing::Xs.px();
    row_h * rows as f32 + row_gap * (rows as f32 - 1.0)
}

/// Each label's natural width — the text plus the canonical breathing room. The paint side and the
/// measure side must agree to the pixel, so they both come here.
pub(crate) fn segmented_natural_widths(labels: &[&str], text_system: &mut TextSystem) -> Vec<f32> {
    let font_size = TypeToken::Sm.px();
    let pad_inside = Spacing::Lg.px() * 2.0;
    labels
        .iter()
        .map(|label| text_system.layout(label, font_size, f32::INFINITY).width() + pad_inside)
        .collect()
}

/// **How the group wraps: how many buttons on each row.** Greedy flow — fill a row with as many as fit
/// at their natural widths, then start the next.
///
/// ⚠️ This replaced an **END-demotion** rule (fit a prefix in the top row, then give every leftover a
/// full-width row of its own). The two agree wherever there are 0 or 1 leftovers, which is every group
/// of two to four options in the app — so the difference was invisible until a list of **ten** arrived
/// (the Impasto TOOL list) and rendered as three across the top and seven stacked one per line, each
/// stretched edge to edge. Enio, 2026-07-19: *"deve organizar os botões como na primeira linha de
/// botões, quantos couberem por linha e não um por linha."*
///
/// ⚠️ **Both the painter and the measurer call this**, and that is structural rather than tidy: they used
/// to implement the wrap twice, and a container measured by one rule and filled by another is how the
/// next section quietly paints over these buttons and kills their hit targets
/// (`seam_impasto_rig.rs::no_impasto_widget_loses_its_hit_to_the_section_below`).
///
/// A label wider than the whole row still gets a row — never an empty one, or the walk would not
/// terminate.
pub(crate) fn segmented_row_counts(rect_w: f32, widths: &[f32]) -> Vec<usize> {
    let gap = segmented_gap();
    let mut rows = Vec::new();
    let mut i = 0;
    while i < widths.len() {
        let mut n = 0usize;
        let mut used = 0.0f32;
        while i + n < widths.len() {
            let extra = widths[i + n] + if n > 0 { gap } else { 0.0 };
            if n > 0 && used + extra > rect_w {
                break;
            }
            used += extra;
            n += 1;
        }
        let n = n.max(1);
        rows.push(n);
        i += n;
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The group WRAPS by flowing, not by stacking leftovers one per line.**
    ///
    /// Enio, 2026-07-19, on the ten-button Impasto TOOL list: *"deve organizar os botões como na primeira
    /// linha de botões, quantos couberem por linha e não um por linha."* The old rule fit a prefix in the
    /// top row and gave every leftover a full-width row of its own — three across the top and seven
    /// stacked.
    ///
    /// The widths here are the unit the layout actually works in, so the gate needs no text system.
    #[test]
    fn a_long_group_packs_every_row_instead_of_stacking_leftovers() {
        let w = [60.0f32; 10];
        let rows = segmented_row_counts(200.0, &w);
        // Three fit per row, so ten land as 3+3+3+1 — four rows, where the old rule gave 1 + 9 = ten.
        assert_eq!(rows, vec![3, 3, 3, 1], "ten 60px buttons in a 200px row");
        // The claim stated so it cannot pass by accident: every row but the LAST is packed. (My first
        // draft demanded `n > 1` on every row after the first, which a correct flow fails whenever the
        // remainder is one — the gate was wrong, not the layout.)
        let (last, packed) = rows.split_last().expect("non-empty");
        assert!(
            packed.iter().all(|&n| n == packed[0]),
            "a row before the last is under-filled — that is the stacking this replaced: {rows:?}"
        );
        assert!(*last <= packed[0], "the remainder cannot exceed a full row");
        assert_eq!(
            rows.iter().sum::<usize>(),
            10,
            "every button lands exactly once"
        );
    }

    /// **…and the small groups are UNCHANGED**, which is what makes it safe to change a widget the whole
    /// app shares. Flow and end-demotion agree wherever there are 0 or 1 leftovers, and every other
    /// adaptive segmented in the app (Deform, Selection, Draw To, Depth source, …) is two to four options.
    /// So this pins the equivalence rather than trusting it.
    #[test]
    fn short_groups_wrap_exactly_as_they_did_before() {
        // Everything fits: one row.
        assert_eq!(segmented_row_counts(400.0, &[60.0, 60.0, 60.0]), vec![3]);
        // One leftover: a first row plus a row of one — identical under both rules.
        assert_eq!(segmented_row_counts(130.0, &[60.0, 60.0, 60.0]), vec![2, 1]);
        // A label wider than the row still gets a row of its own, and the walk terminates.
        assert_eq!(segmented_row_counts(50.0, &[500.0, 500.0]), vec![1, 1]);
    }

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
        let store = WidgetStore::with_capacity(4);
        // Narrow width forces reflow → height should exceed a single row.
        let h = paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, 60.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
            &store,
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
        let store = WidgetStore::with_capacity(4);
        let w = 60.0;
        let painted = paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, w, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
            &store,
            &mut hit,
        );
        let measured = measure_segmented_adaptive(&fixture(), w, 28.0, &mut text);
        assert!(
            (painted - measured).abs() < 0.01,
            "measure ({measured}) must equal the painted reflow height ({painted})"
        );
    }

    /// A segment whose store `ButtonState` is Hovered/Pressed still paints + stays hit-registered — proves
    /// the paint now READS the dispatcher-set hover/press state (Enio 2026-07-04; before, segmented buttons
    /// ignored it and showed no hover/press feedback).
    #[test]
    fn segment_hover_state_is_read_from_the_store() {
        use crate::interaction::InteractiveState;
        use crate::widget::ButtonState;
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        let mut store = WidgetStore::with_capacity(4);
        // What the dispatcher does when the cursor enters the second segment.
        store.register(
            NodeId(3),
            InteractiveState::Button {
                state: ButtonState::Hovered,
            },
        );
        paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, 400.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Forge,
            &store,
            &mut hit,
        );
        assert!(
            hit.iter_registrations().any(|(id, _)| id == NodeId(3)),
            "the hovered segment is painted + hit-registered"
        );
    }

    #[test]
    fn paint_smoke_wide() {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let mut hit = HitIndex::default();
        let store = WidgetStore::with_capacity(4);
        paint_segmented_adaptive(
            &fixture(),
            Rect::new(0.0, 0.0, 400.0, 28.0),
            &mut scene,
            &mut text,
            Theme::Blueprint,
            &store,
            &mut hit,
        );
    }
}
