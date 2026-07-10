//! Watercolor **field math** — the deterministic building blocks of the optical composite
//! ([`super::watercolor_render`]): the integer-hash value noise (warp + built-in paper tooth), the
//! smoothstep / bilinear samplers and the O(n) separable box blur, plus the per-stroke [`WetStrokeStyle`]
//! + session tables. The optical `s2l`/`ln`/`exp` LUTs live in [`super::watercolor_lut`] (re-exported
//! below so callers keep using `watercolor_field::{Luts, luts}`). Split from `watercolor_render.rs` for
//! the workspace LOC cap.

pub(super) use super::watercolor_lut::*;
use rayon::prelude::*;

// ── Deterministic value noise (integer hash; HR-5 transcendental-free) ───────────────────────────────
// Distinct seeds keep the two warp axes + the paper grain decorrelated (else the boundary would ripple
// along the diagonal and the granulation would track the warp).
pub(super) const SEED_WARP_X_A: u32 = 0x1111_1111;
pub(super) const SEED_WARP_X_B: u32 = 0x2222_2222;
pub(super) const SEED_WARP_Y_A: u32 = 0x3333_3333;
pub(super) const SEED_WARP_Y_B: u32 = 0x4444_4444;
pub(super) const SEED_GRAIN: u32 = 0x5555_5555;
pub(super) const SEED_GRAIN_FINE: u32 = 0x6666_6666;

