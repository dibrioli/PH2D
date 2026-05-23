//! Parameters for each background-removal backend.
//!
//! All defaults are tuned for the "give me a reasonable result on the first
//! click" case. Power users tweak via the panel sliders / toggles.

/// Protection-brush dab profile: how the painted strength falls off from
/// the dab centre (`d = 0`) to its rim (`d = 1`). Drives both paint and
/// erase. All profiles are `1` at the centre and `0` at the rim, and
/// monotonically decreasing — deterministic (HR-5).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum BrushFalloff {
    /// `s = 1 − (3d² − 2d³)` — smoothstep-soft edge. The default: a soft
    /// feathered keep-region blends cleanly into the matte.
    #[default]
    Smooth,
    /// `s = √(1 − d²)` — hemisphere profile (rounded shoulder).
    Sphere,
    /// `s = (1 − d)²` — concentrated peak, quick falloff.
    Sharp,
    /// `s = 1` — hard disc (every covered pixel fully protected).
    Constant,
}

impl BrushFalloff {
    pub fn label(self) -> &'static str {
        match self {
            BrushFalloff::Smooth => "Smooth",
            BrushFalloff::Sphere => "Sphere",
            BrushFalloff::Sharp => "Sharp",
            BrushFalloff::Constant => "Hard",
        }
    }

    pub fn all() -> [BrushFalloff; 4] {
        [
            BrushFalloff::Smooth,
            BrushFalloff::Sphere,
            BrushFalloff::Sharp,
            BrushFalloff::Constant,
        ]
    }

    /// Strength `s ∈ [0,1]` at normalized distance `d ∈ [0,1]` from the
    /// dab centre. Inputs outside `[0,1]` are clamped (a pixel beyond the
    /// rim contributes `0`).
    pub fn strength(self, d: f32) -> f32 {
        let d = d.clamp(0.0, 1.0);
        match self {
            BrushFalloff::Constant => 1.0,
            BrushFalloff::Smooth => 1.0 - (3.0 * d * d - 2.0 * d * d * d),
            BrushFalloff::Sphere => (1.0 - d * d).max(0.0).sqrt(),
            BrushFalloff::Sharp => (1.0 - d) * (1.0 - d),
        }
    }
}

/// Tuning parameters for `algorithm::chroma::segment`.
#[derive(Copy, Clone, Debug)]
pub struct ChromaParams {
    /// Color-distance threshold in Oklab units. Pixels within `tolerance`
    /// of the detected background color are marked as background (after
    /// the connectivity step). Typical range 0.02..0.30; default 0.10.
    pub tolerance: f32,
    /// Width of the soft-alpha transition band, in Oklab units, *added*
    /// to `tolerance`. Pixels with ΔE in `[tolerance, tolerance + feather]`
    /// receive fractional alpha. Decoupled from `tolerance` so sharpness
    /// and softness tune independently. Range 0.0..0.20; default 0.04.
    pub feather: f32,
    /// If `Some`, override corner-auto detection and use this RGB color
    /// (sRGB 8-bit) as the background reference. `None` = auto-detect
    /// from 4 corners via mini k-means (k=2).
    pub reference_color: Option<[u8; 3]>,
    /// When true, subtract the background's chroma from soft-band pixels
    /// to remove color halos common to greenscreen-style backgrounds.
    pub despill: bool,
    /// When true, seed flood-fill from image borders and only kill
    /// connected bg-similar regions (prevents bleed-through into subject
    /// interiors that share the bg color). Auto-disabled if border-bg
    /// confidence < 60% (subject-touches-border case).
    pub use_flood: bool,
}

impl Default for ChromaParams {
    fn default() -> Self {
        Self {
            // Tuned defaults (Enio 2026-05-20): normalized slider 0.6 →
            // ΔE 0.18, slider 0.9 → soft-band 0.18. Project through
            // TOLERANCE_FULL_SCALE / FEATHER_FULL_SCALE.
            tolerance: 0.6 * TOLERANCE_FULL_SCALE,
            feather: 0.9 * FEATHER_FULL_SCALE,
            reference_color: None,
            despill: true,
            use_flood: true,
        }
    }
}

