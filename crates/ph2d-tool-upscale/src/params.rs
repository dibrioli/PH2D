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
//! - [`UpscaleAlgorithm::Epx`] — edge-directed corner replacement
//!   (EPX / Scale2x family, Johnson 1992 / Mazzoleni 2001), evaluated
//!   as a continuous reconstruction so **every** integer stop from 1 to
//!   [`SCALE_FULL_SCALE`] is a different image. ⛔ It is not Hyllian
//!   xBR; see [`crate::algorithm::upscale_epx`] for the honest scope
//!   and the reason the old "xBR" label was removed.
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
    /// Edge-directed corner replacement (EPX family; see module docs).
    /// Snaps to whole factors — every integer `1..=SCALE_FULL_SCALE`.
    Epx,
}

impl UpscaleAlgorithm {
    /// Whether the algorithm snaps the slider to a whole factor.
    ///
    /// EPX does: a pixel-art enlargement at `2.5×` gives runs of two
    /// and three destination pixels for neighbouring source pixels,
    /// which is the artefact pixel artists pick this mode to avoid.
    /// ⚠️ Snapping to an INTEGER is not the same as snapping to a SET —
    /// the old code clamped to `{2, 3, 4}` and killed `4×…16×`.
    pub fn snaps_to_whole_factor(self) -> bool {
        matches!(self, UpscaleAlgorithm::Epx)
    }

    /// **THE door** — project the slider's continuous scale factor onto
    /// the value the algorithm will actually run at.
    ///
    /// Every consumer goes through here: `run_full_resolution` (what
    /// Apply bakes), the panel's chip, and the panel's size readout.
    /// That is what makes the chip's text and the baked image the same
    /// number by construction rather than by two people agreeing.
    ///
    /// - `Lanczos3`, `Nearest`: clamp to `[MIN_SCALE_FACTOR,
    ///   SCALE_FULL_SCALE]`, otherwise pass through.
    /// - `Epx`: the same clamp, then round to a whole factor.
    pub fn project_scale(self, slider: f32) -> f32 {
        let clamped = slider.clamp(MIN_SCALE_FACTOR, SCALE_FULL_SCALE);
        match self {
            UpscaleAlgorithm::Lanczos3 | UpscaleAlgorithm::Nearest => clamped,
            UpscaleAlgorithm::Epx => clamped.round(),
        }
    }
}

/// Slider track (`0..=1`) → the factor the algorithm will actually run
/// at, through [`UpscaleAlgorithm::project_scale`].
///
/// ⚠️ **The panel MUST print this, never `slider_to_scale(track)`.**
/// A chip that prints the request instead of the delivery is the most
/// expensive shape of a dead control: the artist confirms it with their
/// eyes and is wrong.
pub fn effective_factor(algorithm: UpscaleAlgorithm, track: f32) -> f32 {
    algorithm.project_scale(slider_to_scale(track))
}

