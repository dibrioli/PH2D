//! `BrushSpec` — the brush parameters.
//!
//! Clean-room model of the relevant fields of Blender's `Brush` (`makesdna/DNA_brush_types.h`):
//! radius, strength, the per-dab build-up (`flow`/`alpha`), spacing, blend mode, the distance
//! falloff curve, jitter, and the paint colour. Fields the texture painter does not need yet
//! (texture slots, stencil masks, projection options) are deliberately omitted — see
//! `docs/Painter/02_plano_de_implementacao.md` for what each phase adds.

use crate::blend::BrushBlend;
use crate::falloff::Falloff;
use crate::falloff_curve::FalloffCurve;
use crate::stroke_method::{JitterUnit, StrokeMethod};
use crate::texture::TextureSettings;

/// Largest brush radius the engine will allocate a dab for, in pixels. Derived from the editor
/// overlay budget (HR-4): a 4096-px-radius dab would be an 8k² bbox — far past interactive. This
/// caps the bbox, not the artist's intent (the value is clamped, not rejected).
pub const MAX_BRUSH_RADIUS_PX: f32 = 4096.0;

/// Airbrush **Rate** (timer period, seconds) soft-range floor — the fastest spray the UI slider
/// reaches (~100 Hz). Blender's `rate` `ui_range` is `0.01..1.0` s (default `0.1`); the hard RNA
/// range is wider, but `rna_brush.cc` is not in the vendored checkout, so this is the well-known
/// Blender soft range, not an in-tree-verified value. The default `0.1` IS verified
/// (`DNA_brush_types.h:232`). See [`crate::Stroke::tick`].
pub const AIRBRUSH_RATE_MIN_S: f32 = 0.01;
/// Airbrush **Rate** soft-range ceiling (slowest spray, ~1 Hz). See [`AIRBRUSH_RATE_MIN_S`].
pub const AIRBRUSH_RATE_MAX_S: f32 = 1.0;