/// Tuning parameters for `algorithm::guided_filter::refine`.
#[derive(Copy, Clone, Debug)]
pub struct GuidedFilterParams {
    /// Radius of the guided-filter window, in pixels at full resolution.
    /// Mapped to the UX slider "Feather Radius" 1..100. Default 30. Set
    /// to 0 to disable refinement entirely (hard binary mask output).
    pub radius: u32,
    /// Regularization term ε. Smaller = edges held tighter to the guide;
    /// larger = smoother. He's matting paper uses 1e-7 with [0,1]-normalized
    /// inputs. Range 1e-9..1e-3 (log-scale slider); default 1e-7.
    pub epsilon: f32,
    /// When true, use the full RGB guide (per-pixel 3×3 covariance solve).
    /// When false, use a luma-only guide (~3× faster). Color guide gives
    /// meaningfully better edges where bg and fg are chromatically similar
    /// but luminance-distinct (red hair on orange skin, green shirt on
    /// grass, etc.).
    pub color_guide: bool,
    /// When true, only process pixels within ±2r of the mask boundary;
    /// interior is copied verbatim. 5-10× speedup on typical sprite masks
    /// (mostly-solid interior, thin boundary). Disable only for debugging.
    pub boundary_only: bool,
}

impl Default for GuidedFilterParams {
    fn default() -> Self {
        Self {
            // Tuned defaults (Enio 2026-05-20): normalized Refine slider
            // 0.01 → radius round(0.01 × 100) = 1. ε kept consistent with
            // the Feather slider's log map at feather01 = 0.9 (the same
            // `1e-6 · 10^(5·v)` curve `apply_ui_edit` applies) so the
            // boot state equals "Feather slider parked at 0.9".
            radius: (0.01 * REFINE_RADIUS_FULL_SCALE).round() as u32,
            epsilon: 1.0e-6 * 10.0_f32.powf(5.0 * 0.9),
            color_guide: true,
            boundary_only: true,
        }
    }
}

/// Top-level parameter bag passed into [`super::algorithm::run_pipeline`].
#[derive(Clone, Debug)]
pub struct BgRemovalParams {
    pub chroma: ChromaParams,
    pub refinement: GuidedFilterParams,
    /// Signed morphological grow/shrink of the final alpha matte, in
    /// pixels. `0.0` = no change (default). Negative erodes the matte
    /// (eats the residual background outline at the cost of nibbling the
    /// subject edge); positive dilates it (penetrates outward).
    /// Backend-agnostic post-process applied by `algorithm::compose`.
    pub grow_px: f32,
    /// Extra user-picked background colours (sRGB 8-bit), sampled from
    /// the image via the panel eyedropper. The chroma backend treats a
    /// pixel as background when it is within `tolerance` of the
    /// auto-detected colour OR **any** of these — so the user can knock
    /// out multi-coloured / gradient backgrounds the corner-auto pass
    /// misses. Empty by default (auto only).
    pub extra_bg_colors: Vec<[u8; 3]>,
}

impl Default for BgRemovalParams {
    fn default() -> Self {
        Self {
            chroma: ChromaParams::default(),
            refinement: GuidedFilterParams::default(),
            // Tuned default (Enio 2026-05-20): Grow chip −0.10 (a slight
            // erode to trim residual outline). The chip shows the signed
            // value `(grow01−0.5)·2`, so −0.10 ⇒ grow_px = −0.1·SCALE.
            grow_px: -0.1 * GROW_FULL_SCALE,
            extra_bg_colors: Vec::new(),
        }
    }
}

/// Max number of eyedropper-picked extra background colours. Bounds the
/// swatch row + the per-pixel min-ΔE inner loop; click-drag sampling
/// stops appending once full. // LITERAL-OK: UI + perf budget
pub const MAX_EXTRA_BG_COLORS: usize = 12;

