//! Parameters + UI projection for the Equalize Sizes tool.
//!
//! The vocab lives in the tool crate (TG-B/TG-C single-source-of-truth):
//! the panel reads [`EqualizeSizesUiSnapshot`] for paint and pushes
//! `EditorAction::ToolPanelEvent(PanelEvent::…)` for edits; the tool's
//! `handle_panel_event` maps each `NodeId` back into a typed
//! [`EqualizeSizesUiEdit`] and forwards to [`apply_ui_edit`], which is
//! the **only** site that clamps + commits values.

/// Hard cap on the fixed-mode dimensions (px). The user can type any
/// integer up to this in the W/H chips. `4096` is a safe upper bound for
/// the texture pool; the bake itself reuses the per-sprite Individual
/// texture path that already enforces `image::Limits` against
/// out-of-budget allocations.
pub const EQS_MAX_FIXED_DIM: u32 = 4096;

/// Hard cap on the grid unit (px). Matches `EQS_MAX_FIXED_DIM` for
/// symmetry — the slider track covers `[1, EQS_MAX_FIXED_DIM]` linearly.
pub const EQS_MAX_GRID_UNIT: u32 = 4096;

/// Three target-dim modes the tool offers. Stored on the tool; the panel
/// paints 3 toggle-buttons (radio-style) and the chip rows visible per
/// mode follow.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum TargetMode {
    /// Target = the max(W·|scale_x|, H·|scale_y|) over each selected
    /// sprite, aspect-preserving. Default: matches the user's intuition
    /// "make them all as big as the biggest one in the selection".
    #[default]
    MaxOfSelection,
    /// Target = exactly `(fixed_w, fixed_h)` in pixels. User types both
    /// in the W/H chips.
    Fixed,
    /// Target = each sprite's current visual size snapped UP to the next
    /// multiple of `grid_unit`. Keeps relative sizes proportional but
    /// quantizes the canvas to a grid (eg. 16, 32, 64-aligned sprites).
    GridUnit,
}

/// Three upscale algorithms exposed when `upscale_if_smaller` is on. xBR
/// is integer-factor only; the tool falls back to Lanczos3 for the
/// non-integer remainder if the user picks xBR with a fractional ratio.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UpscaleAlgorithm {
    /// Sinc-based 6×6 kernel. Default. Excellent for photos and gradient
    /// sprites.
    #[default]
    Lanczos3,
    /// Pixel replication. Preserves the exact pixel grid; only correct
    /// choice for pixel art when the user wants no filtering.
    Nearest,
    /// Edge-aware corner blending (Hyllian 2011). Integer factors only.
    Xbr,
}

/// Projection of the tool's state for the typed panel to paint. Pushed
/// once per frame by the host (analog to the Padding pattern).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EqualizeSizesUiSnapshot {
    pub target_mode: TargetMode,
    pub fixed_w: u32,
    pub fixed_h: u32,
    /// Grid cell size in pixels — mirror of `GridSnapState::square_cfg.cell_size`
    /// converted via `pixels_per_meter`. Synced into the tool every
    /// frame by the bridge (Grid mode panel shows it read-only).
    pub grid_unit: u32,
    pub upscale_if_smaller: bool,
    pub upscale_algorithm: UpscaleAlgorithm,
    pub rasterize_after: bool,
    /// Grid-mode "offset" (px). The target dim is `(grid_unit - offset,
    /// grid_unit - offset)` — same value applied to both axes (legacy
    /// `EqualizeModal` semantics). Clamped to `0..grid_unit/2`.
    pub grid_offset: u32,
    /// When `true` AND `target_mode == GridUnit`, Apply lays the
    /// selection out 1 sprite per cell sorted by world `(y, x)`, like
    /// the legacy "Arrange on Grid (1 per unit)" toggle.
    pub arrange_on_grid: bool,
}

impl Default for EqualizeSizesUiSnapshot {
    fn default() -> Self {
        // Mirrors `EqualizeSizesParams::default` / `EqualizeSizesTool::default`.
        Self {
            target_mode: TargetMode::MaxOfSelection,
            fixed_w: 256,
            fixed_h: 256,
            grid_unit: 32,
            upscale_if_smaller: false,
            upscale_algorithm: UpscaleAlgorithm::Lanczos3,
            rasterize_after: true,
            grid_offset: 0,
            arrange_on_grid: false,
        }
    }
}