/// Parameters of a single brush. Cheap to copy; the stroke engine reads it per dab.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BrushSpec {
    /// Dab radius in image pixels (UI label "Radius"). Clamped to `[0.5, MAX_BRUSH_RADIUS_PX]`.
    pub radius_px: f32,
    /// Plateau before the falloff begins, `0..1`. `0` = pure falloff curve; `1` = hard disk.
    /// Mirrors Blender's brush `hardness` as a distance remap applied before [`Self::falloff`].
    pub hardness: f32,
    /// Maximum opacity a single stroke can build to, `0..1` (Blender "Strength"). Enforced as a
    /// per-stroke accumulation cap by the stroke engine (later phase), not by a single dab.
    pub strength: f32,
    /// Per-dab build-up, `0..1` (Blender "Alpha"/flow). Scales every dab's coverage.
    pub flow: f32,
    /// Distance between dabs as a fraction of the diameter (Blender "Spacing"; default `0.10`).
    pub spacing: f32,
    /// How the dab colour combines with the layer.
    pub blend: BrushBlend,
    /// Radial intensity profile.
    pub falloff: Falloff,
    /// Random per-dab position offset when [`Self::jitter_unit`] is [`JitterUnit::Brush`]: a
    /// fraction of the diameter, `0..1` (Blender `Brush.jitter`, default `0.0`,
    /// `DNA_brush_types.h:221`). Max radial offset ≈ `jitter × diameter`.
    pub jitter: f32,
    /// Paint colour, straight RGB in `[0, 1]` in the layer's native space.
    pub color: [f32; 3],
    /// The editable profile used when [`Self::falloff`] is [`Falloff::Custom`]
    /// (ignored otherwise). Kept inline so the spec stays `Copy`/alloc-free.
    pub custom_falloff: FalloffCurve,

    // ── Stroke panel (Blender `paint_stroke.cc` / `DNA_brush_types.h`) ──────────────
    /// How the pointer path becomes dabs (Blender `stroke_method`, default `Space`,
    /// `DNA_brush_types.h:213`).
    pub stroke_method: StrokeMethod,
    /// "Adjust Strength for Spacing" — normalise total deposited opacity so a densely-spaced
    /// stroke doesn't pile up to full opacity (Blender `BRUSH_SPACE_ATTEN`, default ON,
    /// `DNA_brush_types.h:206`). Only applies for spacing < 100% (see [`Self::space_overlap_factor`]).
    pub space_attenuation: bool,
    /// **Accumulate** (Blender `BRUSH_ACCUMULATE`, bit 13, default OFF): when OFF, a single stroke is
    /// capped at [`Self::strength`] — overlapping dabs (incl. passing the brush back over itself) do
    /// NOT build past Strength (a per-stroke coverage mask, the classic "alpha mask"). When ON, every
    /// dab adds on top so opacity builds up without the per-stroke cap (airbrush-like buildup). At
    /// Strength `1.0` the two are identical (the cap is full coverage). See [`crate::dab`].
    pub accumulate: bool,
    /// Dash "on" fraction of each dash period, `0..1` (Blender `dash_ratio`, default `1.0` = solid,
    /// `DNA_brush_types.h:275`).
    pub dash_ratio: f32,
    /// Dash period length in dab-slots (Blender `dash_samples`, default `20`,
    /// `DNA_brush_types.h:276`). `0` is treated as "no dash" (solid).
    pub dash_samples: u32,
    /// Unit for [`Self::jitter`] / [`Self::jitter_absolute_px`] (Blender `BRUSH_ABSOLUTE_JITTER`).
    pub jitter_unit: JitterUnit,
    /// Random per-dab offset in pixels when [`Self::jitter_unit`] is [`JitterUnit::View`]
    /// (Blender `jitter_absolute`, default `0`, `DNA_brush_types.h:223`). Max radial offset
    /// ≈ `2 × jitter_absolute_px`, independent of brush size.
    pub jitter_absolute_px: f32,
    /// Number of most-recent raw input samples box-averaged before processing, `>= 1`
    /// (Blender `input_samples`, default `1`, `DNA_brush_types.h:216`).
    pub input_samples: u32,
    /// Stroke **stabilizer** intensity, `0..1` — a single "how regular" knob (substitute for
    /// Blender's smooth-stroke). `0` = the raw path (straight chords through every sample, exact,
    /// real-time); higher = a stronger lazy-mouse position filter (removes hand tremor) plus more
    /// inter-sample curvature, so a shaky hand draws a clean, regular line. Real-time (causal); the
    /// lag is proportional to the intensity the artist dials in. See [`Stroke`].
    pub stabilizer: f32,
    /// Airbrush emission period in seconds (Blender `rate`, default `0.1` = 10 Hz,
    /// `DNA_brush_types.h:232`). Used only by [`StrokeMethod::Airbrush`]; the tool's tick drives it.
    pub airbrush_rate_s: f32,
    /// "Edge to Edge" — Anchored only (Blender `BRUSH_EDGE_TO_EDGE`, `DNA_brush_enums.h:376`).
    /// OFF: the single stamp is centred on the anchor (press point) with radius = drag distance.
    /// ON: it's centred on the midpoint anchor→cursor with half that radius, so the dab spans
    /// edge-to-edge from the press point to the cursor. See the Anchored arm of [`Stroke::extend`].
    pub edge_to_edge: bool,

    // ── Grain slot (Blender `MTex` / `brush_painter_2d_tex_mapping`; = Procreate "Grain") ──────
    /// Brush **Grain**: a per-texel mask that modulates each dab's coverage *inside* the silhouette
    /// (Blender's brush `mtex`; the Procreate Grain). Default [`TextureSettings::kind`] is `None` (no
    /// modulation). The internal field name is `texture` for back-compat; the UI labels it "Grain".
    /// See [`crate::texture`] and `docs/Painter/05_design_dois_slots_textura.md`.
    pub texture: TextureSettings,
    /// **Grain Depth** (Procreate): how strongly the Grain bites, `0..1`. `1.0` (default) = the Grain
    /// fully modulates (`g_eff = g`, the historical behaviour); `0.0` = the Grain has no effect
    /// (`g_eff = 1`, silhouette only). `g_eff = 1 + (g − 1)·depth`. See [`crate::dab`].
    pub grain_depth: f32,

    // ── Shape slot (= Procreate "Shape": the silhouette / tip of the dab) ──────────────────────
    /// Brush **Shape**: the silhouette (alpha tip) of each dab, as an image. When inactive (default
    /// `kind == None`, or `Image` with no pixels loaded) the silhouette is the procedural
    /// [`Self::falloff`] — so a default brush is byte-identical to before. When a Shape **image** is
    /// assigned it **replaces** the falloff as the silhouette (a crisp imported tip; the falloff goes
    /// inactive in the UI). Pixels live in `PaintState` (heavy), like the Grain image. The `mapping` is
    /// always View-plane (dab-relative); `angle_deg`/`rake`/`random_angle` rotate the tip frame.
    pub shape: TextureSettings,

    // ── Dab flatten + rotate (Procreate "Shape" panel gizmo; Enio 2026-06-26) ───────────────────
    /// **Flatten** the dab footprint, `0..1`: `0` = round, → `1` = a thin ellipse (squished on the
    /// minor axis). Deforms the falloff envelope AND everything sampled in the footprint (the Shape
    /// silhouette + the View-mapped Grain) into a rotated ellipse — each slot keeps its own relative
    /// Size/Offset/Angle on top. Clamped to [`crate::footprint::DAB_FLATTEN_MAX`] so the minor axis
    /// never collapses. See [`Self::footprint_deform`].
    pub dab_flatten: f32,
    /// **Dab rotation** in whole degrees (`0..=360`) of the flatten/rotate frame — rotates the
    /// elliptical footprint and, even when round, the pattern sampled within it.
    pub dab_angle_deg: u16,

    // ── Per-dab randomize (Blender "Color → Randomize" + two PH2D extras; see [`crate::jitter`]) ──
    /// Master switch for **Randomize Color** (the subsection's enable checkbox). When off, every dab
    /// uses [`Self::color`] unchanged. Default `false` (clean-room model of Blender's brush
    /// `BRUSH_USE_RAND_HUE`/`SAT`/`VAL` group toggle).
    pub color_jitter_enabled: bool,
    /// **Hue** randomize amount, `0..1` — the per-dab hue offset is `±amount` on the colour wheel.
    pub color_jitter_hue: f32,
    /// **Saturation** randomize amount, `0..1` — per-dab offset `±amount`, clamped to `[0, 1]`.
    pub color_jitter_sat: f32,
    /// **Value** randomize amount, `0..1` — per-dab offset `±amount`, clamped to `[0, 1]`.
    pub color_jitter_val: f32,
    /// **Jitter Scale** (PH2D, not Blender): per-dab radius scatter, `0..1`. Each dab's radius is
    /// multiplied by `1 ± amount`, clamped so it never collapses. `0` = no scatter.
    pub jitter_scale: f32,
    /// **Jitter Rotate** (PH2D, not Blender): per-dab texture-rotation scatter, `0..1`. Each dab's
    /// texture frame is rotated by `±(amount · 180°)`. Only visible with a texture (round dabs are
    /// isotropic) and only for mappings that use per-dab rotation (not Stencil). `0` = no scatter.
    pub jitter_rotate: f32,
    /// **Jitter Spacing** (PH2D, not Blender): per-gap scatter of the distance between dab centres,
    /// `0..1`. Each gap along the Space walk is multiplied by `1 ± amount` (clamped so it stays ≥ 1px),
    /// so dabs land irregularly instead of on a perfectly even grid. `0` = even spacing.
    pub jitter_spacing: f32,
}