/// Smoothstep-in-`[0,1]` fade (the value-noise interpolation weight; a cubic, so no transcendental).
#[inline]
pub(super) fn smooth01(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Integer hash of a lattice cell → `[0, 1)` (a `lowbias32`-style avalanche; deterministic + fast).
#[inline]
pub(super) fn hash2(ix: i32, iy: i32, seed: u32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x8DA6_B343)
        ^ (iy as u32).wrapping_mul(0xD816_3841)
        ^ seed.wrapping_mul(0x1B56_C4E9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Bilinear value noise with `cell`-px features at `(x, y)` → `[0, 1]` (wet_edges `valueNoise`).
#[inline]
pub(super) fn value_noise(x: f32, y: f32, cell: f32, seed: u32) -> f32 {
    let fx = x / cell;
    let fy = y / cell;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let sx = smooth01(fx - x0);
    let sy = smooth01(fy - y0);
    let a = hash2(ix, iy, seed);
    let b = hash2(ix + 1, iy, seed);
    let c = hash2(ix, iy + 1, seed);
    let d = hash2(ix + 1, iy + 1, seed);
    let top = a + (b - a) * sx;
    let bot = c + (d - c) * sx;
    top + (bot - top) * sy
}

/// Fractal displacement field (two octaves, cells 22/8 px) in `[-1, 1]` for a warp axis (wet_edges warp).
#[inline]
pub(super) fn warp_axis(x: f32, y: f32, sa: u32, sb: u32) -> f32 {
    (value_noise(x, y, 22.0, sa) * 0.65 + value_noise(x, y, 8.0, sb) * 0.35 - 0.5) * 2.0
}

/// Paper-tooth granulation height at `(x, y)` in `[0, 1]` — the built-in fallback when no Paper
/// slot is set. TWO octaves (5 px + 2.5 px, doc 12 GRAN-3): the audit measured the old single
/// octave as mid-band and uniform (good) but mono-scale full-range — the "digital" side of the
/// default look; the fine second octave breaks the single-frequency signature.
#[inline]
pub(super) fn paper_height(x: f32, y: f32) -> f32 {
    0.65 * value_noise(x, y, 5.0, SEED_GRAIN) + 0.35 * value_noise(x, y, 2.5, SEED_GRAIN_FINE)
}

/// Smoothstep from `e0` to `e1` evaluated at `x` (cubic; clamps outside the edges).
#[inline]
pub(super) fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    smooth01(((x - e0) / (e1 - e0)).clamp(0.0, 1.0))
}

/// Bilinear sample of a region-local scalar field (clamped to the field edges).
#[inline]
pub(super) fn sample_bilinear(src: &[f32], w: usize, h: usize, fx: f32, fy: f32) -> f32 {
    let fx = fx.clamp(0.0, (w - 1) as f32);
    let fy = fy.clamp(0.0, (h - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(w - 1);
    let y1 = (y0 + 1).min(h - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let a = src[y0 * w + x0];
    let b = src[y0 * w + x1];
    let c = src[y1 * w + x0];
    let d = src[y1 * w + x1];
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    top + (bot - top) * ty
}

/// Separable box blur, O(n) via prefix sums (window count clamped at the borders — no darkening
/// bias); deterministic. **Parallel (ADR-0109 exception):** each pass distributes over its
/// INDEPENDENT axis with the serial prefix origin ⇒ bit-identical at any thread count; the
/// vertical pass writes transposed (contiguous, no `unsafe`) then a row-parallel transpose.
pub(super) fn box_blur(src: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }
    // Horizontal pass (src → tmp, row-major): each output row depends only on its own source row, so the
    // per-row prefix from `x = 0` is exactly the serial `pref`.
    let mut tmp = vec![0.0f32; w * h];
    tmp.par_chunks_mut(w)
        .zip(src.par_chunks(w))
        .for_each(|(trow, srow)| {
            let mut pref = vec![0.0f32; w + 1];
            for x in 0..w {
                pref[x + 1] = pref[x] + srow[x];
            }
            for (x, t) in trow.iter_mut().enumerate() {
                let lo = x.saturating_sub(radius);
                let hi = (x + radius).min(w - 1);
                *t = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
            }
        });
    // Vertical pass (tmp column `x` → `out_t` row `x`): the per-column prefix from `y = 0` is exactly the
    // serial `prefc`. Writing the TRANSPOSED layout keeps each task's output contiguous (a safe chunk).
    let mut out_t = vec![0.0f32; w * h]; // out_t[x * h + y]
    out_t.par_chunks_mut(h).enumerate().for_each(|(x, ocol)| {
        let mut pref = vec![0.0f32; h + 1];
        for y in 0..h {
            pref[y + 1] = pref[y] + tmp[y * w + x];
        }
        for (y, o) in ocol.iter_mut().enumerate() {
            let lo = y.saturating_sub(radius);
            let hi = (y + radius).min(h - 1);
            *o = (pref[hi + 1] - pref[lo]) / (hi - lo + 1) as f32;
        }
    });
    // Transpose out_t (x*h + y) → out (y*w + x), row-parallel (each output row is contiguous).
    let mut out = vec![0.0f32; w * h];
    out.par_chunks_mut(w).enumerate().for_each(|(y, orow)| {
        for (x, o) in orow.iter_mut().enumerate() {
            *o = out_t[x * h + y];
        }
    });
    out
}

// ── Rewet composite fields (moved from `watercolor_render` for the file-LOC cap) ────────────────────

/// The rewet's fields, built once per composite: raw paint presence, the water soak (dwell) RAW
/// (contact lift) + HALO (blurred outward, widening the dissolve), and the presence +
/// presence-weighted-colour blurs at the plain (`near` = spread) and lingering (`far` = 2×)
/// scales, `[presence, r, g, b]` each; the halo lerps `near → far`.
///
/// **Downsampled at high Spread** (`ds > 1`) on a **globally aligned** grid (`lox0/loy0`), so a
/// sampled value is independent of the frame's dirty-rect window (gate
/// `…incremental_composite_matches_full`); `ds == 1` (Spread ≤ [`REWET_DS_SPREAD`]) is exact.
pub(super) struct RewetFields {
    pub(super) pres: Vec<f32>,
    pub(super) soak_raw: Vec<f32>,
    pub(super) soak_halo: Vec<f32>,
    /// EDGE-2: the CARRIED-water pool's halo (2× blur of `stroke_water`) — the backrun ring is
    /// `raw − halo` (a shell just inside the pool's serrated contour). Empty unless water poured.
    pub(super) water_halo: Vec<f32>,
    pub(super) near: [Vec<f32>; 4],
    /// `None` until the stroke actually poured dwell (`wet_soak_active`) — a no-dwell stroke pays
    /// exactly the plain 4-blur rewet cost (measured 1.16 → ~0.6 ms/frame @2048²).
    pub(super) far: Option<[Vec<f32>; 4]>,
    /// Low-res grid: downsample factor + dims + global block origin (for the coord mapping).
    pub(super) ds: usize,
    pub(super) lw: usize,
    pub(super) lh: usize,
    pub(super) lox0: usize,
    pub(super) loy0: usize,
}

impl RewetFields {
    /// Sample a low-res field at the window-local warped coord `(sx, sy)` (full-res), mapping through
    /// the global-aligned downsample grid. `ds == 1` ⇒ `(sx, sy)` verbatim (full-res, exact).
    #[inline]
    pub(super) fn samp(&self, field: &[f32], rx0: usize, ry0: usize, sx: f32, sy: f32) -> f32 {
        let lx = (rx0 as f32 + sx) / self.ds as f32 - self.lox0 as f32;
        let ly = (ry0 as f32 + sy) / self.ds as f32 - self.loy0 as f32;
        sample_bilinear(field, self.lw, self.lh, lx, ly)
    }
}

/// Spread (px) at/below which the rewet fields stay full-resolution (`ds = 1`, exact); above it
/// the grid downsamples (`ds = spread / this`, capped). Above every unit test's Spread.
pub(super) const REWET_DS_SPREAD: usize = 12;

// ── Rewet tuning constants (moved from `watercolor_render`'s fn body for the file-LOC cap) ─────────

/// Max fraction of the base's pigment the rewet lifts at `wet = 1` under full water (never a
/// full erase — dried pigment doesn't fully redissolve).
pub(super) const REWET_LIFT: f32 = 0.85;
/// How much a fully-soaked pixel (the brush LINGERED here) deepens the lift beyond
/// [`REWET_LIFT`] — capped by [`LIFT_MAX`] (still never a full erase).
pub(super) const SOAK_LIFT: f32 = 0.12;
/// Hard ceiling of the lift fraction, soak included.
pub(super) const LIFT_MAX: f32 = 0.95;
/// How much a fully-soaked pixel boosts the dissolve amount (the redissolved pigment reads
/// stronger where the water sat) — full soak doubles it.
pub(super) const SOAK_DISSOLVE: f32 = 1.0;
/// How much of the dissolved pigment re-enters the wash's optical density (the bloom's body).
pub(super) const REWET_POOL: f32 = 0.35;
/// How much a fully-wet wash thins its own interior fill (deepest where `inner` ≈ 1 — the
/// pigment migrated out to the receding front; the rim keeps full body).
pub(super) const WET_THIN: f32 = 0.35;
/// Reference Spread (px) at/below which the interior thinning matches the historical look;
/// above it the thinning scales UP (a wetter wash empties its centre MORE — the "spread
/// clears the centre" dynamic the artist wants back at high Spread, Enio 2026-07-07).
pub(super) const SPREAD_THIN_REF: f32 = 16.0;
/// Cap of the extra thinning multiplier above the reference Spread.
pub(super) const SPREAD_THIN_MAX: f32 = 2.5;
/// Edge-pool gain of a fully-wet wash (`wet = 1` doubles the receding-front pooling).
pub(super) const WET_EDGE_BOOST: f32 = 1.0;
/// How strongly the paper tooth modulates the wet edge (a wet bloom is ragged, not a clean
/// ring): ±75% of the pool at `wet = 1`.
pub(super) const WET_RAGGED: f32 = 0.75;

/// Build the [`RewetFields`] for one composite window (moved out of `apply_watercolor` for the
/// file-LOC cap — pure function of the frozen buffers + window; behaviour unchanged, see the field
/// docs on [`RewetFields`]).
#[allow(clippy::too_many_arguments)] // field-build seam: every input is a distinct buffer/flag
pub(super) fn build_rewet_fields(
    base: &[u8],
    ground: &[u8],
    wet_soak: &[u8],
    soaked: bool,
    stroke_water: &[u8],
    watered: bool,
    (fw, fh): (usize, usize),
    (rx0, ry0, rx1, ry1): (usize, usize, usize, usize),
    spread: usize,
) -> RewetFields {
    // Downsample factor: 1 (exact) up to `REWET_DS_SPREAD`, then `spread / REWET_DS_SPREAD`
    // capped at 4 (blur cost /ds²). Global-aligned low-res grid over the read window.
    let ds = (spread / REWET_DS_SPREAD).clamp(1, 4);
    let lox0 = rx0 / ds;
    let loy0 = ry0 / ds;
    let lw = (rx1).div_ceil(ds) - lox0;
    let lh = (ry1).div_ceil(ds) - loy0;
    let mut pres = vec![0.0f32; lw * lh];
    let mut wr = vec![0.0f32; lw * lh];
    let mut wg = vec![0.0f32; lw * lh];
    let mut wb = vec![0.0f32; lw * lh];
    let mut soak = vec![0.0f32; lw * lh];
    // EDGE-2: the carried-water pool, downsampled like the soak (blurred below into its halo).
    let mut water = vec![0.0f32; lw * lh];
    let half = ds / 2;
    // Field fill, PARALLEL over grid rows (ADR-0109 class: each cell is a pure function of the
    // frozen base/ground/soak at its own sampled pixel — no cross-cell reduction, disjoint row
    // slices per task ⇒ byte-identical to the serial loop). This build ran serial while the
    // composite below went wide, and at Bleed ≤ 12 it is FULL-res (`ds = 1`) — the frame
    // profiler pinned it as the Rewet-only FPS dip (Enio 2026-07-07).
    let soak_src = wet_soak;
    pres.par_chunks_mut(lw)
        .zip(wr.par_chunks_mut(lw))
        .zip(wg.par_chunks_mut(lw))
        .zip(wb.par_chunks_mut(lw))
        .zip(soak.par_chunks_mut(lw))
        .zip(water.par_chunks_mut(lw))
        .enumerate()
        .for_each(|(lj, (((((prow, rrow), grow), brow), srow), wrow))| {
            // Sample each low-res cell at its block CENTRE (ds=1 ⇒ every full-res pixel, exact).
            let gy = (((loy0 + lj) * ds) + half).min(fh - 1);
            for li in 0..lw {
                let gx = (((lox0 + li) * ds) + half).min(fw - 1);
                let bi = (gy * fw + gx) * 4;
                let ab = f32::from(base[bi + 3]) / 255.0;
                let (gr, gg, gb) = (
                    f32::from(ground[bi]),
                    f32::from(ground[bi + 1]),
                    f32::from(ground[bi + 2]),
                );
                // The base over the real ground, straight sRGB bytes (the paint as seen).
                let r = f32::from(base[bi]) * ab + gr * (1.0 - ab);
                let g = f32::from(base[bi + 1]) * ab + gg * (1.0 - ab);
                let b = f32::from(base[bi + 2]) * ab + gb * (1.0 - ab);
                // Presence = how far this pixel departs from the LOCAL ground — only the active
                // layer's own paint differs from it (an unpainted pixel composites to the ground
                // exactly), so the reference is per-pixel true, light pigments included. The old
                // global-cream reference read a white canvas as 0.8 presence everywhere (flooded
                // the pool, "matou o efeito dinâmico do spread"), and paint LIGHTER than cream
                // as none. Dead-zoned so anti-aliasing crumbs don't count as paint (wet_edges
                // `PAINT_LO`/`PAINT_HI`).
                let d = (gr - r).abs().max((gg - g).abs()).max((gb - b).abs());
                let p = smoothstep(14.0, 50.0, d); // LITERAL-PX-OK: wet_edges PAINT_LO/PAINT_HI
                prow[li] = p;
                rrow[li] = r * p; // presence-premultiplied: the blur averages PAINT colour only
                grow[li] = g * p;
                brow[li] = b * p;
                if soaked {
                    srow[li] = f32::from(soak_src[gy * fw + gx]) / 255.0;
                }
                if watered {
                    wrow[li] = f32::from(stroke_water[gy * fw + gx]) / 255.0;
                }
            }
        });
    // Blur radii in low-res units (the low-res blur of radius r/ds ≈ a full-res blur of r).
    let r1 = (spread / ds).max(1);
    let r2 = ((spread * 2) / ds).max(1);
    // Two blur scales: the plain water spread + the lingering (soaked) spread — the per-pixel
    // soak lerps the dissolve between them ("the longer the water sits, the farther it
    // dissolves"). Soak all-zero ⇒ the second scale is never sampled with weight > 0.
    let bpres = box_blur(&pres, lw, lh, r1);
    let br = box_blur(&wr, lw, lh, r1);
    let bg = box_blur(&wg, lw, lh, r1);
    let bb = box_blur(&wb, lw, lh, r1);
    // The far (2×) fields + the soak halo exist only once dwell was poured: the dwelling
    // water DIFFUSES outward (the halo pushes the widened dissolve BEYOND the nib's own
    // disc — a raw disc gated the far blur to exactly the pixels under the brush), while
    // the RAW soak drives the lift (contact: deepest right under the nib).
    let water_halo = if watered {
        box_blur(&water, lw, lh, r2)
    } else {
        Vec::new()
    };
    let (far, soak_halo) = if soaked {
        let far = [
            box_blur(&pres, lw, lh, r2),
            box_blur(&wr, lw, lh, r2),
            box_blur(&wg, lw, lh, r2),
            box_blur(&wb, lw, lh, r2),
        ];
        (Some(far), box_blur(&soak, lw, lh, r2))
    } else {
        (None, Vec::new())
    };
    RewetFields {
        pres,
        soak_raw: soak,
        soak_halo,
        water_halo,
        near: [bpres, br, bg, bb],
        far,
        ds,
        lw,
        lh,
        lox0,
        loy0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Blur of a constant field is the same constant (window-count clamp handles the borders).
    #[test]
    fn box_blur_preserves_constant() {
        let src = vec![0.5f32; 8 * 6];
        let out = box_blur(&src, 8, 6, 2);
        for v in out {
            assert!((v - 0.5).abs() < 1e-6, "constant field must blur to itself");
        }
    }

    /// A single spike spreads mass to neighbours (peak drops, a neighbour rises) — proves it blurs.
    #[test]
    fn box_blur_spreads_a_spike() {
        let (w, h) = (9, 9);
        let mut src = vec![0.0f32; w * h];
        src[4 * w + 4] = 1.0;
        let out = box_blur(&src, w, h, 1);
        assert!(out[4 * w + 4] < 1.0, "peak dropped");
        assert!(out[4 * w + 5] > 0.0, "mass reached the neighbour");
    }

    /// The Beer–Lambert LUT path is the exact optical model: at full transmittance (`od = 0`) the pigment
    /// is invisible (output = base); as `od` grows the output moves monotonically toward the pigment. This
    /// pins the `s2l`/`exp`/`l2s` LUT composition against the closed-form `base·T + pigment·(1−T)`.
    #[test]
    fn beer_lambert_lut_matches_closed_form() {
        let lut = luts();
        let base: u8 = 200;
        let pig: u8 = 40;
        // od = 0 → T = 1 → output is the base byte exactly.
        assert_eq!(
            lut.transmittance(pig, 0.0),
            1.0,
            "od=0 ⇒ full transmittance"
        );
        let lin0 = lut.s2l[base as usize] * 1.0 + lut.s2l[pig as usize] * 0.0;
        assert_eq!(lut.l2s_byte(lin0), base, "od=0 composite = base");
        // Growing od darkens toward the pigment (monotone, bounded by the pigment byte).
        let mut prev = base as i32;
        for &od in &[0.5f32, 1.0, 2.0, 4.0, 8.0] {
            let t = lut.transmittance(pig, od);
            let lin = lut.s2l[base as usize] * t + lut.s2l[pig as usize] * (1.0 - t);
            let outv = lut.l2s_byte(lin) as i32;
            assert!(
                outv <= prev,
                "od={od}: composite moves toward the (darker) pigment"
            );
            assert!(outv >= pig as i32 - 1, "never past the pigment");
            prev = outv;
        }
    }

    /// Value noise is deterministic, in `[0, 1]`, and varies across cells (not a constant field).
    #[test]
    fn value_noise_is_deterministic_and_bounded() {
        let a = value_noise(12.3, 45.6, 5.0, SEED_GRAIN);
        let b = value_noise(12.3, 45.6, 5.0, SEED_GRAIN);
        assert_eq!(a, b, "same input ⇒ same value (deterministic)");
        assert!((0.0..=1.0).contains(&a), "in range");
        let c = value_noise(112.3, 245.6, 5.0, SEED_GRAIN);
        assert!(
            (a - c).abs() > 1e-4,
            "distant cells differ (it actually varies)"
        );
    }
}

/// EDGE-1 **per-stroke style** (doc 13 topo, Enio 2026-07-09): the wash params captured at each
/// session stroke's pen-down. The union re-bake resolves them PER PIXEL via the owner map — an
/// already-painted wash keeps ITS Concentration (depth) / Body / Edge / water / granulation
/// instead of being re-styled by the current brush (the reported bug: any param change propagated
/// through the wet pools inside the composite's rectangular window). Values are stored
/// pre-clamped EXACTLY as the composite clamped its globals, so owner-resolved math is
/// bit-identical for a single-style session.
#[derive(Clone, Copy)]
pub(super) struct WetStrokeStyle {
    pub(super) fill: f32,
    pub(super) depth: f32,
    /// Per-owner **Opacity** (pigment body / hiding power): a baked light-pigment wash keeps ITS body
    /// when a later stroke changes Opacity, like every other wash param. `0` = pure transmittance.
    pub(super) opacity: f32,
    pub(super) edge_gain: f32,
    pub(super) wet: f32,
    pub(super) granulation: f32,
    pub(super) warp: f32,
    pub(super) pigment_mix: f32,
    pub(super) color: [u8; 3],
    /// Per-owner GEOMETRY (doc 13 "mudança no brush propaga"): thinning multiplier, feather-blur
    /// radius, Spread — a baked wash re-renders with ITS geometry, never the live brush's.
    pub(super) spread_thin: f32,
    pub(super) core_r: u16,
    pub(super) spread_px: u16,
    /// Per-owner SUBSTRATE (doc 14 #13, smoke 2026-07-10): the Paper slot + its Depth + the "Same as
    /// Paper" flag + the Grain slot. A baked wash keeps ITS paper/grain — changing the substrate for
    /// the next stroke must NOT re-texture the pool below (the "aplica a tudo" + rectangles bug). The
    /// IMAGES (loaded custom paper/grain) stay session-shared in v1; only the SETTINGS are per-owner,
    /// which covers the reported triggers (paper Kind · Same as Paper · Grain Amount).
    pub(super) paper: ph2d_painter_brush::TextureSettings,
    pub(super) paper_depth: f32,
    pub(super) granulation_use_paper: bool,
    pub(super) texture: ph2d_painter_brush::TextureSettings,
}

impl WetStrokeStyle {
    /// Capture the current brush's wash params — the composite's exact clamps, verbatim. `forced_wet`
    /// is the **Wet the layer** floor (#3): the captured Rewet is `max(brush Rewet, forced)`, so strokes
    /// made after the Wet button lift the existing paint even at brush Rewet `0` (`forced = 0` ⇒ verbatim).
    pub(super) fn capture(spec: &ph2d_painter_brush::BrushSpec, forced_wet: f32) -> Self {
        let spread_px = spec.edge_spread.round().clamp(0.0, 48.0) as usize;
        Self {
            fill: spec.fill.clamp(0.0, 1.0),
            depth: spec.depth.max(0.0),
            opacity: spec.opacity.clamp(0.0, 1.0),
            edge_gain: spec.edge_gain.max(0.0),
            wet: spec.wet_rewet.max(forced_wet).clamp(0.0, 1.0),
            granulation: spec.granulation.clamp(0.0, 1.0),
            warp: spec.warp.max(0.0),
            pigment_mix: spec.effective_pigment_mix(),
            color: [
                (spec.color[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (spec.color[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                (spec.color[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ],
            spread_thin: (1.0 + (spread_px as f32 - SPREAD_THIN_REF).max(0.0) / SPREAD_THIN_REF)
                .min(SPREAD_THIN_MAX),
            core_r: spread_px.min(((spec.radius_px * 0.5).round() as usize).max(1)) as u16,
            spread_px: spread_px as u16,
            paper: spec.paper,
            paper_depth: spec.paper_depth.clamp(0.0, 1.0),
            granulation_use_paper: spec.granulation_use_paper,
            texture: spec.texture,
        }
    }
}

/// The wet session's per-stroke style TABLE + per-pixel OWNER map (`0` = unowned → the current
/// brush's style; `k` = `table[k−1]`). Ownership is RECENCY — the last stroke to touch a pixel
/// styles it, matching the colour buffer's source-over. Cleared with the session.
#[derive(Default)]
pub(super) struct WetSessionStyles {
    pub(super) table: Vec<WetStrokeStyle>,
    pub(super) owner: Vec<u8>,
}

impl WetSessionStyles {
    /// Register the beginning stroke's style; the index saturates at 255 (a 256th stroke in one
    /// wet session shares the last slot — far beyond any real ~8.5 s session).
    pub(super) fn push_capture(&mut self, spec: &ph2d_painter_brush::BrushSpec, forced_wet: f32) {
        if self.table.len() < 255 {
            self.table.push(WetStrokeStyle::capture(spec, forced_wet));
        } else if let Some(last) = self.table.last_mut() {
            *last = WetStrokeStyle::capture(spec, forced_wet);
        }
    }

    /// The CURRENT stroke's owner byte for the coverage splat.
    pub(super) fn current_owner(&self) -> u8 {
        self.table.len().min(255) as u8
    }

    pub(super) fn clear(&mut self) {
        self.table.clear();
        self.owner = Vec::new();
    }
}

// ── Granulation settling (doc 12 GRAN-1 — Curtis §4.5 valley deposition, Tier-2) ────────────────────
// Drying model (Enio 2026-07-08, take 3): the BAKE settles FULLY (the wash "sets" on pen-up);
// the LIVE preview runs close to dry so the release is subtle. KNOBS: `GRAN_SETTLE_BASE` (1.0 =
// WYSIWYG), `WET`/`SOAK` add live water response (capped at the dry value).
/// LIVE settle floor — how close the wet preview sits to the dry (baked) settle with no water.
pub(super) const GRAN_SETTLE_BASE: f32 = 0.80;
/// How much Rewet (water) raises the live settle toward the dry value.
pub(super) const GRAN_SETTLE_WET: f32 = 0.12;
/// How much the per-pixel soak (dwell) raises the live settle toward the dry value.
pub(super) const GRAN_SETTLE_SOAK: f32 = 0.12;
/// Peak-side strength of the valley gate (`1 − k·h·γ`): peaks shed up to γ of their share into the
/// valleys; `< 1` keeps a floor so full granulation never zeroes the wash (the old symmetric form
/// clamped low-h texels to 0 → white speckle holes).
pub(super) const GRAN_GAMMA: f32 = 0.9;

/// GRAN-1 valley-gated granulation factor (Curtis §4.5, doc 12): pigment settles INTO the
/// valleys (peaks shed up to γ). The settle grows with water + dwell live and goes FULL when
/// `settled` — the pen-up bake OR a pixel whose OWNER is already committed (a baked wash keeps
/// its DRY settle in the session window, Enio 2026-07-09). Amount 0 ⇒ `1 + paper_component`.
#[inline]
pub(super) fn granulation_factor(
    gran_h: Option<f32>,
    paper_component: f32,
    granulation: f32,
    wet: f32,
    soak_v: f32,
    settled: bool,
) -> f32 {
    match gran_h {
        Some(h) => {
            let settle = if settled {
                1.0
            } else {
                (GRAN_SETTLE_BASE + GRAN_SETTLE_WET * wet + GRAN_SETTLE_SOAK * soak_v).min(1.0)
            };
            let k = (granulation * settle).clamp(0.0, 1.0);
            ((1.0 + paper_component) * (1.0 - k * h * GRAN_GAMMA)).max(0.0)
        }
        None => (1.0 + paper_component).max(0.0),
    }
}

// ── EDGE-2 backrun (doc 12 — Curtis §2.2: "water tends to push pigment along as it spreads,
// resulting in complex, branching shapes with severely darkened edges") ──────────────────────────
/// Serration cell of the water-pool contour (px) — the mid-cell integer-hash jitter that turns the
/// smooth soak disc into the cauliflower boundary (doc 12 prescribes ~8-16 px).
pub(super) const BACKRUN_JAG_CELL: f32 = 12.0;
/// Serration amplitude (px) — how far the water contour read is displaced per cell.
pub(super) const BACKRUN_JAG_PX: f32 = 3.0; // KNOB (Enio 2026-07-09): serrilhado mais fino, menos pixelação
/// Pool-ring gain: dissolved pigment deposited along the water's serrated contour (the severely
/// darkened backrun edge). Scales `bp × ring`; calibration knob.
pub(super) const BACKRUN_POOL: f32 = 1.6; // KNOB (Enio 2026-07-09): banda do bloom mais leve
/// Ring CONCENTRATION: how much the ring deepens the dissolved pigment's absorbance. Beer–Lambert
/// saturates AT the pigment colour — density alone can never render darker than the wash the
/// pigment came from, but a backrun edge is that paint CONCENTRATED (darker floor). Knob.
pub(super) const BACKRUN_CONC: f32 = 1.5; // KNOB: 1.2 clareou o anel aprovado além do guard — mantido

/// The paper-tooth height at a canvas pixel: the Paper slot's tiled/rotated sample, or the
/// built-in noise fallback — shared by the substrate pre-pass + its fallback (LOC split).
pub(super) fn paper_h_px(
    paper_active: bool,
    paper_tex: &ph2d_painter_brush::TextureSettings,
    paper_img: Option<&ph2d_painter_brush::texture::ImageMask>,
    paper_rot: [f32; 2],
    gx: usize,
    gy: usize,
) -> f32 {
    if paper_active {
        ph2d_painter_brush::texture::sample_tiled_rot(
            paper_tex, gx as i64, gy as i64, paper_img, paper_rot,
        )
    } else {
        paper_height(gx as f32, gy as f32)
    }
}

/// EDGE-4: how much per-pixel DWELL (soak) boosts the rim gain — the rim strengthens where the
/// water pooled/lingered and stays plain elsewhere (the rim tells the gesture's story). Knob.
pub(super) const EDGE_SOAK_BOOST: f32 = 0.5;
const SEED_JAG_X: u32 = 0x4A47_5801;
const SEED_JAG_Y: u32 = 0x4A47_5902;

/// Sample the WATER channel (the session soak) at a SERRATED coordinate: canvas-anchored value
/// noise (cell [`BACKRUN_JAG_CELL`]) displaces the read up to ±[`BACKRUN_JAG_PX`], so the pool's
/// contour — and the backrun ring derived from it — comes out jagged/organic instead of the soak
/// disc's smooth edge. Returns `(water 0..1, global x, global y)` of the displaced read (the same
/// coord must sample the halo, or the ring shell drifts off the serration). Deterministic (HR-5).
#[inline]
pub(super) fn water_at(soak: &[u8], fw: usize, fh: usize, gx: usize, gy: usize) -> (f32, f32, f32) {
    let jx = (value_noise(gx as f32, gy as f32, BACKRUN_JAG_CELL, SEED_JAG_X) * 2.0 - 1.0)
        * BACKRUN_JAG_PX;
    let jy = (value_noise(gx as f32, gy as f32, BACKRUN_JAG_CELL, SEED_JAG_Y) * 2.0 - 1.0)
        * BACKRUN_JAG_PX;
    let wx = (gx as f32 + jx).clamp(0.0, (fw - 1) as f32);
    let wy = (gy as f32 + jy).clamp(0.0, (fh - 1) as f32);
    // BILINEAR read of the u8 pool: a nearest read left 1-px stairs on the ring's inner edge,
    // which the exponential CONC step amplified into visible pixelation (Enio smoke 2026-07-09).
    let (x0, y0) = (wx.floor(), wy.floor());
    let (tx, ty) = (wx - x0, wy - y0);
    let (ix, iy) = (x0 as usize, y0 as usize);
    let (ix1, iy1) = ((ix + 1).min(fw - 1), (iy + 1).min(fh - 1));
    let s = |x: usize, y: usize| f32::from(soak[y * fw + x]) / 255.0;
    let top = s(ix, iy) + (s(ix1, iy) - s(ix, iy)) * tx;
    let bot = s(ix, iy1) + (s(ix1, iy1) - s(ix, iy1)) * tx;
    (top + (bot - top) * ty, wx, wy)
}