/// Authoritative tool state. The shell never reads anything else for the
/// bake — `EqualizeSizesTool` holds one of these, projects it to a
/// snapshot per frame, and exposes it back via `params()`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EqualizeSizesParams {
    pub target_mode: TargetMode,
    pub fixed_w: u32,
    pub fixed_h: u32,
    /// See [`EqualizeSizesUiSnapshot::grid_unit`].
    pub grid_unit: u32,
    pub upscale_if_smaller: bool,
    pub upscale_algorithm: UpscaleAlgorithm,
    pub rasterize_after: bool,
    /// See [`EqualizeSizesUiSnapshot::grid_offset`].
    pub grid_offset: u32,
    /// See [`EqualizeSizesUiSnapshot::arrange_on_grid`].
    pub arrange_on_grid: bool,
}

impl Default for EqualizeSizesParams {
    fn default() -> Self {
        // Identical defaults to the snapshot — Params IS the spec the
        // snapshot mirrors; keep both in lock-step.
        Self {
            target_mode: TargetMode::MaxOfSelection,
            fixed_w: 256,
            fixed_h: 256,
            grid_unit: 32,
            upscale_if_smaller: false,
            upscale_algorithm: UpscaleAlgorithm::Lanczos3,
            rasterize_after: true,
            grid_offset: 0,
            arrange_on_grid: false,
        }
    }
}

/// One panel-originated edit. Crossed through
/// `EditorAction::ToolPanelEvent(PanelEvent::…)` and rebuilt from the
/// `NodeId` by `EqualizeSizesTool::handle_panel_event`. Inverse of
/// [`EqualizeSizesUiSnapshot`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EqualizeSizesUiEdit {
    /// Pick a target-dim mode (any of the 3 mode buttons).
    SetMode(TargetMode),
    /// W chip committed a new fixed-mode width (px).
    SetFixedW(u32),
    /// H chip committed a new fixed-mode height (px).
    SetFixedH(u32),
    /// Bridge-only: sync `params.grid_unit` from `GridSnapState`. The
    /// panel has no slider/chip for grid unit; the cell size is owned
    /// by the Grid Snap tool and pushed in by the bridge every frame.
    SetGridUnit(u32),
    /// Grid-mode offset (px) committed by the panel's offset
    /// slider/chip. The tool clamps to `0..grid_unit/2`.
    SetGridOffset(u32),
    /// Toggle button flipped — true means upscale small sprites with the
    /// selected `upscale_algorithm` before fitting to target.
    ToggleUpscaleIfSmaller,
    /// Pick an upscale algorithm (any of the 3 algorithm buttons).
    SetUpscaleAlgorithm(UpscaleAlgorithm),
    /// Toggle button flipped — true means Mitchell-Netravali resample
    /// each sprite to its target canvas (baking scale into pixels).
    ToggleRasterizeAfter,
    /// Toggle button flipped — true means Grid-mode Apply also lays
    /// the selection out 1 sprite per cell sorted by world `(y, x)`.
    ToggleArrangeOnGrid,
    /// Apply button pressed — the host drains a pending-apply latch and
    /// runs `run_full_resolution_multi` on the selection.
    Apply,
    /// Reset every param back to default. Fired by the panel's Reset
    /// button AND by the tool's `on_activate`.
    ResetAll,
}

/// Apply one panel-originated edit against [`EqualizeSizesParams`].
/// **Only site** that clamps + commits values — `handle_panel_event`
/// routes here, the panel never duplicates clamp logic.
///
/// Returns `true` if this edit asks the host to bake (the user pressed
/// Apply); the caller (the tool) reads that flag and flips its own
/// `pending_apply` latch — keeping the param mutator pure of side state.
pub fn apply_ui_edit(params: &mut EqualizeSizesParams, edit: EqualizeSizesUiEdit) -> bool {
    match edit {
        EqualizeSizesUiEdit::SetMode(m) => {
            params.target_mode = m;
            false
        }
        EqualizeSizesUiEdit::SetFixedW(w) => {
            params.fixed_w = w.clamp(1, EQS_MAX_FIXED_DIM);
            false
        }
        EqualizeSizesUiEdit::SetFixedH(h) => {
            params.fixed_h = h.clamp(1, EQS_MAX_FIXED_DIM);
            false
        }
        EqualizeSizesUiEdit::SetGridUnit(g) => {
            params.grid_unit = g.clamp(1, EQS_MAX_GRID_UNIT);
            // Re-clamp offset since its max depends on grid_unit.
            let max_off = params.grid_unit / 2;
            if params.grid_offset > max_off {
                params.grid_offset = max_off;
            }
            false
        }
        EqualizeSizesUiEdit::SetGridOffset(o) => {
            let max_off = params.grid_unit / 2;
            params.grid_offset = o.min(max_off);
            false
        }
        EqualizeSizesUiEdit::ToggleUpscaleIfSmaller => {
            params.upscale_if_smaller = !params.upscale_if_smaller;
            false
        }
        EqualizeSizesUiEdit::SetUpscaleAlgorithm(a) => {
            params.upscale_algorithm = a;
            false
        }
        EqualizeSizesUiEdit::ToggleRasterizeAfter => {
            params.rasterize_after = !params.rasterize_after;
            false
        }
        EqualizeSizesUiEdit::ToggleArrangeOnGrid => {
            params.arrange_on_grid = !params.arrange_on_grid;
            false
        }
        EqualizeSizesUiEdit::Apply => true,
        EqualizeSizesUiEdit::ResetAll => {
            *params = EqualizeSizesParams::default();
            false
        }
    }
}