impl Default for BrushSpec {
    /// A soft round black brush, matching Blender's default "TexDraw": smooth falloff, full
    /// strength/flow, 10% spacing.
    fn default() -> Self {
        Self {
            radius_px: 25.0,
            hardness: 0.0,
            strength: 1.0,
            flow: 1.0,
            spacing: 0.10,
            blend: BrushBlend::Mix,
            falloff: Falloff::Smooth,
            jitter: 0.0,
            color: [0.0, 0.0, 0.0],
            custom_falloff: FalloffCurve::default(),
            stroke_method: StrokeMethod::Space,
            space_attenuation: false, // Adjust Strength off by default (Enio 2026-06-24)
            accumulate: false,
            dash_ratio: 1.0,
            dash_samples: 20,
            jitter_unit: JitterUnit::Brush,
            jitter_absolute_px: 0.0,
            input_samples: 1,
            stabilizer: 0.5,
            airbrush_rate_s: 0.1,
            edge_to_edge: false,
            texture: TextureSettings::default(),
            grain_depth: 1.0,
            shape: TextureSettings::default(),
            dab_flatten: 0.0,
            dab_angle_deg: 0,
            color_jitter_enabled: false,
            color_jitter_hue: 0.0,
            color_jitter_sat: 0.0,
            color_jitter_val: 0.0,
            jitter_scale: 0.0,
            jitter_rotate: 0.0,
            jitter_spacing: 0.0,
        }
    }
}

