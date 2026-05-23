//! Parameters + UI vocabulary for the stateful Upscale tool.
//!
//! Three algorithms cover the design space:
//!
//! - [`UpscaleAlgorithm::Lanczos3`] (default) — sinc-based separable
//!   resample (Duchon 1979). Best for photos, sprites with gradients,
//!   any source where ringing artefacts at hard edges are acceptable
//!   in exchange for smooth interpolation. Accepts non-integer factors.
//! - [`UpscaleAlgorithm::Nearest`] — pixel replication. Preserves the
//!   exact source grid; the only algorithm that gives "honest pixel
//!   art" output when the user wants no filtering at all. Accepts
//!   non-integer factors (the destination has rectangular runs).
//! - [`UpscaleAlgorithm::Xbr`] — edge-aware corner replacement (here
//!   implemented as Scale2x/3x/4x, Mazzoleni 2001 — the canonical
//!   free pixel-art-friendly upscaler with corner-blending behaviour
//!   close to Hyllian xBR's first-pass rules; a full Hyllian xBR is
//!   tracked as a fan-out follow-up). Integer factors only — clamps
//!   to 2 / 3 / 4 when the slider sits between them.
//!
//! Scale range: 1.0–[`SCALE_FULL_SCALE`] × (default
//! [`DEFAULT_SCALE_FACTOR`] = 2.0). Values <1 are clamped to 1; values
//! >`SCALE_FULL_SCALE` are clamped to `SCALE_FULL_SCALE`.

/// Default scale factor when the tool boots / on reset.
pub const DEFAULT_SCALE_FACTOR: f32 = 2.0;

/// Minimum scale factor the slider accepts (1× = no-op upscale, kept
/// in range so the panel can paint a saturating slider thumb).
pub const MIN_SCALE_FACTOR: f32 = 1.0;

/// Maximum scale factor the slider accepts. The 16× ceiling matches
/// the briefing UX and keeps the worst-case full-res output (16K² × 4
/// = ~1 GiB) within reason for a desktop tool — beyond it the user
/// almost certainly wants a streaming pipeline, not a one-shot bake.
pub const SCALE_FULL_SCALE: f32 = 16.0;

/// Which algorithm runs on Apply (and feeds the panel preview).
///
/// `Default` is `Lanczos3` — the briefing default; safest choice for
/// generic input the user has not yet told us is pixel-art.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum UpscaleAlgorithm {
    /// Sinc-based separable resample (Duchon 1979).
    #[default]
    Lanczos3,
    /// Pixel replication. Preserves the source grid exactly.
    Nearest,
    /// Edge-aware corner replacement (Scale2x/3x/4x as v1; see module
    /// docs). Integer factors only.
    Xbr,
}

impl UpscaleAlgorithm {
    /// Whether the algorithm requires an integer scale factor. xBR
    /// clamps to `{2, 3, 4}` regardless of the slider value; the other
    /// two accept the slider value as-is.
    pub fn requires_integer_factor(self) -> bool {
        matches!(self, UpscaleAlgorithm::Xbr)
    }

    /// Project the slider's continuous scale factor onto the closest
    /// value the algorithm actually supports.
    ///
    /// - `Lanczos3`, `Nearest`: clamp to `[MIN_SCALE_FACTOR,
    ///   SCALE_FULL_SCALE]`, otherwise pass through.
    /// - `Xbr`: clamp to `{2, 3, 4}` (rounded to nearest).
    pub fn project_scale(self, slider: f32) -> f32 {
        let clamped = slider.clamp(MIN_SCALE_FACTOR, SCALE_FULL_SCALE);
        match self {
            UpscaleAlgorithm::Lanczos3 | UpscaleAlgorithm::Nearest => clamped,
            UpscaleAlgorithm::Xbr => match clamped.round() as i32 {
                ..=2 => 2.0,
                3 => 3.0,
                _ => 4.0,
            },
        }
    }
}

/// Linear scale factor → normalized slider track `0.0..=1.0` (inverse
/// of [`slider_to_scale`]). The slider track is linear in scale
/// (`1.0` → `0.0`, `SCALE_FULL_SCALE` → `1.0`).
pub fn scale_to_slider(scale: f32) -> f32 {
    let c = scale.clamp(MIN_SCALE_FACTOR, SCALE_FULL_SCALE);
    ((c - MIN_SCALE_FACTOR) / (SCALE_FULL_SCALE - MIN_SCALE_FACTOR)).clamp(0.0, 1.0)
}

/// Normalized slider track `0.0..=1.0` → linear scale factor. Inverse
/// of [`scale_to_slider`].
pub fn slider_to_scale(track: f32) -> f32 {
    let t = track.clamp(0.0, 1.0);
    MIN_SCALE_FACTOR + t * (SCALE_FULL_SCALE - MIN_SCALE_FACTOR)
}

