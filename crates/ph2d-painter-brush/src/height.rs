//! **Impasto**: the brush's height channel — the paint's own thickness.
//!
//! The engine paints two things per dab, from **one** kernel: colour and, when
//! [`crate::BrushSpec::impasto`] is on, a height `h`. The height is a *second output* of the dab
//! pipeline that already exists — it consumes the same dab list (already mirrored by Symmetry,
//! already replicated by Tiling) and the same [`crate::StampMask`] (silhouette × grain) that the
//! colour consumes. That is what makes Shape / Shape-Tone / Grain / Falloff / Stroke / Jitter /
//! Mirror / Tiling work under impasto **for free** — see `docs/Painter/16_impasto_plano_implementacao.md` §0.
//!
//! `h` is a signed `f32`: positive lifts paint off the canvas, negative carves into it.
//!
//! Not a lighting module — the light pass is the compositor's (`impasto_pass`). This is only the
//! *material*: what the brush deposits.

/// Where a dab's height comes from — which part of the dab mask the colour path already built
/// (`silhouette × grain`) sculpts the relief.
///
/// **Two sources, not three.** The design doc listed a third, `Shape` ("the silhouette sample
/// alone"), but for every brush an artist actually builds it is a *silent duplicate* of `Uniform`:
/// with no Shape slot the silhouette IS the falloff, and with an Image Shape the image already
/// replaces the falloff, so "silhouette alone" and "grain neutral" are the same number. Shipping it
/// would have been a knob that does nothing — the exact species of bug the 2026-07-12 sweep spent
/// its length exterminating. Cut before it was written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DepthSource {
    /// **Uniform** (default) — the Grain does *not* bite: the relief takes the dab's own silhouette
    /// profile (`w`). A soft round brush lays a smooth ridge; a hard tip or a Shape image lays that
    /// tip's profile. Constant across the falloff's plateau, so the interior of a stroke is level —
    /// paint laid by a loaded, smooth brush. The Grain still textures the *pigment*; it just doesn't
    /// shape the *body*.
    #[default]
    Uniform,
    /// **Grain** — the full dab mask (`w × g`): the Grain's striations become bristle marks in the
    /// relief, so the height varies *inside* the dab. This is the real impasto brush (Corel Painter's
    /// bristle depth, ArtRage's loaded brush): the grain's valleys are where the tuft left less paint.
    Grain,
}