impl BrushSpec {
    /// Effective dab radius after clamping to the allocation cap.
    #[must_use]
    pub fn clamped_radius(&self) -> f32 {
        self.radius_px.clamp(0.5, MAX_BRUSH_RADIUS_PX)
    }

    /// The baked dab flatten + rotate ([`crate::footprint::FootprintDeform`]) — applied to the
    /// footprint of the falloff, the Shape silhouette and the View-mapped Grain so they deform together.
    #[must_use]
    pub fn footprint_deform(&self) -> crate::footprint::FootprintDeform {
        crate::footprint::FootprintDeform::new(self.dab_flatten, self.dab_angle_deg)
    }

    /// Whether **Randomize Color** has any non-zero amount (so enabling it would actually change a
    /// dab). Used to gate the RNG draw in [`crate::jitter::per_dab`].
    #[must_use]
    pub fn has_colour_jitter_amount(&self) -> bool {
        self.color_jitter_hue > 0.0 || self.color_jitter_sat > 0.0 || self.color_jitter_val > 0.0
    }

    /// Whether this brush rotates each dab's texture frame independently (Jitter Rotate), so the
    /// constant-orientation stamp caches must be bypassed (each dab needs its own basis). Only
    /// meaningful with a texture that uses per-dab rotation (i.e. not Stencil); Drag Dot / Anchored
    /// opt out of all per-dab jitter, so they never trigger it.
    #[must_use]
    pub fn has_per_dab_rotation(&self) -> bool {
        self.jitter_rotate > 0.0
            && self.stroke_method.allows_jitter()
            && self.texture.is_active()
            && self.texture.mapping.uses_dab_rotation()
    }

    /// Whether the **Shape** slot supplies the silhouette (else the [`Self::falloff`] does). True when
    /// a shape kind is assigned AND — for the `Image` kind — the pixels are present: an Image shape with
    /// no image falls back to the falloff, so the brush never paints a blank silhouette. `has_shape_image`
    /// is whether the tool currently holds Shape pixels (the heavy buffer lives in `PaintState`).
    #[must_use]
    pub fn shape_silhouette_active(&self, has_shape_image: bool) -> bool {
        self.shape.is_active()
            && (self.shape.kind != crate::texture::TextureKind::Image || has_shape_image)
    }

    /// Grain Depth clamped to `[0, 1]`. The per-texel grain value `g` becomes `1 + (g − 1)·depth`, so
    /// `depth = 1` is the historical full-bite behaviour and `depth = 0` disables the grain.
    #[must_use]
    pub fn grain_depth(&self) -> f32 {
        self.grain_depth.clamp(0.0, 1.0)
    }

    /// Compose the dab silhouette from a Shape sample `shape_val` and the round `falloff` envelope. The
    /// **Image** kind REPLACES the falloff (a crisp finite tip stays uneroded); any **procedural** kind is
    /// MASKED BY it (`falloff × pattern`, so the soft round envelope shapes the texture — Enio 2026-06-25).
    /// `None` never reaches here (the caller uses the bare falloff when the Shape is inactive). The single
    /// source for all three stamp paths (per-pixel, scale-invariant bake, canvas-cached) so they agree.
    #[must_use]
    pub fn compose_shape_silhouette(&self, shape_val: f32, falloff: f32) -> f32 {
        crate::texture::compose_shape_silhouette_kind(self.shape.kind, shape_val, falloff)
    }

    /// Whether the **Shape** slot rotates its silhouette frame per dab (Rake follows the stroke, or a
    /// per-dab Random angle), so the constant-orientation caches can't apply — each dab needs its own
    /// Shape basis. Only meaningful when the Shape is the active silhouette. Mirrors
    /// [`Self::has_per_dab_rotation`] for the Grain.
    #[must_use]
    pub fn shape_has_per_dab_rotation(&self, has_shape_image: bool) -> bool {
        self.shape_silhouette_active(has_shape_image)
            && (self.shape.rake || self.shape.random_angle)
            && self.shape.mapping.uses_dab_rotation()
    }