/// Destination size the tool will bake, given a source size and the
/// live slider track. Same door as the chip — one projection, two
/// readouts.
pub fn effective_output_size(
    algorithm: UpscaleAlgorithm,
    track: f32,
    src_w: u32,
    src_h: u32,
) -> (u32, u32) {
    let f = effective_factor(algorithm, track);
    (
        ((src_w as f32 * f).round() as u32).max(1),
        ((src_h as f32 * f).round() as u32).max(1),
    )
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
/// is the value the algorithm will actually use ([`Epx`] rounds to a
/// whole factor). The panel paints the slider knob against
/// `scale_factor`, prints `effective_factor` in the chip, and prints
/// `source_w × source_h` alongside the projected output size.
///
/// ⚠️ `source_w` / `source_h` are `0` until the host pushes a snapshot
/// — the readout says so rather than inventing a size.
///
/// [`Epx`]: UpscaleAlgorithm::Epx
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UpscaleUiSnapshot {
    pub algorithm: UpscaleAlgorithm,
    pub scale_factor: f32,
    pub effective_factor: f32,
    pub source_w: u32,
    pub source_h: u32,
}

impl Default for UpscaleUiSnapshot {
    fn default() -> Self {
        let algorithm = UpscaleAlgorithm::default();
        let scale_factor = DEFAULT_SCALE_FACTOR;
        Self {
            algorithm,
            scale_factor,
            effective_factor: algorithm.project_scale(scale_factor),
            source_w: 0,
            source_h: 0,
        }
    }
}

/// One panel-originated edit. After ADR-0040 TG-B these edits travel
/// as `EditorAction::ToolPanelEvent(PanelEvent::…)` — the shell calls
/// [`crate::UpscaleTool::handle_panel_event`], which maps the
/// `NodeId` back to one of these variants and forwards it to
/// `apply_ui_edit`. Single source of truth for clamping.
///
/// **Audit T1.6 R9 V1-H2:** `#[non_exhaustive]` mirrors the
/// `BgRemovalUiEdit` precedent (R7 I1-1, commit `5f7680c`) — adding a
/// variant downstream stays semver-additive for external `match`.
#[derive(Copy, Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum UpscaleUiEdit {
    /// Algorithm dropdown / segmented selection changed.
    SetAlgorithm(UpscaleAlgorithm),
    /// Scale slider track moved (normalized `0.0..=1.0`, mapped via
    /// [`slider_to_scale`]).
    Scale(f32),
    /// Apply pressed — bake at full resolution.
    Apply,
    /// Reset every param back to default (algorithm + scale).
    ResetAll,
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

    /// ⭐ **The 80 %-dead-course gate, at the projection.** EPX snaps to
    /// a WHOLE factor, never to a SET: every integer stop from
    /// `MIN_SCALE_FACTOR` to `SCALE_FULL_SCALE` must survive the
    /// projection as itself. The old code answered `4.0` for everything
    /// from `3.5` up.
    #[test]
    fn epx_snaps_to_a_whole_factor_and_keeps_every_integer_stop() {
        for f in (MIN_SCALE_FACTOR as u32)..=(SCALE_FULL_SCALE as u32) {
            let want = f as f32;
            assert_eq!(
                UpscaleAlgorithm::Epx.project_scale(want),
                want,
                "integer stop {f} was projected away — the slider is dead there"
            );
        }
        // Between stops it rounds, and it rounds to the NEAR stop, not
        // to a member of some fixed set.
        assert_eq!(UpscaleAlgorithm::Epx.project_scale(2.6), 3.0);
        assert_eq!(UpscaleAlgorithm::Epx.project_scale(7.4), 7.0);
        assert_eq!(UpscaleAlgorithm::Epx.project_scale(12.6), 13.0);
        // Out of range still clamps.
        assert_eq!(UpscaleAlgorithm::Epx.project_scale(0.1), MIN_SCALE_FACTOR);
        assert_eq!(UpscaleAlgorithm::Epx.project_scale(99.0), SCALE_FULL_SCALE);
    }

    /// ⭐ **The chip must print the DELIVERY, not the request.**
    /// [`effective_factor`] is the door both the chip and the bake
    /// read; this pins that it disagrees with the raw slider reading
    /// exactly where the algorithm snaps.
    #[test]
    fn effective_factor_is_the_projection_not_the_raw_track() {
        let track = scale_to_slider(7.4);
        let raw = slider_to_scale(track);
        assert!((raw - 7.4).abs() < 1e-3, "raw track reads {raw}");
        // Lanczos3 passes the request through.
        assert!((effective_factor(UpscaleAlgorithm::Lanczos3, track) - raw).abs() < 1e-3);
        // EPX delivers 7, and the chip has to say 7.
        assert_eq!(effective_factor(UpscaleAlgorithm::Epx, track), 7.0);
    }

    #[test]
    fn effective_output_size_reads_through_the_same_door() {
        let track = scale_to_slider(7.4);
        assert_eq!(
            effective_output_size(UpscaleAlgorithm::Epx, track, 64, 32),
            (448, 224)
        );
        assert_eq!(
            effective_output_size(UpscaleAlgorithm::Epx, track, 64, 32),
            {
                let f = effective_factor(UpscaleAlgorithm::Epx, track);
                (((64.0 * f) as u32), ((32.0 * f) as u32))
            },
            "the readout must be derived from the door, never re-computed"
        );
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
