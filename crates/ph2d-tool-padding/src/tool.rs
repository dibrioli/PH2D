//! [`PaddingTool`] — stateful editor Tool for canvas padding / crop.
//!
//! Model: four signed per-edge pixel counts + a one-shot `pending_apply`
//! flag. Far leaner than `ph2d_tool_bgremoval::BgRemovalTool` — no source
//! snapshot, no thumbnail, no scratch, no live preview (those are v2).
//! The tool reacts only to its panel widgets, never to the canvas (the
//! §5.5 ENTREGÁVEL contract).
//!
//! ## Apply flow
//!
//! On the panel's Apply, [`PaddingTool::apply_ui_edit`] sets
//! `pending_apply`. The shell drains it via [`PaddingTool::take_pending_apply`]
//! each frame; on `true` it reads the live `Sprite.source`, builds a
//! `ph2d_tool_padding::PaddingSpec` from [`PaddingTool::spec`], runs
//! `add_padding` at full resolution, swaps the texture for a fresh
//! Individual one, and reprojects the pivot so the world position holds
//! (the make_square precedent).

use ph2d_editor_core::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use ph2d_editor_core::tool::Tool;

use super::params::{PaddingUiEdit, PaddingUiSnapshot};

/// Editor Tool implementing the stateful Padding / Expand feature.
///
/// `Default` is hand-written (not derived) because `recenter_pivot`
/// defaults to `true`, not the `bool` zero value.
#[derive(Clone, Debug)]
pub struct PaddingTool {
    /// Signed per-edge padding/crop, in pixels (positive = expand with
    /// transparent pixels, negative = crop).
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
    /// Pivot mode. `true` (default) = recenter: the shell recalculates
    /// the sprite translation on Apply so the original content's world
    /// position is preserved. `false` = keep the pivot unchanged (the
    /// shell leaves the translation alone, so the canvas resizes around
    /// the current pivot point and the content visually shifts).
    recenter_pivot: bool,
    /// Set `true` when the user presses Apply; the host drains it via
    /// [`Self::take_pending_apply`] and bakes at full resolution.
    pending_apply: bool,
}

impl Default for PaddingTool {
    fn default() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
            recenter_pivot: true,
            pending_apply: false,
        }
    }
}

impl PaddingTool {
    /// Project the current per-edge state into the snapshot the typed
    /// `ph2d-panel-padding` paints. Published by the host once per frame
    /// while the tool is active (forward of [`Self::apply_ui_edit`]).
    pub fn ui_snapshot(&self) -> PaddingUiSnapshot {
        PaddingUiSnapshot {
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            left: self.left,
            recenter_pivot: self.recenter_pivot,
        }
    }

    /// Apply one panel-originated edit against the live state. `Apply`
    /// arms the pending-apply flag the host drains via
    /// [`Self::take_pending_apply`]. Inverse of [`Self::ui_snapshot`].
    pub fn apply_ui_edit(&mut self, edit: PaddingUiEdit) {
        match edit {
            PaddingUiEdit::Top(v) => self.top = v,
            PaddingUiEdit::Right(v) => self.right = v,
            PaddingUiEdit::Bottom(v) => self.bottom = v,
            PaddingUiEdit::Left(v) => self.left = v,
            PaddingUiEdit::TogglePivotRecenter => self.recenter_pivot = !self.recenter_pivot,
            PaddingUiEdit::Apply => self.pending_apply = true,
        }
    }

    /// Whether Apply should recenter the pivot (recalculate the sprite
    /// translation to keep the original content world-fixed). `false`
    /// leaves the translation unchanged. The shell reads this at bake
    /// time alongside [`Self::spec`].
    pub fn recenter_pivot(&self) -> bool {
        self.recenter_pivot
    }

    /// Drain the pending-apply flag. Returns `true` exactly once after
    /// each Apply trigger. Host calls this in its per-frame drain loop;
    /// on `true` it runs `add_padding` at full resolution.
    pub fn take_pending_apply(&mut self) -> bool {
        let p = self.pending_apply;
        self.pending_apply = false;
        p
    }