    /// Whether the **Grain** slot rotates its texture frame per dab (Rake follows the stroke, or a per-dab
    /// Random angle) — each dab needs its own Grain basis. Mirrors [`Self::shape_has_per_dab_rotation`]
    /// for the Grain; complements [`Self::has_per_dab_rotation`] (which is the Grain **Jitter Rotate**).
    #[must_use]
    pub fn grain_has_per_dab_rotation(&self) -> bool {
        self.texture.is_active()
            && (self.texture.rake || self.texture.random_angle)
            && self.texture.mapping.uses_dab_rotation()
    }

    /// Whether the dab is eligible for the **scale-invariant cached stamp** ([`crate::stamp`]) given
    /// both slots: the silhouette must be dab-relative-constant (the falloff always is; a Shape image is
    /// when View-static) AND the Grain must be cacheable (None or static View). `has_shape_image` gates
    /// whether an `Image` shape counts as active. Mirrors [`crate::TextureSettings::is_cacheable`] for
    /// the Grain, extended with the Shape. (Per-dab rotation / Accumulate gating stays at the call site.)
    #[must_use]
    pub fn dab_mask_cacheable(&self, has_shape_image: bool) -> bool {
        let shape_static = !self.shape_silhouette_active(has_shape_image)
            || (matches!(
                self.shape.mapping,
                crate::texture::TextureMapping::ViewPlane
            ) && !self.shape.rake
                && !self.shape.random_angle);
        shape_static && self.texture.is_cacheable()
    }

    /// Distance between dab centres in pixels, derived from spacing × diameter.
    /// At least one pixel so a stroke always advances.
    #[must_use]
    pub fn dab_spacing_px(&self) -> f32 {
        (self.spacing.max(0.01) * 2.0 * self.clamped_radius()).max(1.0)
    }

    /// Whether the dab at dash-slot `slot` is painted, given the dash pattern.
    ///
    /// Behavioural reference: `paint_stroke.cc::add_step` (`dash = (slot % dash_samples) /
    /// dash_samples`; skip when `dash > dash_ratio`). `dash_samples == 0` ⇒ no dash (always on).
    /// With the default `dash_ratio = 1.0` every slot is on (solid).
    #[must_use]
    pub fn dash_on(&self, slot: u32) -> bool {
        if self.dash_samples == 0 {
            return true;
        }
        let dash = (slot % self.dash_samples) as f32 / self.dash_samples as f32;
        dash <= self.dash_ratio.clamp(0.0, 1.0)
    }

    /// "Adjust Strength for Spacing" multiplier applied to each dab's coverage, in `(0, 1]`.
    ///
    /// Behavioural reference (clean-room): `paint_stroke.cc::paint_stroke_integrate_overlap` +
    /// `paint_stroke_overlapped_curve`. Models how many neighbouring dab falloff kernels stack at a
    /// given phase and returns `1 / max_phase(Σ kernels)` so a densely-spaced stroke is normalised to
    /// unit opacity instead of piling up. Returns `1.0` (no attenuation) when the flag is off or
    /// spacing ≥ 100% (Blender's exact gate). Uses this brush's own falloff, like Blender.
    #[must_use]
    pub fn space_overlap_factor(&self) -> f32 {
        // Blender stores spacing as an integer percent; this engine stores a 0..1 fraction.
        let spacing_pct = (self.spacing * 100.0).max(0.0);
        if !(self.space_attenuation && spacing_pct < 100.0) {
            return 1.0;
        }
        // Sample the overlap sum at M phases across one period; the factor cancels the peak.
        const M: usize = 10;
        let g = 1.0 / M as f32;
        let mut max = 0.0_f32;
        for i in 0..M {
            let o = self.overlapped_curve(i as f32 * g, spacing_pct).abs();
            if o > max {
                max = o;
            }
        }
        if max == 0.0 { 1.0 } else { 1.0 / max }
    }