/// Full-scale → normalized-slider mapping constants. The panel sliders
/// run 0..1; these are the full-scale maxima each maps onto. Centralized
/// here so `BgRemovalTool::ui_snapshot` (forward) and
/// `BgRemovalTool::apply_ui_edit` (inverse) — in crate
/// `ph2d-tool-bgremoval` — can't drift apart.
pub const TOLERANCE_FULL_SCALE: f32 = 0.30;
pub const FEATHER_FULL_SCALE: f32 = 0.20;
pub const REFINE_RADIUS_FULL_SCALE: f32 = 100.0;
/// Grow/Shrink is BIPOLAR: the slider runs 0..1 with `0.5` = neutral.
/// Each end maps to ∓`GROW_FULL_SCALE` pixels of erosion (`0.0`) /
/// dilation (`1.0`). `grow_px = (grow01 − 0.5) · 2 · GROW_FULL_SCALE`.
pub const GROW_FULL_SCALE: f32 = 8.0;
/// Protection-brush radius full-scale: normalized slider `1.0` maps to
/// this many SOURCE pixels of radius. `paint_protect_at_uv` consumes the
/// resulting radius directly.
pub const BRUSH_SIZE_FULL_SCALE: f32 = 256.0;
/// Default normalized protection-brush size (≈ 38 px of source radius at
/// the default 256 full-scale). Single source for the tool field default
/// + the snapshot default so they can't drift.
pub const DEFAULT_BRUSH_SIZE01: f32 = 0.15;

/// Normalized projection of [`BgRemovalParams`] for the panel UI. All
/// slider fields are in `0.0..=1.0`; the panel paints these directly as
/// slider track positions and the host publishes a fresh snapshot each
/// frame via `ph2d_panel_bgremoval::set_current_bgremoval_snapshot`.
///
/// Decoupling the panel from the full-scale param units keeps the
/// editor-core ↔ panel-crate boundary in normalized space — the panel
/// never needs to know that "Tolerance 1.0" means ΔE 0.30 in Oklab.
///
/// NOTE: this struct is intentionally NOT `Copy` — `extra_colors` is a
/// `Vec`. It is cheap to `Clone` (≤ `MAX_EXTRA_BG_COLORS` × 3 bytes).
#[derive(Clone, Debug, PartialEq)]
pub struct BgRemovalUiSnapshot {
    /// Chroma tolerance, normalized (`chroma.tolerance / TOLERANCE_FULL_SCALE`).
    pub tolerance01: f32,
    /// Soft-band feather, normalized (`chroma.feather / FEATHER_FULL_SCALE`).
    pub feather01: f32,
    /// Guided-filter refine radius, normalized
    /// (`refinement.radius / REFINE_RADIUS_FULL_SCALE`).
    pub refine01: f32,
    /// Grow/Shrink, normalized BIPOLAR (`0.5` = neutral; see
    /// [`GROW_FULL_SCALE`]).
    pub grow01: f32,
    /// User-picked extra background colours (sRGB 8-bit), painted as a
    /// row of swatches below the sliders. Right-clicking a swatch
    /// removes it. Mirrors [`BgRemovalParams::extra_bg_colors`].
    pub extra_colors: Vec<[u8; 3]>,
    /// Whether the panel's eyedropper is armed — while `true`, a
    /// click-drag over the sprite on the canvas samples colours into
    /// `extra_colors`. Drives the toggle button's pressed look.
    pub eyedropper_armed: bool,
    /// Whether the protection brush is armed — while `true`, a click-drag
    /// over the sprite on the canvas paints the freehand protection mask
    /// (forced-foreground "keep" region, both modes). Drives the Protect
    /// toggle button's pressed look.
    pub protect_brush_armed: bool,
    /// Whether the protection mask currently holds any painted pixels —
    /// drives the Clear-protection button's enabled/visible state and the
    /// "you have a protected region" affordance.
    pub has_protect_mask: bool,
    /// Protection-brush radius, normalized (`brush_radius / BRUSH_SIZE_FULL_SCALE`).
    /// Drives the Brush Size slider track + the on-canvas ring gizmo.
    pub brush_size01: f32,
    /// Protection-brush dab falloff profile — drives the 4-way segmented
    /// control's active highlight.
    pub falloff: BrushFalloff,
    /// Whether the painted protection mask is shown as an on-canvas tint
    /// overlay. Drives the Show-Mask toggle's pressed look + the shell's
    /// overlay gate.
    pub show_mask: bool,
}

impl Default for BgRemovalUiSnapshot {
    fn default() -> Self {
        // Mirrors `BgRemovalParams::default` projected through the
        // normalized mapping (tolerance 0.6, feather 0.9, refine 0.01,
        // grow chip −0.10 ⇒ grow01 0.45). Single source: every field
        // derives from the param defaults, never a hand-typed dup.
        let p = BgRemovalParams::default();
        Self {
            tolerance01: p.chroma.tolerance / TOLERANCE_FULL_SCALE,
            feather01: p.chroma.feather / FEATHER_FULL_SCALE,
            refine01: p.refinement.radius as f32 / REFINE_RADIUS_FULL_SCALE,
            grow01: p.grow_px / (2.0 * GROW_FULL_SCALE) + 0.5,
            extra_colors: Vec::new(),
            eyedropper_armed: false,
            protect_brush_armed: false,
            has_protect_mask: false,
            // Brush-state defaults (tool-state, not params): single source
            // is the consts above + the tool's manual Default.
            brush_size01: DEFAULT_BRUSH_SIZE01,
            falloff: BrushFalloff::Smooth,
            show_mask: true,
        }
    }
}

