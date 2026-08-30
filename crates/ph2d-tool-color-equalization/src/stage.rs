//! **The stage doors** — one predicate per pipeline stage, written ONCE
//! and read from BOTH sides.
//!
//! The pipeline (`algorithm::apply_color_equalization`) asks them to
//! decide whether to run a stage; the panel asks them to decide whether
//! that stage's knobs are worth painting. Because it is literally the
//! same expression, a stage that stops running takes its knobs off the
//! panel in the same edit.
//!
//! ## The defect these replace
//!
//! Four rows — **Tile Grid**, **LUT Mix**, **Dither Strength**,
//! **Dither Grain** — were painted unconditionally over stages that are
//! OFF in the state the panel is born in (`clip_limit == CLIP_LIMIT_MIN`,
//! both LUT slots `None`, `posterize_levels == 0`). The artist reaches
//! all four in the first second and none of them does anything.
//!
//! ⚠️ **No instrument in this repo catches that shape.** The value *is*
//! read (it is handed to the stage function), the click *does* reach
//! the tool, and the widget *is* focusable — so the wiring-parity gate,
//! the seam tests and the reaches-a-consumer gate are all green. The
//! only question that finds it is the third one: *does the consumer act
//! on the value, or discard it?*
//!
//! Split into its own file so `params.rs` stays under its frozen
//! workspace LOC cap — an allowance is a debt, not a budget.

use crate::lut_presets::LutPreset;
use crate::params::{CLIP_LIMIT_MIN, ColorEqualizationParams, ColorEqualizationUiSnapshot};

/// Stage 1 — CLAHE. At `CLIP_LIMIT_MIN` the per-tile CDF is uniform, so
/// the stage is skipped entirely and `tile_grid_size` reaches nothing.
pub fn clahe_runs(clip_limit: f32) -> bool {
    clip_limit > CLIP_LIMIT_MIN
}

/// Stage 2.5 — the LUT stage with TWO cubes, the only case in which
/// `lut_mix` is consumed. With one slot at `None` the pipeline applies
/// the other cube directly and the mix is discarded downstream — the
/// fully-wired-but-projected-out shape.
pub fn lut_blend_runs(intensity: f32, a: LutPreset, b: LutPreset) -> bool {
    intensity > 0.0 && a != LutPreset::None && b != LutPreset::None
}

/// Stage 6 — Posterize. `posterize_levels` defaults to `0` (off), so
/// everything downstream of it is inert at boot.
pub fn posterize_runs(levels: u32) -> bool {
    levels >= crate::algorithm::POSTERIZE_LEVELS_MIN
}

/// Stage 6's dither sub-pass — the door for
/// `posterize_dither_strength` and `posterize_dither_grain`. Needs BOTH
/// facts: a posterize stage to dither, and the toggle on.
pub fn dither_runs(levels: u32, dithering: bool) -> bool {
    posterize_runs(levels) && dithering
}

impl ColorEqualizationParams {
    /// Stage 1 — CLAHE runs (the door for `tile_grid_size`).
    pub fn clahe_stage_runs(self) -> bool {
        clahe_runs(self.clip_limit)
    }

    /// Stage 2.5 — the LUT stage runs *and* has TWO cubes to blend,
    /// which is the only case where `lut_mix` is consumed.
    pub fn lut_blend_stage_runs(self) -> bool {
        lut_blend_runs(self.lut_intensity, self.lut_preset_1, self.lut_preset_2)
    }

    /// Stage 6 — Posterize runs (the door for the Dither toggle).
    pub fn posterize_stage_runs(self) -> bool {
        posterize_runs(self.posterize_levels)
    }

    /// Stage 6's dither sub-pass runs — the door for
    /// `posterize_dither_strength` and `posterize_dither_grain`.
    pub fn dither_stage_runs(self) -> bool {
        dither_runs(self.posterize_levels, self.posterize_dithering)
    }
}

impl ColorEqualizationUiSnapshot {
    /// Whether the **Tile Grid** row is worth painting — i.e. whether
    /// CLAHE will consume `tile_grid_size` this frame.
    pub fn clahe_stage_runs(&self) -> bool {
        clahe_runs(self.clip_limit)
    }

    /// Whether the **LUT Mix** row is worth painting — the mix is only
    /// consumed when there are two cubes to blend.
    pub fn lut_blend_stage_runs(&self) -> bool {
        lut_blend_runs(self.lut_intensity, self.lut_preset_1, self.lut_preset_2)
    }

    /// Whether the **Dither** toggle is worth painting — it only means
    /// something once Posterize is on.
    pub fn posterize_stage_runs(&self) -> bool {
        posterize_runs(self.posterize_levels)
    }

    /// Whether the **Dither Strength** / **Dither Grain** rows are worth
    /// painting.
    pub fn dither_stage_runs(&self) -> bool {
        dither_runs(self.posterize_levels, self.posterize_dithering)
    }
}