    /// Sum of overlapping dab falloff kernels at phase `x` for a given `spacing_pct`
    /// (`paint_stroke_overlapped_curve`): `n = floor(100 / spacing_pct)` kernels spaced
    /// `h = spacing_pct / 50` apart, each evaluated through this brush's falloff.
    fn overlapped_curve(&self, x: f32, spacing_pct: f32) -> f32 {
        let clamped = spacing_pct.max(0.1);
        let n = (100.0 / clamped) as i32;
        let h = clamped / 50.0;
        let x0 = x - 1.0;
        let mut sum = 0.0;
        for i in 0..n {
            let xx = (x0 + i as f32 * h).abs();
            if xx < 1.0 {
                sum += self.falloff_weight(xx);
            }
        }
        sum
    }

    /// Falloff weight remapped by [`Self::hardness`]. `t = distance / radius`.
    ///
    /// Hardness pushes the falloff outward: for `t < hardness` the weight is full; the curve then
    /// runs over `[hardness, 1]`. `hardness >= 1` yields a hard disk. [`Falloff::Custom`] reads the
    /// editable [`Self::custom_falloff`] profile; every other preset uses its formula.
    #[must_use]
    pub fn falloff_weight(&self, t: f32) -> f32 {
        let h = self.hardness.clamp(0.0, 1.0);
        if h >= 1.0 {
            return if t < 1.0 { 1.0 } else { 0.0 };
        }
        let remapped = ((t - h) / (1.0 - h)).clamp(0.0, 1.0);
        match self.falloff {
            Falloff::Custom => self.custom_falloff.weight(remapped),
            preset => preset.weight(remapped),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let b = BrushSpec::default();
        assert_eq!(b.blend, BrushBlend::Mix);
        assert_eq!(b.falloff, Falloff::Smooth);
        assert!(b.dab_spacing_px() >= 1.0);
        // Per-dab randomize is off by default (so a default stroke is byte-identical to baseline).
        assert!(!b.color_jitter_enabled);
        assert!(!b.has_colour_jitter_amount());
        assert!(!b.has_per_dab_rotation());
        assert_eq!(b.jitter_scale, 0.0);
        assert_eq!(b.jitter_rotate, 0.0);
    }

    #[test]
    fn compose_shape_image_replaces_procedural_masks() {
        use crate::texture::TextureKind;
        let mut b = BrushSpec::default();
        // Image kind: the silhouette IS the image sample — the falloff is ignored (crisp tip).
        b.shape.kind = TextureKind::Image;
        assert_eq!(b.compose_shape_silhouette(0.7, 0.3), 0.7);
        // A procedural kind: the falloff MASKS the pattern (falloff × pattern).
        b.shape.kind = TextureKind::Checker;
        assert!((b.compose_shape_silhouette(0.7, 0.3) - 0.21).abs() < 1e-6);
        // Falloff 0 (dab edge) zeroes a procedural silhouette, but never an Image one.
        assert_eq!(b.compose_shape_silhouette(1.0, 0.0), 0.0);
        b.shape.kind = TextureKind::Image;
        assert_eq!(b.compose_shape_silhouette(1.0, 0.0), 1.0);
    }

    #[test]
    fn radius_clamped() {
        let b = BrushSpec {
            radius_px: 999_999.0,
            ..Default::default()
        };
        assert_eq!(b.clamped_radius(), MAX_BRUSH_RADIUS_PX);
        let b = BrushSpec {
            radius_px: 0.0,
            ..Default::default()
        };
        assert_eq!(b.clamped_radius(), 0.5);
    }

    #[test]
    fn hardness_full_is_hard_disk() {
        let b = BrushSpec {
            hardness: 1.0,
            ..Default::default()
        };
        assert_eq!(b.falloff_weight(0.0), 1.0);
        assert_eq!(b.falloff_weight(0.99), 1.0);
        assert_eq!(b.falloff_weight(1.0), 0.0);
    }

    #[test]
    fn hardness_plateau_then_falls() {
        let b = BrushSpec {
            hardness: 0.5,
            falloff: Falloff::Linear,
            ..Default::default()
        };
        assert_eq!(b.falloff_weight(0.5), 1.0); // inside plateau
        // At t=0.75, remapped = (0.75-0.5)/0.5 = 0.5 → linear weight 0.5.
        assert!((b.falloff_weight(0.75) - 0.5).abs() < 1e-6);
    }
}
