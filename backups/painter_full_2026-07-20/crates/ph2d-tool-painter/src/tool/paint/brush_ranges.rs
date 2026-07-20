//! Interactive UI **range constants** for the Stroke section's non-`0..1` sliders (the `0..1` track maps
//! onto `0..MAX`). Split from `paint.rs` for the workspace file-LOC cap; re-exported via `pub use` there so
//! the `paint::` path is unchanged. Values mirror `BrushSpec::default` / the engine's Blender soft ranges.

use super::*;

/// Default control-handle grab radius (image px) — the Curve/Ellipse editors share one tolerance until the
/// shell forwards a screen-scaled value via [`PainterTool::set_shape_grab_tol_px`].
pub const DEFAULT_SHAPE_GRAB_TOL_PX: f32 = 8.0;

/// Smallest brush radius the size UI maps to (image px); the size slider + `[` / `]` nudge clamp here.
pub const BRUSH_SIZE_MIN_PX: f32 = 1.0;
/// Largest brush radius the size UI maps to (image px); the interactive range, not the engine's hard cap.
pub const BRUSH_SIZE_MAX_PX: f32 = 512.0;
/// Max spacing the slider reaches, as a fraction of diameter (`1.0` = one full diameter between dabs).
pub const BRUSH_SPACING_MAX: f32 = 1.0;
/// Max absolute jitter the slider reaches, in pixels (View unit).
pub const BRUSH_JITTER_ABS_MAX_PX: f32 = 64.0;
/// Max value for the Input Samples / Dash Length count sliders (the engine's input-sample window cap).
pub const BRUSH_COUNT_SLIDER_MAX: u32 = MAX_INPUT_SAMPLES as u32;
/// Airbrush **Rate** slider floor (seconds) — the panel maps its `0..1` track linearly onto `[MIN, MAX]`.
/// Re-exported from the engine's Blender soft range so the value↔track map shares one source.
pub const BRUSH_AIRBRUSH_RATE_MIN_S: f32 = AIRBRUSH_RATE_MIN_S;
/// See [`BRUSH_AIRBRUSH_RATE_MIN_S`].
pub const BRUSH_AIRBRUSH_RATE_MAX_S: f32 = AIRBRUSH_RATE_MAX_S;
