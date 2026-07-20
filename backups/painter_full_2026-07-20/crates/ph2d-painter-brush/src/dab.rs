//! Stamp one brush dab into an RGBA8 layer buffer.
//!
//! Behavioural reference (clean-room, no code copied): Blender
//! `editors/sculpt_paint/mesh/paint_image_2d.cc` (the soft 2D brush that walks the dab's bounding
//! box, weights each texel by the falloff mask, and blends) + `paint_image_2d_curve_mask.cc` (the
//! per-texel falloff weight). Distance is measured from the dab centre to each **pixel centre**
//! (`px + 0.5`), matching the texel-centre convention.

use crate::ramp_alpha::RampAlphaMode;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

/// The **Shape** slot's per-dab inputs (Procreate "Shape"): the resolved tip frame plus the optional
/// silhouette image. `Some` ⇒ the Shape supplies the dab's silhouette — an Image **replaces** the
/// falloff, a procedural kind is **masked by** it (see [`BrushSpec::compose_shape_silhouette`]); `None`
/// ⇒ the falloff is the silhouette (byte-identical to the pre-Shape engine). The caller only passes
/// `Some` when the Shape is genuinely active (see [`BrushSpec::shape_silhouette_active`]), so an Image
/// shape with no pixels never reaches here. Bundled into one param to keep the stamp signatures
/// tractable (each already carries the Grain's `tex` + `image`).
#[derive(Clone, Copy)]
pub struct ShapeInput<'a> {
    /// The dab-relative texture frame for the Shape (rotation/offset), from [`crate::texture::dab_basis`].
    pub basis: &'a TexDabBasis,
    /// The silhouette pixels for [`crate::TextureKind::Image`]; `None` for a procedural Shape kind.
    pub image: Option<&'a ImageMask<'a>>,
    /// The Shape's **value ramp** LUT (256 grayscale entries, baked from `ph2d_color::ValueRamp`) that
    /// remaps the raw silhouette value (B&W tonal curve / invert); `None` ⇒ no remap. See
    /// [`crate::texture::remap_shape_value`].
    pub ramp_lut: Option<&'a [f32]>,
}

/// The dab's **silhouette** at one canvas pixel — the `w` of the kernel's funnel
/// `a = w · g · coverage`.
///
/// An **Image** Shape *replaces* the falloff (a crisp imported tip stays uneroded); a **procedural**
/// Shape is *masked by* it (`falloff × pattern`); **no** Shape ⇒ the bare falloff. `t` is the
/// normalised distance from the dab centre, already deformed by the dab's flatten/rotate footprint.
///
/// **This is the single source of the dab's shape.** The colour kernel ([`stamp_band`]) and the
/// height kernel ([`crate::height`]) both call it, so pigment and relief can never disagree about
/// what a dab looks like — which is precisely what makes Shape, the Shape-Tone ramp, the falloff and
/// the flatten/rotate footprint sculpt the Impasto relief for free, and keeps them sculpting it after
/// someone edits one of them six months from now.
#[inline]
#[must_use]
pub fn silhouette_at(
    spec: &BrushSpec,
    shape: Option<ShapeInput<'_>>,
    t: f32,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    match shape {
        Some(sh) => {
            let raw = crate::texture::sample_shape_silhouette(
                &spec.shape,
                sh.basis,
                px,
                py,
                center,
                radius,
                sh.image,
            );
            let sv = crate::texture::remap_shape_value(raw, sh.ramp_lut);
            spec.compose_shape_silhouette(sv, spec.falloff_weight(t))
        }
        None => spec.falloff_weight(t),
    }
}

/// The dab's **grain** coverage factor at one canvas pixel — the `g` of the funnel. Folds Grain Depth,
/// the watercolor Granulation gate and the Stencil rect, in that order. The colour kernel inlines this
/// same sequence (it needs the raw sample first, to index a Colour Ramp); the height kernel calls it
/// directly. Every step is itself a shared function, so the two cannot drift.
#[inline]
#[must_use]
pub fn grain_at(
    spec: &BrushSpec,
    basis: &TexDabBasis,
    image: Option<&ImageMask<'_>>,
    px: i64,
    py: i64,
    center: [f32; 2],
    radius: f32,
) -> f32 {
    let s = crate::texture::sample(&spec.texture, basis, px, py, center, radius, image);
    crate::texture::grain_coverage(s, spec.grain_depth(), spec.effective_granulation())
        * crate::texture::stencil_gate(&spec.texture, basis, px, py)
}

