//! The dab's **raster core** (child of [`super`], split for the workspace
//! file-LOC cap): the parallel band splitter, the shared per-band pixel kernel
//! ([`stamp_band`] over a [`DabCtx`]) and the byte encode/composite helpers.
//! The public stamp API and the silhouette/grain doors stay in the parent —
//! this file is how a dab becomes pixels, not what a dab is.

use super::*;
use crate::ramp_alpha::RampAlphaMode;
use crate::spec::BrushSpec;
use crate::texture::{ImageMask, TexDabBasis};

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
pub(super) struct DabCtx<'a> {
    pub(super) spec: &'a BrushSpec,
    pub(super) tex: Option<&'a TexDabBasis>,
    pub(super) image: Option<ImageMask<'a>>,
    /// The Shape slot's silhouette inputs (frame + optional image). `Some` ⇒ the silhouette is the
    /// Shape, replacing the falloff; `None` ⇒ the falloff is the silhouette. See [`ShapeInput`].
    pub(super) shape: Option<ShapeInput<'a>>,
    /// Baked Color Ramp LUT: a per-texel value (Grain pattern, else the silhouette coverage) indexes it for the paint colour. [`stamp_dab_ramped`].
    pub(super) ramp: Option<&'a [[f32; 4]]>,
    /// What the ramp colour's alpha does (only meaningful when `ramp` is `Some`). See [`RampAlphaMode`].
    pub(super) alpha_mode: RampAlphaMode,
    /// Dab flatten/rotate for the falloff `t` (Shape/Grain carry it in their bases); baked once per dab.
    pub(super) footprint: crate::footprint::FootprintDeform,
    pub(super) center: [f32; 2],
    pub(super) cx: f32,
    pub(super) cy: f32,
    pub(super) inv_radius: f32,
    pub(super) radius: f32,
    pub(super) coverage: f32,
    pub(super) preserve_alpha: bool,
    pub(super) x0: i64,
    pub(super) x1: i64,
    pub(super) stride: usize,
    /// Screen-space AA of the film silhouette (BUGS #16) — `None` = single-sample `film_of`,
    /// byte-identical. Hoisted per dab in [`stamp_dab_pixels`].
    pub(super) film_aa: Option<crate::height_film::FilmAa>,
}

/// Stamp the dab's pixels for the full-width row band `dst` whose first row is `band_y0` (so global
/// row `py` lives at local offset `(py - band_y0) * stride`). Returns whether any pixel was written.
/// Pure over disjoint bands — no shared mutable state — so bands run in parallel deterministically.
pub(super) fn stamp_band(
    ctx: &DabCtx,
    dst: &mut [u8],
    mut mask: Option<&mut [u8]>,
    band_y0: i64,
) -> bool {
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