impl DepthSource {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 2;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Uniform => 0,
            Self::Grain => 1,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::Uniform`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Grain,
            _ => Self::Uniform,
        }
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Uniform => "Uniform",
            Self::Grain => "Grain",
        }
    }
}

/// Which channels a dab writes — colour, height, or both.
///
/// Lets one brush be a pure *sculpting* tool (relief with no pigment: the palette knife that
/// spreads clear medium) or a pure *painting* tool over existing relief.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawTo {
    /// **Color + Depth** (default): the dab paints pigment and lays down thickness — the ordinary
    /// loaded brush.
    #[default]
    ColorAndDepth,
    /// **Color** only: pigment with no thickness. Equivalent to impasto off *for this brush*, but
    /// keeps the impasto settings around so the artist can flip back without re-dialling them.
    Color,
    /// **Depth** only: thickness with no pigment — the canvas RGBA is left byte-identical and only
    /// the height field changes. Sculpt clear medium, or carve (negative depth) into paint that is
    /// already down.
    Depth,
}

impl DrawTo {
    /// Number of variants (the panel cycler iterates `0..COUNT`).
    pub const COUNT: u8 = 3;

    /// Stable wire discriminant.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::ColorAndDepth => 0,
            Self::Color => 1,
            Self::Depth => 2,
        }
    }

    /// Inverse of [`Self::to_u8`]; unknown values fall back to [`Self::ColorAndDepth`].
    #[must_use]
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Color,
            2 => Self::Depth,
            _ => Self::ColorAndDepth,
        }
    }

    /// Whether a dab with this setting deposits **pigment**. `false` ⇒ the colour path must leave the
    /// canvas RGBA untouched.
    #[must_use]
    pub fn writes_color(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Color)
    }

    /// Whether a dab with this setting deposits **height**. `false` ⇒ the height field is untouched.
    #[must_use]
    pub fn writes_depth(self) -> bool {
        matches!(self, Self::ColorAndDepth | Self::Depth)
    }

    /// Short label for the panel cycler (English; HR-15).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::ColorAndDepth => "Color + Depth",
            Self::Color => "Color",
            Self::Depth => "Depth",
        }
    }
}

/// One dab's inputs to the height kernel — the *same* resolved frames the colour kernel is handed for
/// that dab (its footprint, its Shape basis, its Grain basis). The caller resolves them once and gives
/// both kernels the same ones; that is the whole trick.
#[derive(Clone, Copy)]
pub struct HeightDab<'a> {
    /// Dab centre in canvas pixels. Already wrapped by Tiling and mirrored by Symmetry — the height
    /// kernel consumes the **dab list**, it never re-derives geometry.
    pub center: [f32; 2],
    /// Dab radius in canvas pixels (this dab's, after Jitter Scale).
    pub radius: f32,
    /// Per-dab dynamics (pressure); folded with Flow × Strength exactly as the colour kernel folds it,
    /// so a light touch lays *thinner* paint, not just fainter paint.
    pub coverage: f32,
    /// This dab's flatten/rotate footprint (incl. Jitter Rotate).
    pub footprint: crate::footprint::FootprintDeform,
    /// The Shape slot's resolved frame + pixels, or `None` when the falloff is the silhouette.
    pub shape: Option<crate::dab::ShapeInput<'a>>,
    /// The Grain's resolved frame — read only when [`DepthSource::Grain`] is selected.
    pub grain: Option<&'a crate::texture::TexDabBasis>,
    /// The Grain's pixels (an `Image` Grain).
    pub grain_image: Option<&'a crate::texture::ImageMask<'a>>,
}

/// Combine an existing height with a newly-deposited one — the **stroke envelope**.
///
/// The larger *magnitude* wins, so passing the brush back over its own stroke does not stack up a
/// staircase of paint (one pass of a loaded brush leaves one thickness), and a carving brush
/// (negative depth) deepens rather than being cancelled by the `max` of two negatives. Separate
/// strokes DO add — that is the caller's job (the per-stroke envelope is merged into the layer at
/// stroke end), not this function's.
#[inline]
#[must_use]
pub fn envelope(a: f32, b: f32) -> f32 {
    if b.abs() > a.abs() { b } else { a }
}

/// Deposit one dab's height into the per-stroke envelope `dst` (canvas-sized, `width × height` f32).
///
/// Reads the **same** silhouette and grain the colour kernel reads, through the same
/// [`crate::dab::silhouette_at`] / [`crate::dab::grain_at`]. Returns the touched rect, or `None` if
/// the dab is off-canvas / deposits nothing.
///
/// The deposited height is `depth × coverage × w` (or `× w × g` for [`DepthSource::Grain`]) — thickness
/// is proportional to how much paint the dab actually lays down, which is why every knob that shapes
/// the dab shapes the relief.
#[must_use]
pub fn accumulate_dab_height(
    dst: &mut [f32],
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<crate::dab::DirtyRect> {
    // Contract guard — a real early-out, not a `debug_assert` that vanishes from the build the artist
    // runs (the lesson of the 2026-07-12 SIGSEGV).
    if dst.len() < (width as usize) * (height as usize) || width == 0 || height == 0 {
        return None;
    }
    let depth = spec.effective_impasto_depth();
    if depth == 0.0 {
        return None;
    }
    // The same fold the colour kernel applies: pressure × Flow × Strength. A light, thin stroke is
    // both fainter AND thinner — one number drives both.
    let coverage =
        dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let use_grain = matches!(spec.impasto_source, DepthSource::Grain) && dab.grain.is_some();
    let mut touched = false;
    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let t = dab.footprint.falloff_t(dx * inv_radius, dy * inv_radius);
            let w = crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius);
            if w <= 0.0 {
                continue;
            }
            let mut a = w;
            if use_grain && let Some(b) = dab.grain {
                a *= crate::dab::grain_at(spec, b, dab.grain_image, px, py, dab.center, radius);
            }
            let h = depth * coverage * a;
            if h == 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            dst[i] = envelope(dst[i], h);
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// **Erase** one dab's footprint from the height field: the relief is scrubbed away in proportion to
/// the dab's coverage, exactly where the eraser removes pigment.
///
/// Not optional, and not the same as depositing a negative depth (that would *carve*). Without it the
/// eraser leaves **ghost relief**: the paint is gone but the light still reports a ridge. Reads the
/// same silhouette as the colour path, so an erase with a Shape tip erases that tip's profile.
#[must_use]
pub fn erase_dab_height(
    dst: &mut [f32],
    width: u32,
    height: u32,
    spec: &crate::BrushSpec,
    dab: &HeightDab<'_>,
) -> Option<crate::dab::DirtyRect> {
    if dst.len() < (width as usize) * (height as usize) || width == 0 || height == 0 {
        return None;
    }
    let coverage =
        dab.coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 {
        return None;
    }
    let radius = dab.radius.max(0.5);
    let (cx, cy) = (dab.center[0], dab.center[1]);
    let x0 = (cx - radius).floor().max(0.0) as i64;
    let y0 = (cy - radius).floor().max(0.0) as i64;
    let x1 = ((cx + radius).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    let inv_radius = 1.0 / radius;
    let mut touched = false;
    for py in y0..y1 {
        let dy = (py as f32 + 0.5) - cy;
        for px in x0..x1 {
            let dx = (px as f32 + 0.5) - cx;
            let t = dab.footprint.falloff_t(dx * inv_radius, dy * inv_radius);
            let w = crate::dab::silhouette_at(spec, dab.shape, t, px, py, dab.center, radius);
            if w <= 0.0 {
                continue;
            }
            let i = (py as usize) * (width as usize) + px as usize;
            if dst[i] == 0.0 {
                continue;
            }
            dst[i] *= 1.0 - (w * coverage).clamp(0.0, 1.0);
            touched = true;
        }
    }
    if !touched {
        return None;
    }
    Some(crate::dab::DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_discriminants_round_trip() {
        for v in 0..DepthSource::COUNT {
            assert_eq!(DepthSource::from_u8(v).to_u8(), v);
        }
        for v in 0..DrawTo::COUNT {
            assert_eq!(DrawTo::from_u8(v).to_u8(), v);
        }
        // Unknown wire values fall back to the default, never panic.
        assert_eq!(DepthSource::from_u8(200), DepthSource::default());
        assert_eq!(DrawTo::from_u8(200), DrawTo::default());
    }

    #[test]
    fn default_draw_to_writes_both_channels() {
        let d = DrawTo::default();
        assert!(d.writes_color() && d.writes_depth());
        assert!(DrawTo::Color.writes_color() && !DrawTo::Color.writes_depth());
        assert!(!DrawTo::Depth.writes_color() && DrawTo::Depth.writes_depth());
    }

    use crate::texture::{TextureKind, TextureMapping};
    use crate::{BrushSpec, Falloff};

    const W: u32 = 33;

    /// A dab at the canvas centre with the given spec; returns the deposited height field.
    fn deposit(spec: &BrushSpec) -> Vec<f32> {
        let mut dst = vec![0.0f32; (W * W) as usize];
        let footprint = spec.footprint_deform();
        let basis = crate::texture::dab_basis(
            &spec.texture,
            [1.0, 0.0],
            &mut 0u64,
            [W as f32, W as f32],
            [1.0, 0.0],
            footprint,
        );
        let dab = HeightDab {
            center: [16.5, 16.5],
            radius: 12.0,
            coverage: 1.0,
            footprint,
            shape: None,
            grain: Some(&basis),
            grain_image: None,
        };
        let _ = accumulate_dab_height(&mut dst, W, W, spec, &dab);
        dst
    }

    /// The heights on the falloff's PLATEAU — the dab's flat interior, where Uniform must be level and
    /// Grain must not be. `hardness = 0.6` ⇒ `w == 1` for `t < 0.6`, i.e. within 7 px of the centre.
    fn plateau(h: &[f32]) -> Vec<f32> {
        let mut out = Vec::new();
        for y in 0..W as i32 {
            for x in 0..W as i32 {
                let (dx, dy) = (f64::from(x) + 0.5 - 16.5, f64::from(y) + 0.5 - 16.5);
                if (dx * dx + dy * dy).sqrt() < 6.0 {
                    out.push(h[(y as usize) * (W as usize) + x as usize]);
                }
            }
        }
        out
    }

    #[test]
    fn depth_source_uniform_is_level_and_grain_is_not() {
        // The gate the plan froze: Uniform lays a LEVEL plateau (the grain textures the pigment, not
        // the body); Grain lets the grain's striations into the relief, so the height VARIES inside the
        // dab. This is the whole difference between a smooth loaded brush and a bristle brush — if the
        // two ever produced the same field, `DepthSource` would be a dead knob.
        let mut base = BrushSpec {
            impasto: true,
            impasto_depth: 1.0,
            hardness: 0.6,
            falloff: Falloff::Smooth,
            radius_px: 12.0,
            ..Default::default()
        };
        base.texture.kind = TextureKind::Noise; // a Grain with real per-texel variation
        base.texture.mapping = TextureMapping::ViewPlane;

        let uniform = deposit(&BrushSpec {
            impasto_source: DepthSource::Uniform,
            ..base
        });
        let grain = deposit(&BrushSpec {
            impasto_source: DepthSource::Grain,
            ..base
        });

        let u_plateau = plateau(&uniform);
        let g_plateau = plateau(&grain);
        assert!(!u_plateau.is_empty(), "the fixture sampled a plateau");

        let spread = |v: &[f32]| {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), &x| (lo.min(x), hi.max(x)));
            hi - lo
        };
        assert!(
            spread(&u_plateau) < 1e-6,
            "Uniform: the plateau must be LEVEL — the Grain shapes pigment, not body (spread {})",
            spread(&u_plateau)
        );
        assert!(
            spread(&g_plateau) > 0.05,
            "Grain: the striations must reach the relief (spread {})",
            spread(&g_plateau)
        );
        // And the plateau really is at full depth (nothing silently scaled it away).
        assert!(
            (u_plateau[0] - 1.0).abs() < 1e-6,
            "full depth on the plateau"
        );
    }

    #[test]
    fn envelope_is_one_pass_of_paint_and_carving_deepens() {
        // Passing the brush back over its own stroke leaves ONE thickness, not a staircase — the
        // per-stroke envelope. And a carving brush (negative depth) must DEEPEN under the same rule:
        // a plain `max` of two negatives would pick the shallower one and the carve would fade.
        assert_eq!(envelope(0.5, 0.5), 0.5, "a second pass adds nothing");
        assert_eq!(envelope(0.3, 0.7), 0.7, "a heavier pass wins");
        assert_eq!(envelope(0.7, 0.3), 0.7, "a lighter pass does not thin it");
        assert_eq!(envelope(-0.3, -0.7), -0.7, "carving deepens");
        assert_eq!(
            envelope(-0.7, -0.3),
            -0.7,
            "a lighter carve does not fill it"
        );
        assert_eq!(
            envelope(0.2, -0.9),
            -0.9,
            "the stronger gesture wins the pixel"
        );
    }

    #[test]
    fn eraser_scrubs_the_relief_it_finds() {
        // Erasing must remove the RELIEF, not carve a hole — otherwise the pigment goes and the light
        // still reports a ridge (ghost relief). Full coverage erases to flat.
        let spec = BrushSpec {
            impasto: true,
            impasto_depth: 1.0,
            hardness: 1.0, // hard disk → deterministic full coverage inside
            falloff: Falloff::Constant,
            radius_px: 12.0,
            ..Default::default()
        };
        let mut field = vec![0.8f32; (W * W) as usize];
        let footprint = spec.footprint_deform();
        let dab = HeightDab {
            center: [16.5, 16.5],
            radius: 12.0,
            coverage: 1.0,
            footprint,
            shape: None,
            grain: None,
            grain_image: None,
        };
        let rect =
            erase_dab_height(&mut field, W, W, &spec, &dab).expect("the eraser touched relief");
        assert!(rect.w > 0 && rect.h > 0);
        let centre = field[(16 * W + 16) as usize];
        assert!(
            centre.abs() < 1e-6,
            "under the dab the relief is gone ({centre})"
        );
        let corner = field[0];
        assert_eq!(corner, 0.8, "outside the dab the relief is untouched");
    }
}