/// Live, full-scale upscale parameters owned by [`crate::UpscaleTool`].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpscaleParams {
    /// Active algorithm (drives both preview and Apply).
    pub algorithm: UpscaleAlgorithm,
    /// Continuous slider scale factor (always in `[MIN_SCALE_FACTOR,
    /// SCALE_FULL_SCALE]`). Projected to the algorithm's supported
    /// range at run time via [`UpscaleAlgorithm::project_scale`].
    pub scale_factor: f32,
}

impl Default for UpscaleParams {
    fn default() -> Self {
        Self {
            algorithm: UpscaleAlgorithm::default(),
            scale_factor: DEFAULT_SCALE_FACTOR,
        }
    }
}

/// Projection of the live tool state for the typed `ph2d-panel-upscale`
/// to paint. Published by the host once per frame while the tool is
/// active; the panel reads it via
/// `ph2d_panel_upscale::set_current_upscale_snapshot`.
///
/// `scale_factor` is the slider value (continuous); `effective_factor`
/// is the value the algorithm will actually use (xBR snaps to
/// `{2, 3, 4}`). The panel paints the slider against `scale_factor`
/// and shows `effective_factor` as the "output size" readout.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpscaleUiSnapshot {
    pub algorithm: UpscaleAlgorithm,
    pub scale_factor: f32,
    pub effective_factor: f32,
}

impl Default for UpscaleUiSnapshot {
    fn default() -> Self {
        let algorithm = UpscaleAlgorithm::default();
        let scale_factor = DEFAULT_SCALE_FACTOR;
        Self {
            algorithm,
            scale_factor,
            effective_factor: algorithm.project_scale(scale_factor),
        }
    }
}

/// One panel-originated edit. After ADR-0040 TG-B these edits travel
/// as `EditorAction::ToolPanelEvent(PanelEvent::…)` — the shell calls
/// [`crate::UpscaleTool::handle_panel_event`], which maps the
/// `NodeId` back to one of these variants and forwards it to
/// `apply_ui_edit`. Single source of truth for clamping.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UpscaleUiEdit {
    /// Algorithm dropdown / segmented selection changed.
    SetAlgorithm(UpscaleAlgorithm),
    /// Scale slider track moved (normalized `0.0..=1.0`, mapped via
    /// [`slider_to_scale`]).
    Scale(f32),
    /// Apply pressed — bake at full resolution.
    Apply,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_params_lanczos3_at_2x() {
        let p = UpscaleParams::default();
        assert_eq!(p.algorithm, UpscaleAlgorithm::Lanczos3);
        assert!((p.scale_factor - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn lanczos3_accepts_non_integer_factor() {
        let s = UpscaleAlgorithm::Lanczos3.project_scale(2.5);
        assert!((s - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn nearest_accepts_non_integer_factor() {
        let s = UpscaleAlgorithm::Nearest.project_scale(2.5);
        assert!((s - 2.5).abs() < f32::EPSILON);
    }

    #[test]
    fn xbr_clamps_to_integer_two_three_four() {
        assert_eq!(UpscaleAlgorithm::Xbr.project_scale(1.0), 2.0);
        assert_eq!(UpscaleAlgorithm::Xbr.project_scale(1.4), 2.0);
        assert_eq!(UpscaleAlgorithm::Xbr.project_scale(2.6), 3.0);
        assert_eq!(UpscaleAlgorithm::Xbr.project_scale(3.5), 4.0);
        assert_eq!(UpscaleAlgorithm::Xbr.project_scale(7.0), 4.0);
    }

    #[test]
    fn scale_clamps_to_range() {
        assert_eq!(UpscaleAlgorithm::Lanczos3.project_scale(0.1), 1.0);
        assert_eq!(UpscaleAlgorithm::Lanczos3.project_scale(99.0), 16.0);
    }

    #[test]
    fn slider_round_trip_endpoints() {
        assert_eq!(scale_to_slider(MIN_SCALE_FACTOR), 0.0);
        assert!((scale_to_slider(SCALE_FULL_SCALE) - 1.0).abs() < f32::EPSILON);
        assert!((slider_to_scale(0.0) - MIN_SCALE_FACTOR).abs() < f32::EPSILON);
        assert!((slider_to_scale(1.0) - SCALE_FULL_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn slider_clamps_oob_input() {
        assert_eq!(scale_to_slider(-5.0), 0.0);
        assert!((scale_to_slider(100.0) - 1.0).abs() < f32::EPSILON);
        assert!((slider_to_scale(-2.0) - MIN_SCALE_FACTOR).abs() < f32::EPSILON);
        assert!((slider_to_scale(7.0) - SCALE_FULL_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn default_snapshot_matches_default_params() {
        let s = UpscaleUiSnapshot::default();
        let p = UpscaleParams::default();
        assert_eq!(s.algorithm, p.algorithm);
        assert!((s.scale_factor - p.scale_factor).abs() < f32::EPSILON);
        assert!((s.effective_factor - 2.0).abs() < f32::EPSILON);
    }
}