/// Project the live tool state into a snapshot the panel paints.
pub fn snapshot_from_params(p: &EqualizeSizesParams) -> EqualizeSizesUiSnapshot {
    EqualizeSizesUiSnapshot {
        target_mode: p.target_mode,
        fixed_w: p.fixed_w,
        fixed_h: p.fixed_h,
        grid_unit: p.grid_unit,
        upscale_if_smaller: p.upscale_if_smaller,
        upscale_algorithm: p.upscale_algorithm,
        rasterize_after: p.rasterize_after,
        grid_offset: p.grid_offset,
        arrange_on_grid: p.arrange_on_grid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_snapshot_default() {
        let p = EqualizeSizesParams::default();
        assert_eq!(snapshot_from_params(&p), EqualizeSizesUiSnapshot::default());
    }

    #[test]
    fn apply_set_mode_overwrites_target_mode() {
        let mut p = EqualizeSizesParams::default();
        let triggered = apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetMode(TargetMode::Fixed));
        assert!(!triggered);
        assert_eq!(p.target_mode, TargetMode::Fixed);
    }

    #[test]
    fn apply_set_fixed_clamps_to_max() {
        let mut p = EqualizeSizesParams::default();
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetFixedW(99_999));
        assert_eq!(p.fixed_w, EQS_MAX_FIXED_DIM);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetFixedH(0));
        assert_eq!(p.fixed_h, 1, "fixed dims must clamp to ≥ 1");
    }

    #[test]
    fn apply_grid_unit_clamps_to_max() {
        let mut p = EqualizeSizesParams::default();
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetGridUnit(0));
        assert_eq!(p.grid_unit, 1);
        apply_ui_edit(
            &mut p,
            EqualizeSizesUiEdit::SetGridUnit(EQS_MAX_GRID_UNIT + 100),
        );
        assert_eq!(p.grid_unit, EQS_MAX_GRID_UNIT);
    }

    #[test]
    fn toggles_flip() {
        let mut p = EqualizeSizesParams::default();
        assert!(!p.upscale_if_smaller);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::ToggleUpscaleIfSmaller);
        assert!(p.upscale_if_smaller);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::ToggleUpscaleIfSmaller);
        assert!(!p.upscale_if_smaller);

        assert!(p.rasterize_after);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::ToggleRasterizeAfter);
        assert!(!p.rasterize_after);
    }

    #[test]
    fn apply_returns_true_only_on_apply_edit() {
        let mut p = EqualizeSizesParams::default();
        assert!(!apply_ui_edit(
            &mut p,
            EqualizeSizesUiEdit::SetMode(TargetMode::Fixed)
        ));
        assert!(apply_ui_edit(&mut p, EqualizeSizesUiEdit::Apply));
    }

    #[test]
    fn toggle_arrange_on_grid_flips() {
        let mut p = EqualizeSizesParams::default();
        assert!(!p.arrange_on_grid);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::ToggleArrangeOnGrid);
        assert!(p.arrange_on_grid);
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::ToggleArrangeOnGrid);
        assert!(!p.arrange_on_grid);
    }

    #[test]
    fn set_grid_offset_clamps_to_half_unit() {
        let mut p = EqualizeSizesParams {
            grid_unit: 64,
            ..EqualizeSizesParams::default()
        };
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetGridOffset(10));
        assert_eq!(p.grid_offset, 10);
        // Beyond `grid_unit / 2 = 32` clamps down.
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetGridOffset(99));
        assert_eq!(p.grid_offset, 32);
    }

    #[test]
    fn shrinking_grid_unit_reclamps_offset() {
        let mut p = EqualizeSizesParams {
            grid_unit: 64,
            grid_offset: 30,
            ..EqualizeSizesParams::default()
        };
        // grid_unit/2 was 32 (offset 30 fit). Now drop to 16
        // (grid_unit=32) — offset must shrink to 16.
        apply_ui_edit(&mut p, EqualizeSizesUiEdit::SetGridUnit(32));
        assert_eq!(p.grid_unit, 32);
        assert_eq!(p.grid_offset, 16);
    }
}