/// Axis-aligned region of the buffer touched by a dab, in pixels (half-open: `[x, x+w)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// Stamp one dab centred at `center` (image-space pixel coordinates) into `buf`.
///
/// `buf` is straight-alpha RGBA8, row-major, `width * height * 4` bytes, in the layer's native
/// space. `coverage` is the dab's overall opacity in `[0, 1]` — the stroke engine folds pressure
/// dynamics and the per-stroke strength cap into it; the brush's [`BrushSpec::flow`] and falloff
/// are applied here. Returns the touched [`DirtyRect`], or `None` if the dab is fully off-canvas
/// or has zero coverage.
///
/// When `preserve_alpha` is set ("alpha lock" / "preserve transparency"), each pixel's coverage is
/// scaled by the destination's existing alpha, so paint only lands where the layer already has
/// opacity (it recolours the shape without growing it).
///
/// Panics in debug if `buf` is too small. The texture-free fast path (delegates to [`stamp_dab_textured`]).
#[must_use]
pub fn stamp_dab(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
) -> Option<DirtyRect> {
    stamp_dab_textured(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        None,
        None,
        None,
    )
}

/// As [`stamp_dab`], but the brush texture ([`BrushSpec::texture`]) modulates each texel's coverage when
/// `tex` is `Some`. `tex` is the per-dab frame from [`crate::texture::dab_basis`] (rotation + random
/// offset); `image` supplies the pixels for [`crate::TextureKind::Image`]. `None`/`None` (or an inactive
/// texture) reproduces [`stamp_dab`] exactly. See [`crate::texture`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        RampAlphaMode::None,
        None,
        [1.0, 0.0],
    )
}

/// As [`stamp_dab_textured`], plus the **Accumulate-OFF** per-stroke coverage `mask` (canvas-sized, 1
/// byte/px): with `Some`, each pixel's stroke coverage caps at the dab target (Strength) — overlapping /
/// back-and-forth dabs build toward but never past it. `None` ⇒ exactly [`stamp_dab_textured`].
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_textured_masked(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    mask: Option<&mut [u8]>,
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        None,
        RampAlphaMode::None,
        mask,
        dab_rotation,
    )
}