/// One panel-originated parameter edit. After ADR-0040 TG-B these edits
/// no longer travel as their own `EditorAction` variant — the panel
/// pushes the generic `EditorAction::ToolPanelEvent(PanelEvent::…)` and
/// the shell calls `BgRemovalTool::handle_panel_event`, which maps the
/// `NodeId` back to one of these variants and forwards it to
/// `apply_ui_edit`. Slider values are normalized `0.0..=1.0`; the tool
/// maps them back to full scale.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BgRemovalUiEdit {
    /// Tolerance slider moved (normalized).
    Tolerance(f32),
    /// Feather slider moved (normalized).
    Feather(f32),
    /// Refine slider moved (normalized).
    Refine(f32),
    /// Grow/Shrink slider moved (normalized bipolar; `0.5` = neutral).
    Grow(f32),
    /// Eyedropper toggle button clicked — flips the armed state. While
    /// armed, the shell samples colours from the canvas on click-drag
    /// and feeds them to the tool directly (no edit variant for the
    /// sample itself — the shell calls `add_extra_color`).
    ToggleEyedropper,
    /// Right-click on extra-colour swatch `idx` — removes it.
    RemoveExtraColor(usize),
    /// Protection-brush toggle button clicked — flips the armed state.
    /// While armed, the shell paints the freehand protection mask on
    /// click-drag (no edit variant for the dab itself — the shell calls
    /// `paint_protect_at_uv` directly, mirroring the eyedropper's
    /// `add_extra_color`).
    ToggleProtectBrush,
    /// Clear-protection button pressed — wipes the painted protection
    /// mask (the tool reruns the preview without any forced-keep region).
    ClearProtectMask,
    /// Protection-brush size slider moved (normalized `0..1`).
    BrushSize(f32),
    /// Protection-brush falloff profile changed (4-way segmented).
    SetFalloff(BrushFalloff),
    /// Show-Mask toggle clicked — flips the on-canvas tint overlay.
    ToggleShowMask,
    /// Apply button pressed — commit at full resolution.
    Apply,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_align_with_design_doc() {
        let p = BgRemovalParams::default();
        // Tuned defaults (Enio 2026-05-20): sliders 0.6 / 0.9 / 0.01 /
        // grow −0.10.
        assert!((p.chroma.tolerance - 0.6 * TOLERANCE_FULL_SCALE).abs() < 1e-6);
        assert!((p.chroma.feather - 0.9 * FEATHER_FULL_SCALE).abs() < 1e-6);
        assert_eq!(p.chroma.reference_color, None);
        assert!(p.chroma.despill);
        assert!(p.chroma.use_flood);
        assert_eq!(p.refinement.radius, 1);
        assert!(p.refinement.color_guide);
        assert!(p.refinement.boundary_only);
        assert!((p.grow_px - (-0.1 * GROW_FULL_SCALE)).abs() < 1e-6);
        assert!(p.extra_bg_colors.is_empty());
    }

    #[test]
    fn falloff_profiles_peak_at_centre_and_decay_to_rim() {
        for f in BrushFalloff::all() {
            assert!((f.strength(0.0) - 1.0).abs() < 1e-6, "{f:?} centre = 1");
            // Rim: Constant stays 1; the others reach (≈)0.
            if f == BrushFalloff::Constant {
                assert_eq!(f.strength(1.0), 1.0);
            } else {
                assert!(f.strength(1.0) < 1e-6, "{f:?} rim ≈ 0");
            }
            // Monotonic non-increasing across the radius.
            let mut prev = f.strength(0.0);
            for k in 1..=10 {
                let s = f.strength(k as f32 / 10.0);
                assert!(s <= prev + 1e-6, "{f:?} not monotonic at {k}");
                prev = s;
            }
            assert!(!f.label().is_empty());
        }
    }
}
