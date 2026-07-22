//! The spec's **derived questions** (child of [`super`], split for the
//! workspace file-LOC cap): every `BrushSpec` method that ANSWERS something
//! about the authored knobs — clamps, effective values, capability predicates,
//! spacing and falloff weights. The knobs themselves (the struct, its consts,
//! the material bundle) stay in the parent; nothing here stores state.

use super::*;

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
        // Jitter Rotate spins the WHOLE dab footprint (falloff + Shape + View-Grain) about the centre. It
        // only *shows* when the footprint is not rotation-invariant — i.e. an anisotropic footprint
        // (`dab_flatten > 0`) OR a texture that rides the dab frame. This used to require
        // `texture.is_active()`, so a FLATTENED, untextured dab with Jitter Rotate looked constant to the
        // guard and Smear/Blur/Clone served it the cached (constant-orientation) mask: every dab came out
        // with the SAME ellipse angle and the knob did nothing (sweep 2026-07-12). The paint path never had
        // the bug — with both slots off it has no cache to serve and resolves `d.rotation` per pixel.
        self.jitter_rotate > 0.0
            && self.stroke_method.allows_jitter()
            && (self.dab_flatten > 0.0
                || (self.texture.is_active() && self.texture.mapping.uses_dab_rotation()))
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

    /// Effective **Granulation** `[0, 1]` (the watercolor deposit gate on the Grain, [`crate::texture::grain_coverage`]).
    /// Zero unless the Watercolor section is on, so a non-watercolor brush keeps the plain Grain multiply
    /// (byte-identical). Pair with a canvas-anchored Grain (`Tiled` mapping + `Grain` kind) for paper granulation.
    #[must_use]
    pub fn effective_granulation(&self) -> f32 {
        if self.watercolor {
            self.granulation.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective **Pigment mix** `[0, 1]` (how much the dab composites subtractively — RYB, Gossett &
    /// Chen 2004 — vs the plain blend; [`crate::blend::blend_over_pigment`]). Zero unless BOTH the
    /// Watercolor section and the Pigment toggle are on, so a normal brush blends exactly as before
    /// (byte-identical). Non-zero ⇒ wet-on-wet mixes like real paint (blue + yellow → green).
    #[must_use]
    pub fn effective_pigment_mix(&self) -> f32 {
        if self.watercolor && self.pigment {
            self.pigment_mix.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's dabs deposit **height** (the impasto relief). The master switch must be on,
    /// the brush must be set to write depth, and the depth must be non-zero — a zero-depth dab is a
    /// no-op *by definition* (it neither lifts nor carves), so the height pass skips it entirely rather
    /// than writing a flat zero over relief that is already there.
    #[must_use]
    pub fn deposits_height(&self) -> bool {
        self.impasto && self.impasto_draw_to.writes_depth() && self.impasto_depth != 0.0
    }

    /// Whether this brush's dabs touch the height field **at all** — it lays body down, or it shoves
    /// existing body around, or both.
    ///
    /// Not the same question as [`Self::deposits_height`], and the difference is a real brush: at
    /// **Depth 0 with Push up** the brush carries no paint and still moves the paint it finds. That is a
    /// dry brush, and it is a palette knife — the tool that does nothing BUT displace. Gating the height
    /// pass on the deposit alone made that brush a no-op, and it is the most physical use of Push there is.
    #[must_use]
    pub fn touches_height(&self) -> bool {
        self.deposits_height() || (self.impasto && self.effective_impasto_push() > 0.0)
    }

    /// Whether this brush's dabs deposit **pigment**. Only [`DrawTo::Depth`] — *with the master switch
    /// on* — suppresses it. With impasto off, [`Self::impasto_draw_to`] is not read at all, so a brush
    /// left on "Depth" in a previous session paints normally again the moment impasto is unticked; the
    /// master switch is the single gate, which is what makes the off state byte-identical.
    #[must_use]
    pub fn deposits_color(&self) -> bool {
        !self.impasto || self.impasto_draw_to.writes_color()
    }

    /// Effective impasto **Depth**, clamped to `[-1, 1]`. Zero unless the master switch is on, so every
    /// caller can read this one number without re-checking the gate.
    #[must_use]
    pub fn effective_impasto_depth(&self) -> f32 {
        if self.impasto {
            self.impasto_depth.clamp(-1.0, 1.0)
        } else {
            0.0
        }
    }

    /// Effective impasto **Smoothing** `[0, 1]`. Zero unless the master switch is on.
    #[must_use]
    pub fn effective_impasto_smoothing(&self) -> f32 {
        if self.impasto {
            self.impasto_smoothing.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Whether this brush's Smear should drag relief: a non-zero **Plow**. Independent of the `impasto`
    /// master switch — the knife moves paint that is already down, whoever laid it.
    #[must_use]
    pub fn impasto_plow_active(&self) -> bool {
        self.impasto_plow > 0.0
    }

    /// Effective impasto **Plow** `[0, 1]`.
    #[must_use]
    pub fn effective_impasto_plow(&self) -> f32 {
        self.impasto_plow.clamp(0.0, 1.0)
    }

    /// Effective impasto **Body** `[0, 1]` — how far the deposit is pushed through the body curve
    /// (`1` = plateau + wall; `0` = the silhouette's own cross-section). No master-switch gate: with
    /// impasto off the kernel is never reached, and the value has no other reader.
    #[must_use]
    pub fn effective_impasto_body(&self) -> f32 {
        self.impasto_body.clamp(0.0, 1.0)
    }

    /// [`Self::impasto_push`], clamped — how much of the ground this brush shoves aside.
    #[must_use]
    pub fn effective_impasto_push(&self) -> f32 {
        self.impasto_push.clamp(0.0, 1.0)
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

    /// Whether the **Shape** slot rotates its silhouette frame per dab — Rake (follows the stroke) OR the
    /// Stroke **Jitter Rotate** (which spins the WHOLE dab stamp, Shape + Grain together) — so the
    /// constant-orientation caches can't apply (each dab needs its own Shape basis). Only meaningful when
    /// the Shape is the active silhouette. Mirrors [`Self::has_per_dab_rotation`].
    #[must_use]
    pub fn shape_has_per_dab_rotation(&self, has_shape_image: bool) -> bool {
        self.shape_silhouette_active(has_shape_image)
            && self.shape.mapping.uses_dab_rotation()
            && (self.shape.rake
                || self.shape.flow
                || (self.jitter_rotate > 0.0 && self.stroke_method.allows_jitter()))
    }

    /// Whether the **Grain** slot rotates its texture frame per dab (Rake follows the stroke) — each dab
    /// needs its own Grain basis. Mirrors [`Self::shape_has_per_dab_rotation`] for the Grain; complements
    /// [`Self::has_per_dab_rotation`] (which is the Grain **Jitter Rotate**).
    #[must_use]
    pub fn grain_has_per_dab_rotation(&self) -> bool {
        self.texture.is_active() && self.texture.rake && self.texture.mapping.uses_dab_rotation()
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
                && !self.shape.flow);
        // Film AA (BUGS #16) is measured in CANVAS texels, and the cached mask is radius-independent —
        // a baked AA rim would be wrong at every other radius, so an AA'd film routes per-pixel.
        shape_static
            && self.texture.is_cacheable()
            && !self.film_aa_wanted(self.shape_silhouette_active(has_shape_image))
    }

    /// Whether the film's screen-space AA ([`crate::height_film::FilmAa`], BUGS #16) applies to this
    /// brush's dabs — the ONE door every consumer asks (the dab/height kernels build the plan from
    /// it, the stamp route refuses the radius-independent cached mask, and the Accumulate-OFF cap
    /// arms so overlapping dabs can't build a rim texel past its area fraction). A Shape silhouette
    /// is a STAMP and keeps its hard edge by design (the `body_edge_t` precedent).
    #[must_use]
    pub fn film_aa_wanted(&self, shape_active: bool) -> bool {
        self.impasto_smooth_edges && self.deposits_height() && !shape_active
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
