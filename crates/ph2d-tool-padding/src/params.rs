//! Parameters + UI projection for the stateful Padding tool.
//!
//! The Padding tool condenses the legacy *Image Padding* + *Directional
//! Expand* tools into one: four SIGNED per-edge values (top / right /
//! bottom / left), where a positive value expands that edge with
//! transparent pixels and a negative value crops it. The pure resize
//! lives in `ph2d-tool-padding::add_padding`; this module is just the
//! editor-side state shape — editor-core deliberately does NOT depend on
//! `ph2d-tool-padding` (the spec is four `i32`s; the shell converts them
//! to `ph2d_tool_padding::PaddingSpec` at bake time).

/// Half-range (px) each per-edge slider spans on either side of its
/// neutral centre. The slider is BIPOLAR: track `0.0` = `−SCALE` px
/// (crop), `0.5` = `0` px, `1.0` = `+SCALE` px (expand). The paired
/// px chip is the exact value (the user can type beyond this range; the
/// slider thumb just saturates at the ends).
pub const PAD_SLIDER_FULL_SCALE: i32 = 512;

/// Normalize a signed pixel count into the bipolar slider track position
/// `0.0..=1.0` (`0.5` = neutral). Inverse of [`slider_to_px`].
pub fn px_to_slider(px: i32) -> f32 {
    (px as f32 / (2.0 * PAD_SLIDER_FULL_SCALE as f32) + 0.5).clamp(0.0, 1.0)
}

/// Map a bipolar slider track position `0.0..=1.0` to a signed pixel
/// count (rounded). Inverse of [`px_to_slider`].
pub fn slider_to_px(track: f32) -> i32 {
    ((track.clamp(0.0, 1.0) - 0.5) * 2.0 * PAD_SLIDER_FULL_SCALE as f32).round() as i32
}

/// Projection of the tool's per-edge state + pivot mode for the typed
/// `ph2d-panel-padding` to paint. The four edge fields are signed pixel
/// counts (positive = expand, negative = crop); `recenter_pivot` drives
/// the pivot-mode toggle. The host publishes a fresh snapshot each frame
/// via `ph2d_panel_padding::set_current_padding_snapshot`.
///
/// Unlike the Bg-Removal snapshot the edges are NOT normalized to
/// `0..1` — the panel paints px chips whose displayed value IS the pixel
/// count, so the snapshot carries the raw `i32`s (the slider track
/// position is derived via [`px_to_slider`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PaddingUiSnapshot {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
    /// `true` = recenter the pivot (recalculate the sprite translation so
    /// the original content's world position is preserved across the
    /// resize); `false` = keep the pivot unchanged (don't recalculate).
    pub recenter_pivot: bool,
}

impl Default for PaddingUiSnapshot {
    fn default() -> Self {
        // Mirrors `PaddingTool::default`: a no-op spec with pivot recenter
        // ON (the least-surprising default — content stays put).
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
            recenter_pivot: true,
        }
    }
}

/// One panel-originated edit. After ADR-0040 TG-C these edits no longer
/// travel as their own `EditorAction` variant — the panel pushes the
/// generic `EditorAction::ToolPanelEvent(PanelEvent::…)` and the shell
/// calls `PaddingTool::handle_panel_event`, which maps the `NodeId` back
/// to one of these variants and forwards it to `apply_ui_edit`. Inverse
/// of `PaddingTool::ui_snapshot`.
///
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1) — additive variants no
/// longer semver-break downstream `match`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PaddingUiEdit {
    /// Top edge field edited (signed px).
    Top(i32),
    /// Right edge field edited (signed px).
    Right(i32),
    /// Bottom edge field edited (signed px).
    Bottom(i32),
    /// Left edge field edited (signed px).
    Left(i32),
    /// Pivot-mode toggle clicked — flips `recenter_pivot`.
    TogglePivotRecenter,
    /// Apply pressed — bake the resized canvas at full resolution.
    Apply,
    /// Reset every per-edge padding back to 0 and the pivot-recenter
    /// toggle to its default. Fired by the panel's Reset button AND
    /// by the tool's `on_activate`.
    ResetAll,
}