    /// The current signed per-edge spec `(top, right, bottom, left)`.
    /// The shell reads this at bake time and converts it to a
    /// `ph2d_tool_padding::PaddingSpec` (editor-core has no dep on that
    /// crate — the spec is just four `i32`s here).
    pub fn spec(&self) -> (i32, i32, i32, i32) {
        (self.top, self.right, self.bottom, self.left)
    }
}

impl Tool for PaddingTool {
    fn id(&self) -> ToolId {
        ToolId::new("padding")
    }

    fn label(&self) -> &str {
        "Padding"
    }

    fn icon_slug(&self) -> &str {
        "padding"
    }

    fn build_panel(&self) -> FloatingPanel {
        // The real UI is the typed `ph2d-panel-padding` crate; the legacy
        // FloatingPanel paint was retired (2026-05-17). A minimal panel
        // shell is still returned so `Tool::build_panel` has a value —
        // it carries no controls.
        let mut panel = FloatingPanel::new(self.id(), "Padding");
        panel.anchor = PanelAnchor::BottomCenter;
        panel
    }

    fn on_deactivate(&mut self) {
        // Clear only the pending-apply latch so a tool switch can't fire a
        // stray bake. The per-edge spec + pivot mode PERSIST across
        // activations (mirrors Bg Removal keeping its params): the panel's
        // slider/chip widget stores also persist, so resetting the spec
        // here would desync the painted fields from the tool the next time
        // the panel opens.
        self.pending_apply = false;
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_is_noop_and_not_pending() {
        let t = PaddingTool::default();
        assert_eq!(t.spec(), (0, 0, 0, 0));
        let mut t = t;
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn id_label_icon() {
        let t = PaddingTool::default();
        assert_eq!(t.id(), ToolId::new("padding"));
        assert_eq!(t.label(), "Padding");
        assert_eq!(t.icon_slug(), "padding");
    }

    #[test]
    fn edits_update_each_edge_and_round_trip_through_snapshot() {
        let mut t = PaddingTool::default();
        t.apply_ui_edit(PaddingUiEdit::Top(10));
        t.apply_ui_edit(PaddingUiEdit::Right(-5));
        t.apply_ui_edit(PaddingUiEdit::Bottom(3));
        t.apply_ui_edit(PaddingUiEdit::Left(-2));
        assert_eq!(t.spec(), (10, -5, 3, -2));
        let s = t.ui_snapshot();
        assert_eq!((s.top, s.right, s.bottom, s.left), (10, -5, 3, -2));
    }

    #[test]
    fn apply_arms_pending_once() {
        let mut t = PaddingTool::default();
        assert!(!t.take_pending_apply());
        t.apply_ui_edit(PaddingUiEdit::Apply);
        assert!(t.take_pending_apply());
        // Drained: second call returns false.
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn deactivate_clears_pending_but_keeps_spec() {
        let mut t = PaddingTool::default();
        t.apply_ui_edit(PaddingUiEdit::Top(20));
        t.apply_ui_edit(PaddingUiEdit::Apply);
        t.on_deactivate();
        // Spec persists (panel widget stores persist too); only the
        // pending-apply latch is cleared.
        assert_eq!(t.spec(), (20, 0, 0, 0));
        assert!(!t.take_pending_apply());
    }

    #[test]
    fn pivot_recenter_defaults_on_and_toggles() {
        let mut t = PaddingTool::default();
        assert!(t.recenter_pivot());
        assert!(t.ui_snapshot().recenter_pivot);
        t.apply_ui_edit(PaddingUiEdit::TogglePivotRecenter);
        assert!(!t.recenter_pivot());
        assert!(!t.ui_snapshot().recenter_pivot);
        t.apply_ui_edit(PaddingUiEdit::TogglePivotRecenter);
        assert!(t.recenter_pivot());
    }
}