/// As [`stamp_dab_textured`], but a baked **Color Ramp** LUT (256 straight-RGBA entries) maps each
/// texel's texture value to a COLOUR: the brush paints `lut[t]`'s RGB, so the texture's scalar drives
/// the painted colour (not just attenuating the single colour). The ramp **alpha** does what `alpha_mode`
/// selects ([`RampAlphaMode`]): ignored, scaling coverage, or driving the pixel's own alpha (punch
/// transparent). With no texture there's nothing to index, so it falls back to the plain stamp.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn stamp_dab_ramped(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: &[[f32; 4]],
    alpha_mode: RampAlphaMode,
    // Accumulate-OFF per-stroke cap so a Color-Ramp stroke honours Accumulate (TextureAlpha uncapped), Enio 2026-06-25.
    mask: Option<&mut [u8]>,
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    stamp_dab_inner(
        buf,
        width,
        height,
        center,
        spec,
        coverage,
        preserve_alpha,
        tex,
        image,
        shape,
        Some(ramp),
        alpha_mode,
        mask,
        dab_rotation,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
fn stamp_dab_inner(
    buf: &mut [u8],
    width: u32,
    height: u32,
    center: [f32; 2],
    spec: &BrushSpec,
    coverage: f32,
    preserve_alpha: bool,
    tex: Option<&TexDabBasis>,
    image: Option<&ImageMask>,
    shape: Option<ShapeInput>,
    ramp: Option<&[[f32; 4]]>,
    alpha_mode: RampAlphaMode,
    // **Accumulate OFF** per-stroke coverage mask (canvas-sized, 1 byte/pixel). `Some` ⇒ cap each
    // pixel's stroke coverage at the dab target (the tool passes it only when Accumulate is off and
    // Strength < 1, where the cap is observable); `None` ⇒ the build-up path (Accumulate ON).
    mask: Option<&mut [u8]>,
    // The dab's composed footprint rotor ([`crate::BrushSpec::dab_rotor`], `[1, 0]` = none): the
    // per-dab **Jitter Rotate** spin composed with the **stroke-follow** rotation. It spins the whole
    // footprint — falloff + Shape + View-Grain — together, which is what keeps a following tip and the
    // pattern inside it from disagreeing on a curve.
    dab_rotation: [f32; 2],
) -> Option<DirtyRect> {
    // Contract guard (sweep 2026-07-12): was `debug_assert!`, i.e. absent from the build the artist runs.
    if buf.len() < (width as usize) * (height as usize) * 4 {
        return None;
    }
    // Per-dab opacity = the stroke's coverage × the brush's Flow (per-dab build-up) × Strength
    // (overall opacity). With `mask` set this is the per-stroke CAP (Accumulate off); without it the
    // dab just builds up (Accumulate on). Both default to 1.0.
    let coverage =
        coverage.clamp(0.0, 1.0) * spec.flow.clamp(0.0, 1.0) * spec.strength.clamp(0.0, 1.0);
    if coverage <= 0.0 || width == 0 || height == 0 {
        return None;
    }
    let radius = spec.clamped_radius();
    let (cx, cy) = (center[0], center[1]);

    // Screen-space AA of the film silhouette (BUGS #16, impasto half) — hoisted once per dab;
    // `None` = the single-sample `film_of` path, byte-identical (checkbox off / no body / Shape tip).
    let film_aa = crate::height_film::FilmAa::for_dab(
        spec,
        shape.filter(|_| spec.shape.is_active()).is_some(),
        radius,
    );
    // The AA's outermost fractional ring can live one texel past the geometric rim (a texel whose
    // CENTRE is outside can still be partially covered) — pad the bbox so it is not clipped.
    let aa_pad = crate::height_film::FilmAa::pad_px(&film_aa);

    // Bounding box of the dab, clamped to the canvas (half-open on the max side).
    let x0 = (cx - radius - aa_pad).floor().max(0.0) as i64;
    let y0 = (cy - radius - aa_pad).floor().max(0.0) as i64;
    let x1 = ((cx + radius + aa_pad).ceil() as i64 + 1).min(width as i64);
    let y1 = ((cy + radius + aa_pad).ceil() as i64 + 1).min(height as i64);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }

    let stride = (width as usize) * 4;
    let ctx = DabCtx {
        spec,
        // Texture frame, only when a texture is actually assigned (else the per-pixel sample is
        // skipped entirely so the texture-free path costs nothing).
        tex: tex.filter(|_| spec.texture.is_active()),
        image: image.copied(),
        // Kept even with no Grain — then the silhouette coverage indexes the ramp (Shape's colour ramp).
        ramp,
        // The Shape supplies the silhouette only when the slot is active (the caller already gates
        // this; the filter is belt-and-braces so an inactive Shape can never blank the falloff).
        shape: shape.filter(|_| spec.shape.is_active()),
        alpha_mode,
        footprint: spec.dab_footprint(dab_rotation),
        center,
        cx,
        cy,
        inv_radius: 1.0 / radius,
        radius,
        coverage,
        preserve_alpha,
        x0,
        x1,
        stride,
        film_aa,
    };

    // The per-pixel work is independent, so a LARGE dab (e.g. a big Anchored stamp re-drawn every
    // pointer move) splits across the cores (see `parallel_band_stamp`). The result is bit-identical
    // to serial, so the texture stays fully visible during the drag. The Accumulate-cap path reads+
    // writes the shared per-stroke mask, so it runs SERIALLY (one band over the dab's rows) — small
    // soft-brush dabs anyway, where the cap is observable.
    let touched = match mask {
        Some(mask) => {
            let region = &mut buf[(y0 as usize) * stride..(y1 as usize) * stride];
            let mrow = width as usize;
            let mask_region = &mut mask[(y0 as usize) * mrow..(y1 as usize) * mrow];
            stamp_band(&ctx, region, Some(mask_region), y0)
        }
        None => parallel_band_stamp(buf, y0, y1, x0, x1, stride, |dst, band_y0| {
            stamp_band(&ctx, dst, None, band_y0)
        }),
    };

    if !touched {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// Footprint area (pixels) at or above which a dab/blit splits across cores. Below it, the serial
/// path wins (thread spawn isn't worth ~1 ms of work). ~128k px ≈ a radius-200 dab — small brush
/// dabs (Space) stay serial; large Anchored stamps parallelise.
pub(crate) const PARALLEL_MIN_AREA: usize = 1 << 17;

/// Run `stamp` over the dab's row span `[y0, y1)` — serial for small footprints, split into disjoint
/// row bands across the cores for large ones (≥ [`PARALLEL_MIN_AREA`]). `stamp(dst, band_y0)` writes
/// the full-width band slice `dst` whose first row is `band_y0`, returning whether it touched a
/// pixel. Disjoint bands ⇒ the result is bit-identical to serial regardless of band count (HR-5).
/// Shared by the per-pixel [`stamp_dab_textured`] and the cached-mask [`crate::stamp::blit_stamp`].
pub(crate) fn parallel_band_stamp<F>(
    buf: &mut [u8],
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stride: usize,
    stamp: F,
) -> bool
where
    F: Fn(&mut [u8], i64) -> bool + Sync,
{
    let region = &mut buf[(y0 as usize) * stride..(y1 as usize) * stride];
    let rows = (y1 - y0) as usize;
    let area = rows * ((x1 - x0).max(0) as usize);
    if area < PARALLEL_MIN_AREA || rows <= 1 {
        return stamp(region, y0);
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, rows);
    let rows_per_band = rows.div_ceil(threads);
    let stamp = &stamp;
    std::thread::scope(|s| {
        region
            .chunks_mut(rows_per_band * stride)
            .enumerate()
            .map(|(bi, chunk)| {
                let band_y0 = y0 + (bi * rows_per_band) as i64;
                s.spawn(move || stamp(chunk, band_y0))
            })
            // Collect first so EVERY band is joined (no short-circuit leaving a thread unjoined).
            .collect::<Vec<_>>()
            .into_iter()
            .fold(false, |acc, h| acc | h.join().unwrap_or(false))
    })
}

/// Like [`parallel_band_stamp`] but splits THREE row-aligned buffers across the bands: the RGBA
/// `canvas` (stride `width*4`) plus the canvas-space texture cache `tex` and its `ready` flags (both
/// stride `width`). Each band owns disjoint rows of all three, so the lazy cache fill + the blend run
/// race-free and bit-identical to serial (HR-5). Used by [`crate::stamp::blit_canvas_cached`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn parallel_band_cached<F>(
    canvas: &mut [u8],
    tex: &mut [u8],
    ready: &mut [u8],
    width: u32,
    y0: i64,
    y1: i64,
    x0: i64,
    x1: i64,
    stamp: F,
) -> bool
where
    F: Fn(&mut [u8], &mut [u8], &mut [u8], i64) -> bool + Sync,
{
    let (cstride, mstride) = ((width as usize) * 4, width as usize);
    let canvas = &mut canvas[(y0 as usize) * cstride..(y1 as usize) * cstride];
    let tex = &mut tex[(y0 as usize) * mstride..(y1 as usize) * mstride];
    let ready = &mut ready[(y0 as usize) * mstride..(y1 as usize) * mstride];
    let rows = (y1 - y0) as usize;
    let area = rows * ((x1 - x0).max(0) as usize);
    if area < PARALLEL_MIN_AREA || rows <= 1 {
        return stamp(canvas, tex, ready, y0);
    }
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .clamp(1, rows);
    let rpb = rows.div_ceil(threads);
    let stamp = &stamp;
    std::thread::scope(|s| {
        canvas
            .chunks_mut(rpb * cstride)
            .zip(tex.chunks_mut(rpb * mstride))
            .zip(ready.chunks_mut(rpb * mstride))
            .enumerate()
            .map(|(bi, ((cb, tb), rb))| {
                let band_y0 = y0 + (bi * rpb) as i64;
                s.spawn(move || stamp(cb, tb, rb, band_y0))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .fold(false, |acc, h| acc | h.join().unwrap_or(false))
    })
}

/// Read-only per-dab context shared by every row band of a [`stamp_dab_textured`] stamp. `Sync`
/// (refs to `Sync` data + `Copy` scalars), so `&DabCtx` is safely shared across the band threads.
struct DabCtx<'a> {
    spec: &'a BrushSpec,
    tex: Option<&'a TexDabBasis>,
    image: Option<ImageMask<'a>>,
    /// The Shape slot's silhouette inputs (frame + optional image). `Some` ⇒ the silhouette is the
    /// Shape, replacing the falloff; `None` ⇒ the falloff is the silhouette. See [`ShapeInput`].
    shape: Option<ShapeInput<'a>>,
    /// Baked Color Ramp LUT: a per-texel value (Grain pattern, else the silhouette coverage) indexes it for the paint colour. [`stamp_dab_ramped`].
    ramp: Option<&'a [[f32; 4]]>,
    /// What the ramp colour's alpha does (only meaningful when `ramp` is `Some`). See [`RampAlphaMode`].
    alpha_mode: RampAlphaMode,
    /// Dab flatten/rotate for the falloff `t` (Shape/Grain carry it in their bases); baked once per dab.
    footprint: crate::footprint::FootprintDeform,
    center: [f32; 2],
    cx: f32,
    cy: f32,
    inv_radius: f32,
    radius: f32,
    coverage: f32,
    preserve_alpha: bool,
    x0: i64,
    x1: i64,
    stride: usize,
    /// Screen-space AA of the film silhouette (BUGS #16) — `None` = single-sample `film_of`,
    /// byte-identical. Hoisted per dab in [`stamp_dab_pixels`].
    film_aa: Option<crate::height_film::FilmAa>,
}

/// Stamp the dab's pixels for the full-width row band `dst` whose first row is `band_y0` (so global
/// row `py` lives at local offset `(py - band_y0) * stride`). Returns whether any pixel was written.
/// Pure over disjoint bands — no shared mutable state — so bands run in parallel deterministically.
fn stamp_band(ctx: &DabCtx, dst: &mut [u8], mut mask: Option<&mut [u8]>, band_y0: i64) -> bool {
    let blend = ctx.spec.blend;
    let mut touched = false;
    for r in 0..dst.len() / ctx.stride {
        let py = band_y0 + r as i64;
        let dy = (py as f32 + 0.5) - ctx.cy;
        let row = r * ctx.stride;
        for px in ctx.x0..ctx.x1 {
            // SILHOUETTE — via the shared [`silhouette_at`], which the Impasto height kernel also
            // calls: one definition of a dab's shape, so relief and pigment cannot drift apart.
            let dx = (px as f32 + 0.5) - ctx.cx;
            let t = ctx
                .footprint
                .falloff_t(dx * ctx.inv_radius, dy * ctx.inv_radius);
            // The FILM ([`crate::height::film_coverage`]): a brush laying body lays no pigment where it
            // lays no body. Applied to the SILHOUETTE — before the Grain, before the dynamics — so every
            // funnel below (grain, Accumulate-OFF cap, ramps) inherits the cut with no arithmetic.
            // With Smooth Edges ([`crate::height_film::FilmAa`], BUGS #16) the film at a rim texel is
            // its fractional AREA coverage — every funnel inherits the anti-aliased cut the same way.
            let w = match &ctx.film_aa {
                Some(aa) => aa.film_at(
                    t,
                    silhouette_at(ctx.spec, ctx.shape, t, px, py, ctx.center, ctx.radius),
                    |ox, oy| {
                        ctx.spec.falloff_weight(
                            ctx.footprint
                                .falloff_t((dx + ox) * ctx.inv_radius, (dy + oy) * ctx.inv_radius),
                        )
                    },
                ),
                None => crate::height::film_coverage(
                    ctx.spec.deposits_height(),
                    silhouette_at(ctx.spec, ctx.shape, t, px, py, ctx.center, ctx.radius),
                ),
            };
            // Skip pixels the silhouette already zeroes BEFORE the grain sample — the grain only
            // modulates where the dab paints, so sampling it there is pure waste (large Anchored).
            if w <= 0.0 {
                continue;
            }
            // Default colour = the brush's; a Color Ramp instead indexes the ramp by the texture value for
            // the per-texel COLOUR, its alpha per `ctx.alpha_mode` (none / less coverage / the pixel's alpha).
            let mut color = ctx.spec.color;
            let mut stamp_alpha = 1.0_f32; // mode TextureAlpha: the source pixel's own alpha
            // `w` = dab PROFILE (falloff × silhouette, the build factor); `g` = texture COVERAGE factor
            // (Grain / Strength-ramp) kept SEPARATE so it CAPS the pixel — a 0.3 grain texel tops at
            // 0.3·Strength under re-passing, not climbing to full (the "fills in" bug). `g=1` ⇒ identical.
            let mut g = 1.0_f32;
            if let Some(b) = ctx.tex {
                let s = crate::texture::sample(
                    &ctx.spec.texture,
                    b,
                    px,
                    py,
                    ctx.center,
                    ctx.radius,
                    ctx.image.as_ref(),
                );
                if let Some(lut) = ctx.ramp {
                    let c = ramp_sample(lut, s);
                    color = [c[0], c[1], c[2]];
                    match ctx.alpha_mode {
                        RampAlphaMode::None => {}             // recolour only — alpha ignored
                        RampAlphaMode::Strength => g *= c[3], // less coverage where translucent
                        RampAlphaMode::TextureAlpha => stamp_alpha = c[3],
                    }
                } else {
                    // GRAIN Depth (Procreate) + watercolor Granulation gate — the single shared combine
                    // ([`crate::texture::grain_coverage`]); byte-identical at Depth 1 / Granulation 0.
                    g *= crate::texture::grain_coverage(
                        s,
                        ctx.spec.grain_depth(),
                        ctx.spec.effective_granulation(),
                    );
                }
                g *= crate::texture::stencil_gate(&ctx.spec.texture, b, px, py); // rect mask, ramp-safe
                if w * g <= 0.0 {
                    continue;
                }
            } else if let Some(lut) = ctx.ramp {
                // No Grain + a Colour Ramp on (Shape's ramp): silhouette `w` indexes it for the COLOUR; alpha per `alpha_mode` (a Strength `g → 0` is caught downstream).
                let c = ramp_sample(lut, w);
                color = [c[0], c[1], c[2]];
                match ctx.alpha_mode {
                    RampAlphaMode::None => {}
                    RampAlphaMode::Strength => g *= c[3],
                    RampAlphaMode::TextureAlpha => stamp_alpha = c[3],
                }
            }
            let i = row + (px as usize) * 4;
            let prev = [
                f32::from(dst[i]) / 255.0,
                f32::from(dst[i + 1]) / 255.0,
                f32::from(dst[i + 2]) / 255.0,
                f32::from(dst[i + 3]) / 255.0,
            ];
            // Mode TextureAlpha stamps the ramp as an RGBA image so translucent areas LOWER the
            // sprite's own alpha (punch it transparent) — alpha-lock can't apply (it edits alpha).
            // The other modes gate coverage by the dest alpha when alpha-locked + blend with the brush
            // mode (colour blend modes keep the dest alpha; you only paint where there's coverage).
            let out = if matches!(ctx.alpha_mode, RampAlphaMode::TextureAlpha) && ctx.ramp.is_some()
            {
                let m = w * g * ctx.coverage;
                if m <= 0.0 {
                    continue;
                }
                stamp_rgba(prev, color, stamp_alpha, m)
            } else {
                let a = match mask.as_deref_mut() {
                    // Accumulate OFF: cap each pixel at the TEXTURE-WEIGHTED target `coverage × g`, so a
                    // grain texel tops at `g·Strength` however many dabs cross it (`w` builds toward the
                    // cap). `g = 1` (no texture) ⇒ byte-identical to the old flat cap (Enio 2026-06-27).
                    //
                    // Under the film's screen-space AA (BUGS #16) `w` at a rim texel is its fractional
                    // AREA, and the plain build-up would converge it right back to a hard edge (measured:
                    // 0.64 → 0.94 over a stroke's overlapping dabs). The AA branch caps the texel at the
                    // film's AREA (`w·coverage`) while the per-dab opacity still builds WITHIN it — and
                    // at `cap = 1` (the whole interior at full pressure) `add = a_dab·(1 − m)` and
                    // `a = add/(1 − m) = a_dab`, the EXACT maskless alpha sequence: the interior is
                    // byte-identical, mask or no mask. (The first arming attempt jumped every texel to
                    // `w·g·cov` — which silently enforced the Grain cap on full-strength strokes and
                    // shifted every grain-textured interior; caught by the wax/shine material gates.)
                    // Without AA the old premise holds: the hard film's rim `w` is exactly zero, so the
                    // cut survives the build model for free.
                    Some(m_buf) => {
                        let mi = r * (ctx.stride / 4) + px as usize;
                        let m = f32::from(m_buf[mi]) / 255.0;
                        let add = if ctx.film_aa.is_some() {
                            let cap = (w * ctx.coverage).min(1.0);
                            if m >= cap {
                                continue; // the film's area is fully laid here
                            }
                            (w * g * ctx.coverage) * (1.0 - m / cap.max(1e-4))
                        } else {
                            let cap = (g * ctx.coverage).min(1.0);
                            if m >= cap {
                                continue; // already at this texel's weighted cap
                            }
                            w * (cap - m)
                        };
                        m_buf[mi] = ((m + add) * 255.0 + 0.5) as u8;
                        let a = add / (1.0 - m).max(1e-4);
                        if ctx.preserve_alpha { a * prev[3] } else { a }
                    }
                    None if ctx.preserve_alpha => w * g * ctx.coverage * prev[3],
                    None => w * g * ctx.coverage,
                };
                if a <= 0.0 {
                    continue;
                }
                crate::blend::blend_over_pigment(
                    blend,
                    prev,
                    color,
                    a,
                    ctx.spec.effective_pigment_mix(),
                )
            };
            dst[i] = encode(out[0]);
            dst[i + 1] = encode(out[1]);
            dst[i + 2] = encode(out[2]);
            dst[i + 3] = encode(out[3]);
            touched = true;
        }
    }
    touched
}

#[inline]
pub(crate) fn encode(v: f32) -> u8 {
    // Round-to-nearest, clamped. (No gamma here — the buffer is already in its native space.)
    (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
}

/// Stamp source `(color, sa)` (straight RGBA) onto destination `dst` within coverage `m ∈ [0,1]`:
/// a premultiplied lerp `out = dst·(1−m) + (color,sa)·m`. Unlike a colour blend this can LOWER the
/// destination alpha (where `sa < dst.a` and `m` is high) — that's how mode [`RampAlphaMode::TextureAlpha`]
/// makes parts of the sprite transparent. With `sa = 1` everywhere it reduces to ordinary opaque paint.
pub(crate) fn stamp_rgba(dst: [f32; 4], color: [f32; 3], sa: f32, m: f32) -> [f32; 4] {
    let sa = sa.clamp(0.0, 1.0);
    let m = m.clamp(0.0, 1.0);
    let da = dst[3];
    let out_a = da * (1.0 - m) + sa * m;
    if out_a <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    // Un-premultiply: each channel is the premultiplied lerp divided by the out alpha.
    let mix = |b: f32, s: f32| (b * da * (1.0 - m) + s * sa * m) / out_a;
    [
        mix(dst[0], color[0]),
        mix(dst[1], color[1]),
        mix(dst[2], color[2]),
        out_a,
    ]
}

/// Sample a baked Color Ramp LUT at `s ∈ [0, 1]` (nearest entry — the 256-step LUT is already fine).
pub(crate) fn ramp_sample(lut: &[[f32; 4]], s: f32) -> [f32; 4] {
    if lut.is_empty() {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let n = lut.len();
    let idx = (s.clamp(0.0, 1.0) * (n as f32 - 1.0) + 0.5) as usize;
    lut[idx.min(n - 1)]
}

#[cfg(test)]
mod tests;
