//! [`PaddingTool`] — stateful editor Tool for canvas padding / crop.
//!
//! Model: four signed per-edge pixel counts + a one-shot `pending_apply`
//! flag. Far leaner than [`BgRemovalTool`](crate::tools::bgremoval) — no
//! source snapshot, no thumbnail, no scratch, no live preview (those are
//! v2). The tool reacts only to its panel widgets, never to the canvas
//! (the §5.5 ENTREGÁVEL contract).
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

use crate::floating_panel::{FloatingPanel, PanelAnchor, ToolId};
use crate::tool::Tool;

use super::params::{PaddingUiEdit, PaddingUiSnapshot};

/// Editor Tool implementing the stateful Padding / Expand feature.
///
/// `Default` is derived — every edge starts at `0` (a no-op spec) and
/// nothing is pending.
#[derive(Clone, Debug, Default)]
pub struct PaddingTool {
    /// Signed per-edge padding/crop, in pixels (positive = expand with
    /// transparent pixels, negative = crop).
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
    /// Set `true` when the user presses Apply; the host drains it via
    /// [`Self::take_pending_apply`] and bakes at full resolution.
    pending_apply: bool,
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
            PaddingUiEdit::Apply => self.pending_apply = true,
        }
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

    /// Reset every edge to `0` + clear the pending flag. Called on
    /// deactivate / Cancel so reactivating later starts clean.
    pub fn reset(&mut self) {
        self.top = 0;
        self.right = 0;
        self.bottom = 0;
        self.left = 0;
        self.pending_apply = false;
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
        // A tool switch / Cancel abandons any in-progress spec so a stale
        // value can't bake on the next activation.
        self.reset();
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
    fn deactivate_resets_spec_and_pending() {
        let mut t = PaddingTool::default();
        t.apply_ui_edit(PaddingUiEdit::Top(20));
        t.apply_ui_edit(PaddingUiEdit::Apply);
        t.on_deactivate();
        assert_eq!(t.spec(), (0, 0, 0, 0));
        assert!(!t.take_pending_apply());
    }
}
