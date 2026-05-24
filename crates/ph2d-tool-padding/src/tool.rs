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
use ph2d_editor_core::ids;
use ph2d_editor_core::tool::{PanelEvent, Tool};

use super::params::{PaddingUiEdit, PaddingUiSnapshot, slider_to_px};

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
    /// Set `true` by Reset / on_activate. Shell drains via
    /// [`Self::take_pending_panel_reset`] and re-runs
    /// `Panel::populate` so the slider visuals snap back to 0.
    pending_panel_reset: bool,
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
            pending_panel_reset: false,
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
            PaddingUiEdit::ResetAll => {
                // Defaults: zero padding all sides + pivot-recenter ON
                // (matches the `Default` ctor of `PaddingTool`).
                self.top = 0;
                self.right = 0;
                self.bottom = 0;
                self.left = 0;
                self.recenter_pivot = true;
                // Stage panel-store repopulation so sliders/chips
                // snap visually back to 0 (shell drains the flag).
                self.pending_panel_reset = true;
            }
        }
    }

    /// Drain the panel-reset request. The shell calls this each frame
    /// and re-runs `Panel::populate(store)` when it returns `true` so
    /// the slider knob + chip text snap back to defaults.
    pub fn take_pending_panel_reset(&mut self) -> bool {
        std::mem::take(&mut self.pending_panel_reset)
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

    fn on_activate(&mut self) {
        // Defaults load on every fresh panel open. Routed through
        // `apply_ui_edit::ResetAll` so the panel snapshot (next
        // frame) reflects the cleared per-edge fields.
        self.apply_ui_edit(PaddingUiEdit::ResetAll);
    }

    fn on_deactivate(&mut self) {
        // Clear only the pending-apply latch so a tool switch can't fire a
        // stray bake.
        self.pending_apply = false;
    }

    fn handle_panel_event(&mut self, event: PanelEvent) {
        // ADR-0040 TG-C: docked-panel `PAD_*` NodeIds route through
        // `apply_ui_edit` so the slider/chip → px projection lives once.
        // Sliders carry the bipolar normalized track 0..1; chips carry the
        // signed pixel value directly. The panel still mirrors slider
        // value ↔ chip value on its own (UI-state-local in the widget
        // store) — the tool only owns the SPEC, never the widget visuals.
        match event {
            // Sliders (bipolar 0..1 → signed px via slider_to_px).
            PanelEvent::SetValue(id, v) if id == ids::PAD_TOP => {
                self.apply_ui_edit(PaddingUiEdit::Top(slider_to_px(v as f32)));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_RIGHT => {
                self.apply_ui_edit(PaddingUiEdit::Right(slider_to_px(v as f32)));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_BOTTOM => {
                self.apply_ui_edit(PaddingUiEdit::Bottom(slider_to_px(v as f32)));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_LEFT => {
                self.apply_ui_edit(PaddingUiEdit::Left(slider_to_px(v as f32)));
            }
            // Number chips (raw signed px, rounded — the chip already
            // commits an integer value but `PanelEvent::SetValue` is f64).
            PanelEvent::SetValue(id, v) if id == ids::PAD_TOP_NUM => {
                self.apply_ui_edit(PaddingUiEdit::Top(v.round() as i32));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_RIGHT_NUM => {
                self.apply_ui_edit(PaddingUiEdit::Right(v.round() as i32));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_BOTTOM_NUM => {
                self.apply_ui_edit(PaddingUiEdit::Bottom(v.round() as i32));
            }
            PanelEvent::SetValue(id, v) if id == ids::PAD_LEFT_NUM => {
                self.apply_ui_edit(PaddingUiEdit::Left(v.round() as i32));
            }
            // Buttons.
            PanelEvent::Click(id) if id == ids::PAD_PIVOT_RECENTER => {
                self.apply_ui_edit(PaddingUiEdit::TogglePivotRecenter);
            }
            PanelEvent::Click(id) if id == ids::PAD_APPLY => {
                self.apply_ui_edit(PaddingUiEdit::Apply);
            }
            PanelEvent::Click(id) if id == ids::PAD_RESET => {
                self.apply_ui_edit(PaddingUiEdit::ResetAll);
            }
            _ => {}
        }
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
    fn spec_stays_stable_across_multi_sprite_drain_cycle() {
        // Padding's bake (`add_padding`) is stateless — the tool only
        // holds the spec + the one-shot pending_apply latch. The shell
        // drains pending_apply once per Apply, then loops over selected
        // sprites calling add_padding(rgba, w, h, spec) for each. This
        // test verifies the spec stays stable through that loop (no
        // accidental mutation by `take_pending_apply` etc.) so all N
        // sprites get padded with the SAME spec the user set.
        // Regression cover for the multi-sprite drain pattern (§12.9).
        let mut t = PaddingTool::default();
        t.apply_ui_edit(PaddingUiEdit::Top(10));
        t.apply_ui_edit(PaddingUiEdit::Right(-5));
        t.apply_ui_edit(PaddingUiEdit::Bottom(3));
        t.apply_ui_edit(PaddingUiEdit::Left(-2));
        t.apply_ui_edit(PaddingUiEdit::Apply);
        // Drain (one-shot Apply latch).
        assert!(t.take_pending_apply());
        // Now simulate the shell looping over N sprites — `spec()` is
        // called once per sprite, must return the same value every time
        // (no internal mutation between draws).
        let spec = t.spec();
        let recenter = t.recenter_pivot();
        for _ in 0..5 {
            assert_eq!(t.spec(), spec, "spec drift across multi-sprite loop");
            assert_eq!(
                t.recenter_pivot(),
                recenter,
                "recenter_pivot drift across multi-sprite loop",
            );
        }
        assert_eq!(spec, (10, -5, 3, -2));
        assert!(recenter);
    }

    #[test]
    fn on_activate_resets_state_and_arms_panel_repopulate() {
        // Regression cover for the §12.3 / §12.4 UI_Bugs entries:
        // `on_activate` must route through `apply_ui_edit::ResetAll` so
        // (a) state goes back to defaults AND (b) `pending_panel_reset`
        // arms so the shell bridge re-runs `Panel::populate(store)` and
        // the slider knobs visually snap to 0.
        let mut t = PaddingTool::default();
        // Dirty the state — simulates a previous session.
        t.apply_ui_edit(PaddingUiEdit::Top(20));
        t.apply_ui_edit(PaddingUiEdit::TogglePivotRecenter);
        assert_eq!(t.spec(), (20, 0, 0, 0));
        assert!(!t.recenter_pivot());
        // Drain any stray reset flag first.
        let _ = t.take_pending_panel_reset();

        t.on_activate();

        assert_eq!(t.spec(), (0, 0, 0, 0));
        assert!(t.recenter_pivot());
        assert!(
            t.take_pending_panel_reset(),
            "on_activate must arm pending_panel_reset so the shell repopulates"
        );
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
