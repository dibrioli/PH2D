//! Wet/burnt **edge settle** — the deferred "dry-down" pass that gives a finished
//! wash stroke its watercolor edge darkening (or a dry-media granulated edge),
//! run once on stroke settle (pen-up).
//!
//! ## Why a deferred pass, and why it is NOT a contour filter
//!
//! Real watercolor edge darkening is **pigment transported to the receding water
//! boundary**: as the wet region dries, its perimeter loses water faster than its
//! interior, water (and the pigment it carries) flows outward, and the pigment is
//! stranded where the water finally leaves — a soft, irregular dark rim that tracks
//! the *wet-region boundary*, not the painted silhouette (Curtis, Anderson, Seims,
//! Fleischer, Salesin, "Computer-Generated Watercolor", SIGGRAPH 1997, §3;
//! patent US6198489B1 `FlowOutward: p ← p − η·(1 − blur(M))·M`).
//!
//! The two earlier attempts in this engine darkened a fixed-width band on the
//! **stroke silhouette** — the wrong boundary, constant width, no transport — which
//! reads as a hard, low-resolution ink outline (the exact failure the brief calls
//! out). The correct operator (this module) drives darkening from the **inner-edge
//! shoulder of the continuous coverage field**: `rim = clamp(COV − blur_K(COV))` is
//! the discrete `(1 − blur(M))·M` Curtis FlowOutward operator — soft, sub-pixel,
//! 0 in the interior and 0 outside. It is weighted by low-frequency FBM noise so the
//! rim varies along its length and never reads as a uniform outline (Adobe "Edge
//! effect" patent US7777745B2: distance × low-frequency Perlin noise).
//!
//! ## v1.5 — physically-grounded Kubelka–Munk dry-down (watercolor, `EdgeStyle::Wet`)
//!
//! The rim is no longer a gamma `c ← c^(1+k·rim)` hack. Instead it deposits extra
//! pigment **optical depth** at the receding boundary and recomputes the colour with
//! the **finite-thickness Kubelka–Munk** equation (Curtis §5.2; ScienceDirect K–M):
//! `R(t) = (1 − Rg(a − b·coth(b·t))) / (a − Rg + b·coth(b·t))`, `a = 1 + K/S`,
//! `b = √(a²−1)`, where `Rg` is the current pixel and `t→∞ → R∞` is the pigment
//! masstone. More deposited pigment ⇒ `R` drops toward the masstone — darker AND more
//! saturated, with the physically-correct hue shift (a gamma can't do the hue), and
//! bounded by the masstone (no black-crush). This is the lever Bousseau/Montesdeoca
//! call "pigment density / accumulation toward edges".
//!
//! On top of the rim, the same pass adds **granulation** (Curtis §4.5 `TransferPigment`
//! adsorption asymmetry, collapsed to the Montesdeoca single-pass form): pigment
//! sediments into the paper-tooth **valleys** and lifts off the **crests**, gated by
//! how much pigment is present. It is *mass-conserving* — valleys deposit toward the
//! masstone, crests lift toward the substrate backdrop — so a flat wash gains a
//! mottled granular texture with no net darkening. Granulating vs staining pigments
//! is the `granulation` strength (heavy mineral pigments settle; fine organics stain
//! and don't — Natural Pigments, Linda Saul RWS).
//!
//! When the masstone (`wash_color`) is absent — the build-up path has no wash-colour
//! buffer — the rim falls back to the original gamma operator. `EdgeStyle::Burnt`
//! (dry media: charcoal / sumi-e) is not watercolour pigment-in-water, so it keeps
//! the bidirectional paper-tooth gamma speckle.
//!
//! This is the productized two-phase lifecycle (DiVerdi et al., "Painting with
//! Polygons", I3D 2012; Procreate / Adobe Fresco re-render on pen-up): the live
//! stroke shows honest wet paint (interior only, no rim), and the rim + granulation
//! "settle" in on stroke-end from the now-known wet boundary + paper surface.

use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};

/// Reflectance floor/ceiling for the Kubelka–Munk maps (a 0 or 1 band sends `K/S`
/// to 0 or ∞).
const REFL_FLOOR: f32 = 1.0e-4;
/// Optical-depth gain mapping `strength·rim·noise` → the rim's deposited pigment
/// depth `t`. `strength 0.6` on a sharp edge (`rim ≈ 0.5`, `noise ≈ 1`) reaches
/// `t ≈ 0.6·0.5·1·3 = 0.9` → a strong, watercolor-grade concentration toward the
/// masstone (research: `t ≈ 0.2..1` already reads as a deep edge).
const WET_EDGE_T_GAIN: f32 = 3.0;
/// Granulation optical-depth amplitude: at full `granulation`, a deep valley with
/// full coverage deposits up to this much extra depth toward the masstone (and the
/// matching crest lifts the same toward the backdrop).
const GRAN_T_AMP: f32 = 1.4;
/// Hard cap on any single deposit's optical depth so a pathological field can't
/// drive `coth` to the saturated tail in one step.
const WET_T_MAX: f32 = 6.0;
/// Outward-bleed band radius as a multiple of `rim_px` — the water front carries
/// pigment this much further than the inner rim before it dries.
const BLEED_RADIUS_MULT: f32 = 2.6;
/// `smoothstep` band on the warped front that selects the bleed fringe: the warped
/// blurred-coverage value runs ~0.5 at the silhouette down to 0 far out; `[LO, HI]`
/// places the fringe just OUTSIDE the edge and fades it smoothly to nothing (no hard
/// outer ring).
const BLEED_FRONT_LO: f32 = 0.04;
const BLEED_FRONT_HI: f32 = 0.34;
/// Max optical depth of the bleed fringe — a THIN glaze (Curtis: the fringe is a
/// faint translucent wash, far less pigment than the body).
const BLEED_T_MAX: f32 = 0.55;
/// Fringe alpha cap so the bloom onto a transparent layer stays translucent.
const BLEED_ALPHA_MAX: f32 = 0.8;

/// `smoothstep(lo, hi, x)` — the Hermite S-curve, 0 below `lo`, 1 above `hi`, with
/// zero slope at both ends (so a fringe gated by it fades to *exactly* 0 — a
/// Gaussian/exp would leave a faint ring).
#[inline]
fn smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = ((x - lo) / (hi - lo).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 3-octave value-noise fBM in `[0,1]` over the deterministic simplex grain field —
/// the domain-warp source for the bleed tendrils (Inigo Quilez fBM/warp). Pure
/// arithmetic + fixed seed → HR-5 deterministic.
#[inline]
fn fbm2(x: f32, y: f32, seed: u32) -> f32 {
    let mut f = 1.0f32;
    let mut amp = 0.5f32;
    let mut sum = 0.0f32;
    let mut norm = 0.0f32;
    for o in 0..3u32 {
        sum += amp
            * crate::grain_noise::grain_value(
                crate::grain_noise::GRAIN_SIMPLEX,
                x * f,
                y * f,
                seed ^ (o.wrapping_mul(0x9e37_79b9)),
            );
        norm += amp;
        f *= 2.0;
        amp *= 0.5;
    }
    sum / norm.max(1e-6)
}

/// Finite-thickness **Kubelka–Munk** reflectance of a pigment whose masstone (R∞)
/// is `masstone`, deposited at optical depth `t ≥ 0` over a backdrop reflectance
/// `rg`. `t = 0 → rg` (no-op); `t → ∞ → masstone`. Per channel, linear light.
///
/// `a = 1 + K/S`, `b = √(a²−1)` with `K/S = (1−R∞)²/(2R∞)`; `coth` uses the
/// series near 0 (avoids `0/0`) and saturates to 1 in the tail. A white/clear
/// pigment (`b ≈ 0`, no absorption) is a no-op. Replaces the gamma `c^e` rim with
/// a physically-grounded "more pigment = darker + more saturated, bounded by the
/// masstone" deposit (ScienceDirect Kubelka–Munk; Curtis SIGGRAPH 1997 §5.2).
#[inline]
fn km_deposit(masstone: f32, rg: f32, t: f32) -> f32 {
    if t <= 0.0 {
        return rg;
    }
    let r_inf = masstone.clamp(REFL_FLOOR, 1.0 - REFL_FLOOR);
    let ks = (1.0 - r_inf) * (1.0 - r_inf) / (2.0 * r_inf);
    let a = 1.0 + ks;
    let b = (a * a - 1.0).sqrt();
    if b <= 1.0e-6 {
        return rg; // no absorption (white/clear) — nothing to deposit
    }
    let tau = (b * t).min(40.0);
    let coth = if tau < 1.0e-2 {
        1.0 / tau + tau / 3.0
    } else {
        tau.cosh() / tau.sinh()
    };
    let rg = rg.clamp(REFL_FLOOR, 1.0 - REFL_FLOOR);
    let r = (1.0 - rg * (a - b * coth)) / (a - rg + b * coth);
    r.clamp(REFL_FLOOR, 1.0 - REFL_FLOOR)
}

/// **Outward bleed / bloom** — the water front carrying pigment PAST the painted
/// silhouette (Curtis SIGGRAPH 1997 capillary layer: the wet region grows beyond
/// where pigment was laid). A single deferred pass: blur the coverage to a soft
/// "front" extending `radius_px` past the edge, **domain-warp** the sample point
/// with fBM so the front breaks into irregular fingers/tendrils (Saffman–Taylor
/// fingering; Inigo Quilez warp) instead of a uniform halo, then deposit a THIN
/// Kubelka–Munk glaze of the stroke's mean pigment over the backdrop, tapering to
/// zero (no hard outer ring) and settling into the paper valleys.
///
/// `mean_pigment` is the stroke's representative pigment (linear, averaged over the
/// solid body). Only pixels OUTSIDE the silhouette (`coverage ≈ 0`) are touched —
/// the body + rim are owned by [`apply_wash_settle`]'s main loop. Deterministic
/// (fBM + fixed `seed`).
#[allow(clippy::too_many_arguments)]
fn outward_bleed(
    canvas: &mut [u8],
    backdrop: &[u8],
    coverage: &[f32],
    mean_pigment: [f32; 3],
    width: u32,
    height: u32,
    region: (u32, u32, u32, u32),
    radius_px: u32,
    strength: f32,
    granulation: f32,
    seed: u32,
) {
    let w = width as i32;
    let h = height as i32;
    let r = radius_px.max(2) as i32;
    // Expanded region = covered bbox grown by the bleed radius (clamped to canvas).
    let ex0 = (region.0 as i32 - r).max(0);
    let ey0 = (region.1 as i32 - r).max(0);
    let ex1 = ((region.0 + region.2) as i32 + r).min(w);
    let ey1 = ((region.1 + region.3) as i32 + r).min(h);
    let ew = (ex1 - ex0) as usize;
    let eh = (ey1 - ey0) as usize;
    if ew == 0 || eh == 0 {
        return;
    }
    // Materialize the blurred coverage "front" over the expanded region (σ ≈ r/2),
    // so the warped lookups below have random access (Curtis wet-mask proxy).
    let k = (r / 2).max(1);
    let win = (2 * k + 1) as f32;
    let ty0 = (ey0 - k).max(0);
    let ty1 = (ey1 + k).min(h);
    let th = (ty1 - ty0) as usize;
    let mut tmp = vec![0.0f32; th * ew];
    for ly in 0..th {
        let y = ty0 + ly as i32;
        let row = y as usize * width as usize;
        for lx in 0..ew {
            let x = ex0 + lx as i32;
            let mut s = 0.0;
            for dx in -k..=k {
                s += coverage[row + (x + dx).clamp(0, w - 1) as usize];
            }
            tmp[ly * ew + lx] = s / win;
        }
    }
    let mut blurred = vec![0.0f32; eh * ew];
    for ly in 0..eh {
        let y = ey0 + ly as i32;
        for lx in 0..ew {
            let mut s = 0.0;
            for dy in -k..=k {
                let ty = (y + dy).clamp(0, h - 1);
                s += tmp[(ty - ty0) as usize * ew + lx];
            }
            blurred[ly * ew + lx] = s / win;
        }
    }
    let sample = |fx: f32, fy: f32| -> f32 {
        let xi = (fx.round() as i32 - ex0).clamp(0, ew as i32 - 1) as usize;
        let yi = (fy.round() as i32 - ey0).clamp(0, eh as i32 - 1) as usize;
        blurred[yi * ew + xi]
    };
    // Lower-frequency noise (longer fingers, not fine speckle) + a warp amplitude
    // close to the band width, so the front breaks into clear irregular tendrils
    // rather than a uniform halo (the noise-warped bleed the brief asks for).
    let ns = 0.8 / (r as f32).max(1.0);
    let warp_amp = 0.85 * r as f32;
    for ly in 0..eh {
        let y = ey0 + ly as i32;
        for lx in 0..ew {
            let x = ex0 + lx as i32;
            let pix = y as usize * width as usize + x as usize;
            if coverage[pix] > 0.02 {
                continue; // body + rim are the main loop's job
            }
            let (xf, yf) = (x as f32, y as f32);
            // Domain-warp the front into fingers (IQ warp: offset by an fBM vector).
            let qx = fbm2(xf * ns, yf * ns, seed);
            let qy = fbm2(xf * ns + 5.2, yf * ns + 1.3, seed);
            let front = sample(
                xf + warp_amp * (2.0 * qx - 1.0),
                yf + warp_amp * (2.0 * qy - 1.0),
            );
            let wet = smoothstep(BLEED_FRONT_LO, BLEED_FRONT_HI, front);
            if wet <= 1.0e-3 {
                continue;
            }
            // Pigment settles into the paper valleys (Curtis granulation): bias the
            // deposit by `(1 − paper_h)`, scaled by the granulation amount.
            let paper_h = super::paper_tooth_height(xf + 0.5, yf + 0.5);
            let tooth = (1.0 - granulation * paper_h).max(0.0);
            let t = (strength * wet * BLEED_T_MAX * tooth).min(BLEED_T_MAX);
            if t <= 1.0e-4 {
                continue;
            }
            // Composite a thin pigment layer (colour = mean_pigment, alpha = fa) OVER
            // the backdrop — correct for both opaque paper (tints toward pigment) and
            // a transparent layer (adds faint coloured alpha = the bloom is new paint).
            let idx = pix * 4;
            let fa = (1.0 - (-t).exp()).min(BLEED_ALPHA_MAX);
            let back_a = backdrop[idx + 3] as f32 / 255.0;
            let out_a = fa + back_a * (1.0 - fa);
            if out_a <= 1.0e-4 {
                continue;
            }
            for ch in 0..3 {
                let back = srgb_to_linear_byte(backdrop[idx + ch]) * back_a;
                let src = mean_pigment[ch].clamp(0.0, 1.0) * fa;
                let outc = ((src + back * (1.0 - fa)) / out_a).clamp(0.0, 1.0);
                canvas[idx + ch] = linear_to_srgb_byte(outc);
            }
            canvas[idx + 3] = (out_a * 255.0 + 0.5) as u8;
        }
    }
}

/// The stroke's representative pigment (linear, averaged over the solid body) for
/// the [`outward_bleed`] fringe. `None` if the stroke has no solid body. Uses
/// `coverage > 0.5` so the average is the masstone, not the soft AA fringe.
fn mean_body_pigment(
    coverage: &[f32],
    wash_color: &[[f32; 3]],
    width: u32,
    region: (u32, u32, u32, u32),
) -> Option<[f32; 3]> {
    let (mut sum, mut cnt) = ([0.0f64; 3], 0u64);
    for y in region.1..region.1 + region.3 {
        let row = y as usize * width as usize;
        for x in region.0..region.0 + region.2 {
            let pix = row + x as usize;
            if coverage[pix] > 0.5 {
                let c = wash_color[pix];
                sum[0] += c[0] as f64;
                sum[1] += c[1] as f64;
                sum[2] += c[2] as f64;
                cnt += 1;
            }
        }
    }
    (cnt > 0).then(|| {
        let n = cnt as f64;
        [
            (sum[0] / n) as f32,
            (sum[1] / n) as f32,
            (sum[2] / n) as f32,
        ]
    })
}

/// Spatial frequency of the rim-breaking FBM noise (cycles per canvas pixel). Low
/// enough (~20px wavelength) that the rim intensity *undulates* along its length
/// rather than turning into high-frequency speckle.
const WET_NOISE_FREQ: f32 = 0.05;
/// Floor of the noise modulation so the rim never fully vanishes (`[floor, 1]`).
const WET_NOISE_FLOOR: f32 = 0.35;
/// Maps `strength·rim·noise` to the darkening exponent boost. `strength = 0.6` on a
/// sharp edge (`rim ≈ 0.5`) reaches `e ≈ 1 + 0.6·0.5·1·3 = 1.9` → a `0.5` value
/// drops to `0.5^1.9 ≈ 0.27`: a strong, watercolor-grade dark rim.
const WET_EDGE_GAIN: f32 = 3.0;
/// Hard cap on the exponent boost so a pathological coverage field can't crush the
/// rim to black.
const WET_EDGE_BOOST_MAX: f32 = 2.5;

/// Burnt-edge (dry-media) grain frequency — a coarse, clearly-visible `tooth`
/// (~3px wavelength) so the edge reads as granular charcoal/sumi-e, not film grain.
const BURNT_NOISE_FREQ: f32 = 0.3;
/// Burnt edges hit harder than a watercolor rim (charcoal is high-contrast).
const BURNT_EDGE_GAIN: f32 = 3.2;
/// Burnt band width multiplier on `rim_px` — dry media granulates a BROAD edge
/// zone (vs the thin watercolor rim), so the speckle covers a wider fringe.
const BURNT_BAND_MULT: i32 = 3;
/// How far burnt edges may LIGHTEN at a tooth valley (`e = 1 − this` floor → the
/// paper showing through the grain). Bidirectional darken/lighten is what makes a
/// dry edge read as granular rather than a smooth dark rim.
const BURNT_LIGHTEN_MAX: f32 = 0.62;

/// Which media the edge settle emulates — watercolor wet rim vs dry-media
/// (charcoal / sumi-e) granulated edge. Both are the SAME inner-edge transport
/// band; they differ only in how the rim is broken up (smooth low-freq undulation
/// vs high-freq paper-tooth speckle) and how hard it darkens.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EdgeStyle {
    /// Watercolor: a soft dark rim deposited at the receding water boundary.
    Wet,
    /// Dry media (charcoal / sumi-e): a granular dark edge from the paper tooth.
    Burnt,
}

/// Apply the watercolor **dry-down** pass to a finished wash stroke in-place:
/// Kubelka–Munk edge darkening at the receding water boundary + mass-conserving
/// granulation into the paper tooth (`EdgeStyle::Wet` with `wash_color`), or the
/// dry-media gamma speckle (`EdgeStyle::Burnt` / build-up fallback).
///
/// `coverage` is the stroke's per-pixel pigment coverage ∈[0,1]. `wash_color` is
/// the per-pixel deposited pigment **masstone** (linear, the wash path's
/// `wash_color` buffer) — `Some` enables the physically-grounded K–M path; `None`
/// (build-up, no colour buffer) falls back to the gamma rim. `backdrop` is the
/// pre-stroke canvas (straight sRGB8) the crest-lift granulation removes pigment
/// toward; `None` ⇒ granulation only deposits into valleys. `region` = `(x,y,w,h)`
/// is the dirty bbox (work is limited to it). `strength` ∈[0,1] is the rim
/// `edge_intensity`; `granulation` ∈[0,1] the sediment amount; `rim_px` the blur
/// radius (≈ rim width); `noise_seed` the stroke seed (HR-5 deterministic +
/// replay-stable).
///
/// Canvas is straight sRGB8; the K–M maps + gamma run in linear light (decode →
/// map → encode), matching the render's gamma discipline.
#[allow(clippy::too_many_arguments)]
pub fn apply_wash_settle(
    canvas: &mut [u8],
    backdrop: Option<&[u8]>,
    coverage: &[f32],
    wash_color: Option<&[[f32; 3]]>,
    width: u32,
    height: u32,
    region: (u32, u32, u32, u32),
    strength: f32,
    rim_px: u32,
    granulation: f32,
    noise_seed: u32,
    style: EdgeStyle,
) {
    let (noise_freq, gain) = match style {
        EdgeStyle::Wet => (WET_NOISE_FREQ, WET_EDGE_GAIN),
        EdgeStyle::Burnt => (BURNT_NOISE_FREQ, BURNT_EDGE_GAIN),
    };
    let n = (width as usize) * (height as usize);
    assert_eq!(canvas.len(), n * 4, "canvas size must match width*height*4");
    assert_eq!(coverage.len(), n, "coverage size must match width*height");
    if let Some(wc) = wash_color {
        assert_eq!(wc.len(), n, "wash_color size must match width*height");
    }
    if let Some(bk) = backdrop {
        assert_eq!(bk.len(), n * 4, "backdrop size must match width*height*4");
    }
    let strength = strength.clamp(0.0, 1.0);
    let granulation = granulation.clamp(0.0, 1.0);
    // Physically-grounded K–M watercolor path: only when we have the pigment
    // masstone (the wash path's colour buffer). Build-up + burnt use the gamma rim.
    let km = matches!(style, EdgeStyle::Wet) && wash_color.is_some();
    let do_gran = km && granulation > 0.0;
    // Nothing to do: no rim strength AND no granulation.
    if strength <= 0.0 && !do_gran {
        return;
    }
    let w = width as i32;
    let h = height as i32;
    // Clamp the region to the canvas.
    let rx = region.0.min(width) as i32;
    let ry = region.1.min(height) as i32;
    let rw = (region.0 + region.2).min(width) as i32 - rx;
    let rh = (region.1 + region.3).min(height) as i32 - ry;
    if rw <= 0 || rh <= 0 {
        return;
    }
    // Burnt granulates a broad edge zone; wet leaves a thin rim.
    let band_mult = match style {
        EdgeStyle::Wet => 1,
        EdgeStyle::Burnt => BURNT_BAND_MULT,
    };
    let k = (rim_px.max(1) as i32 * band_mult).min(64);
    let win = (2 * k + 1) as f32;
    let rw_us = rw as usize;

    // Separable box-blur of `coverage` over the region. Horizontal pass first,
    // into `tmp` covering rows [ry-k, ry+rh+k) (clamped) so the vertical pass has
    // its full ±k window; the canvas-edge clamp keeps reads in bounds.
    let ty0 = (ry - k).max(0);
    let ty1 = (ry + rh + k).min(h);
    let th = (ty1 - ty0) as usize;
    let mut tmp = vec![0.0f32; th * rw_us];
    for ly in 0..th {
        let y = ty0 + ly as i32;
        let row = y as usize * width as usize;
        for lx in 0..rw_us {
            let x = rx + lx as i32;
            let mut sum = 0.0;
            for dx in -k..=k {
                let sx = (x + dx).clamp(0, w - 1) as usize;
                sum += coverage[row + sx];
            }
            tmp[ly * rw_us + lx] = sum / win;
        }
    }

    // Vertical pass + the rim darkening, over the dirty region proper.
    for ly in 0..rh as usize {
        let y = ry + ly as i32;
        let row = y as usize * width as usize;
        for lx in 0..rw_us {
            let mut sum = 0.0;
            for dy in -k..=k {
                let ty = (y + dy).clamp(0, h - 1);
                let tly = (ty - ty0) as usize;
                sum += tmp[tly * rw_us + lx];
            }
            let blurred = sum / win;
            let x = rx + lx as i32;
            let pix = row + x as usize;
            let idx = pix * 4;
            let cov = coverage[pix];
            // Inner-edge shoulder = the receding water boundary (Curtis FlowOutward).
            let rim = (cov - blurred).clamp(0.0, 1.0);
            let (xf, yf) = (x as f32, y as f32);

            if km {
                // ── Physically-grounded Kubelka–Munk dry-down (watercolor) ──
                let wash_color = wash_color.unwrap();
                // Granulation: mass-conserving sediment into the paper tooth — deposit
                // toward the masstone in valleys, lift toward the backdrop on crests
                // (Curtis §4.5 adsorption asymmetry, single-pass). Gated by how much
                // pigment is present (`cov`).
                let mut t_mass = 0.0f32;
                let mut t_back = 0.0f32;
                if do_gran && cov > 1.0 / 255.0 {
                    let paper_h = super::paper_tooth_height(xf + 0.5, yf + 0.5);
                    let signed = 1.0 - 2.0 * paper_h; // + = valley, − = crest
                    let mag = granulation * GRAN_T_AMP * cov * signed.abs();
                    if signed >= 0.0 {
                        t_mass += mag;
                    } else {
                        t_back += mag;
                    }
                }
                // Rim edge darkening: extra pigment depth toward the masstone at the
                // receding boundary, undulated by low-freq noise so it never reads as
                // a uniform outline.
                if strength > 0.0 && rim > 1.0 / 255.0 {
                    let noise = crate::grain_noise::grain_value(
                        crate::grain_noise::GRAIN_SIMPLEX,
                        xf * WET_NOISE_FREQ,
                        yf * WET_NOISE_FREQ,
                        noise_seed,
                    );
                    let nmod = WET_NOISE_FLOOR + (1.0 - WET_NOISE_FLOOR) * noise;
                    t_mass += strength * rim * nmod * WET_EDGE_T_GAIN;
                }
                if t_mass <= 1.0e-4 && t_back <= 1.0e-4 {
                    continue;
                }
                let c = wash_color[pix];
                let t_mass = t_mass.min(WET_T_MAX);
                let t_back = t_back.min(WET_T_MAX);
                // Crest-lift REMOVES coloured pigment (revealing the substrate) — the
                // inverse of a deposit, so it is a saturating lerp toward the backdrop,
                // NOT a `km_deposit` (depositing white pigment is a no-op). Needs a
                // defined paper colour; skip where the substrate is (near-)transparent.
                let back_opaque = backdrop.is_some_and(|bk| bk[idx + 3] > 8);
                let lift = if t_back > 0.0 && back_opaque {
                    1.0 - (-t_back).exp() // ∈[0,1): more lifted depth → closer to paper
                } else {
                    0.0
                };
                for ch in 0..3 {
                    let mut rg = srgb_to_linear_byte(canvas[idx + ch]);
                    // Valleys: deposit pigment toward the masstone (darker, saturated).
                    if t_mass > 0.0 {
                        rg = km_deposit(c[ch].clamp(0.0, 1.0), rg, t_mass);
                    }
                    // Crests: lift pigment off toward the bare substrate (lighter).
                    if lift > 0.0 {
                        let paper = srgb_to_linear_byte(backdrop.unwrap()[idx + ch]);
                        rg += (paper - rg) * lift;
                    }
                    canvas[idx + ch] = linear_to_srgb_byte(rg);
                }
                // Alpha untouched — granulation/rim redistribute pigment, not coverage.
            } else {
                // ── Gamma fallback: build-up wet rim + dry-media (burnt) speckle ──
                if strength <= 0.0 || rim <= 1.0 / 255.0 {
                    continue;
                }
                let noise = crate::grain_noise::grain_value(
                    crate::grain_noise::GRAIN_SIMPLEX,
                    xf * noise_freq,
                    yf * noise_freq,
                    noise_seed,
                );
                // WET: smooth dark rim (`e ≥ 1`). BURNT: paper tooth — noise is
                // BIDIRECTIONAL, so tooth peaks darken and valleys lighten (the
                // two-sided speckle that reads as granular dry media, not a wet line).
                let e = match style {
                    EdgeStyle::Wet => {
                        let nmod = WET_NOISE_FLOOR + (1.0 - WET_NOISE_FLOOR) * noise;
                        1.0 + (strength * rim * nmod * gain).min(WET_EDGE_BOOST_MAX)
                    }
                    EdgeStyle::Burnt => {
                        let signed = 2.0 * noise - 1.0;
                        let delta = (strength * rim * signed * gain)
                            .clamp(-BURNT_LIGHTEN_MAX, WET_EDGE_BOOST_MAX);
                        1.0 + delta
                    }
                };
                let r = srgb_to_linear_byte(canvas[idx]).powf(e);
                let g = srgb_to_linear_byte(canvas[idx + 1]).powf(e);
                let b = srgb_to_linear_byte(canvas[idx + 2]).powf(e);
                canvas[idx] = linear_to_srgb_byte(r);
                canvas[idx + 1] = linear_to_srgb_byte(g);
                canvas[idx + 2] = linear_to_srgb_byte(b);
            }
        }
    }

    // ── Outward bleed (watercolor bloom past the silhouette) ──
    // A second deferred pass over the region GROWN by the bleed radius: the wet
    // front carries pigment beyond the painted edge into irregular tendrils. Wash
    // mode only (needs the masstone + a substrate to bloom onto); driven by the same
    // wet-edge `strength`.
    if km
        && strength > 0.0
        && let Some(backdrop) = backdrop
        && let Some(wash_color) = wash_color
        && let Some(mean_pigment) = mean_body_pigment(coverage, wash_color, width, region)
    {
        let radius_px = (rim_px as f32 * BLEED_RADIUS_MULT).round() as u32;
        outward_bleed(
            canvas,
            backdrop,
            coverage,
            mean_pigment,
            width,
            height,
            region,
            radius_px,
            strength,
            granulation,
            noise_seed,
        );
    }
}

/// The stroke's coverage bounding box `(x, y, w, h)` — the pixels with any
/// deposited pigment. `None` if the stroke painted nothing. Used to scope
/// [`apply_wash_settle`] to the stroke instead of the whole canvas.
#[must_use]
pub fn coverage_bbox(coverage: &[f32], width: u32, height: u32) -> Option<(u32, u32, u32, u32)> {
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (u32::MAX, u32::MAX, 0u32, 0u32);
    let mut any = false;
    for y in 0..height {
        let row = y as usize * width as usize;
        for x in 0..width {
            if coverage[row + x as usize] > 1.0 / 255.0 {
                any = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if !any {
        return None;
    }
    Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `w×h` straight-sRGB8 canvas filled with one opaque colour, plus a
    /// coverage buffer that is a filled disc of `radius` at the centre (a wash
    /// blob). Returns `(canvas, coverage)`.
    fn disc_canvas(w: u32, h: u32, color: [u8; 3], radius: f32) -> (Vec<u8>, Vec<f32>) {
        let n = (w * h) as usize;
        let mut canvas = vec![0u8; n * 4];
        let mut cov = vec![0.0f32; n];
        let (cx, cy) = (w as f32 / 2.0, h as f32 / 2.0);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let d = (((x as f32 - cx).powi(2)) + ((y as f32 - cy).powi(2))).sqrt();
                if d <= radius {
                    canvas[i * 4] = color[0];
                    canvas[i * 4 + 1] = color[1];
                    canvas[i * 4 + 2] = color[2];
                    canvas[i * 4 + 3] = 255;
                    cov[i] = 1.0;
                }
            }
        }
        (canvas, cov)
    }

    fn luma(canvas: &[u8], w: u32, x: u32, y: u32) -> f32 {
        let i = ((y * w + x) as usize) * 4;
        (srgb_to_linear_byte(canvas[i])
            + srgb_to_linear_byte(canvas[i + 1])
            + srgb_to_linear_byte(canvas[i + 2]))
            / 3.0
    }

    #[test]
    fn wet_edge_darkens_the_rim_not_the_centre() {
        // THE physics check: a uniform wash blob must come out darker near its
        // boundary (the inner shoulder) and unchanged at its centre — edge
        // darkening, not a flat tint and not a silhouette outline.
        let (w, h) = (80u32, 80u32);
        let (mut canvas, cov) = disc_canvas(w, h, [120, 120, 120], 28.0);
        let centre_before = luma(&canvas, w, 40, 40);
        // A ring pixel ~2px inside the boundary (radius 28; pick r≈26 at the top).
        let rim_x = 40u32;
        let rim_y = 40 - 26; // 14
        let rim_before = luma(&canvas, w, rim_x, rim_y);
        assert!(
            (centre_before - rim_before).abs() < 1e-6,
            "uniform before settle"
        );

        // No wash_color ⇒ the gamma fallback path (this test validates that).
        apply_wash_settle(
            &mut canvas,
            None,
            &cov,
            None,
            w,
            h,
            (0, 0, w, h),
            0.8,
            5,
            0.0,
            12345,
            EdgeStyle::Wet,
        );

        let centre_after = luma(&canvas, w, 40, 40);
        // Centre essentially unchanged (deep interior: cov ≈ blur(cov) → rim ≈ 0).
        assert!(
            (centre_after - centre_before).abs() < 0.01,
            "centre must stay put: {centre_before} -> {centre_after}"
        );
        // Somewhere on the inner shoulder must darken meaningfully.
        let mut max_darkening = 0.0f32;
        for dy in 0..14i32 {
            let yy = (40 - 28 + 2 + dy) as u32; // sweep the inner shoulder band, top side
            let before = {
                // recompute "before" luma on the original color where covered
                let i = ((yy * w + 40) as usize) * 4;
                if canvas[i + 3] == 0 {
                    continue;
                }
                centre_before
            };
            let after = luma(&canvas, w, 40, yy);
            max_darkening = max_darkening.max(before - after);
        }
        assert!(
            max_darkening > 0.02,
            "inner shoulder must darken (wet edge); max darkening {max_darkening}"
        );
    }

    #[test]
    fn zero_strength_is_a_noop() {
        let (w, h) = (40u32, 40u32);
        let (mut canvas, cov) = disc_canvas(w, h, [90, 130, 200], 14.0);
        let before = canvas.clone();
        apply_wash_settle(
            &mut canvas,
            None,
            &cov,
            None,
            w,
            h,
            (0, 0, w, h),
            0.0,
            5,
            0.0,
            1,
            EdgeStyle::Wet,
        );
        assert_eq!(canvas, before, "strength 0 must not touch the canvas");
    }

    #[test]
    fn settle_is_deterministic() {
        let (w, h) = (48u32, 48u32);
        let (mut a, cov) = disc_canvas(w, h, [200, 60, 60], 16.0);
        let mut b = a.clone();
        let masstone = vec![[0.3f32, 0.1, 0.1]; (w * h) as usize];
        let wc = Some(masstone.as_slice());
        apply_wash_settle(
            &mut a,
            None,
            &cov,
            wc,
            w,
            h,
            (0, 0, w, h),
            0.7,
            4,
            0.8,
            999,
            EdgeStyle::Wet,
        );
        apply_wash_settle(
            &mut b,
            None,
            &cov,
            wc,
            w,
            h,
            (0, 0, w, h),
            0.7,
            4,
            0.8,
            999,
            EdgeStyle::Wet,
        );
        assert_eq!(
            a, b,
            "settle must be deterministic for the same seed (HR-5)"
        );
    }

    #[test]
    fn burnt_edge_makes_a_grainier_rim_than_wet() {
        // Dry-media (burnt) edges break the rim into high-frequency paper-tooth
        // speckle, where the wet rim undulates smoothly. Measure the variance of
        // the darkened rim band: burnt must be grainier (higher variance) than wet
        // for the same stroke + strength.
        let (w, h) = (80u32, 80u32);
        // High-pass measure: the discrete Laplacian `|v[x-1] − 2v[x] + v[x+1]|`
        // along the inner shoulder, summed over several rows. A second difference
        // zeroes out smooth gradients (the low-freq wet undulation) and lights up
        // only the high-frequency tooth grain (burnt).
        let highpass = |canvas: &[u8]| {
            let get = |x: u32, y: u32| {
                srgb_to_linear_byte(canvas[((y * w + x) as usize) * 4 + 2]) // blue channel
            };
            let mut total = 0.0f32;
            for y in 13..20u32 {
                for x in 17..63u32 {
                    total += (get(x - 1, y) - 2.0 * get(x, y) + get(x + 1, y)).abs();
                }
            }
            total
        };
        let (mut wet, cov) = disc_canvas(w, h, [60, 90, 200], 28.0);
        let mut burnt = wet.clone();
        // Both gamma-path (no wash_color): compares the wet smooth rim vs burnt speckle.
        apply_wash_settle(
            &mut wet,
            None,
            &cov,
            None,
            w,
            h,
            (0, 0, w, h),
            0.8,
            5,
            0.0,
            7,
            EdgeStyle::Wet,
        );
        apply_wash_settle(
            &mut burnt,
            None,
            &cov,
            None,
            w,
            h,
            (0, 0, w, h),
            0.8,
            5,
            0.0,
            7,
            EdgeStyle::Burnt,
        );
        let hp_wet = highpass(&wet);
        let hp_burnt = highpass(&burnt);
        assert!(
            hp_burnt > hp_wet * 1.5,
            "burnt rim must be grainier (more high-freq) than wet: burnt {hp_burnt} vs wet {hp_wet}"
        );
    }

    #[test]
    fn km_deposit_endpoints_and_monotone() {
        // Finite-thickness K–M: t=0 ⇒ backdrop; t→∞ ⇒ masstone (R∞); strictly
        // between for finite t (more pigment = closer to the masstone).
        let (masstone, rg) = (0.1f32, 0.8f32); // dark pigment over a light glaze
        assert!(
            (km_deposit(masstone, rg, 0.0) - rg).abs() < 1e-6,
            "t=0 is the backdrop"
        );
        let big = km_deposit(masstone, rg, 80.0);
        assert!(
            (big - masstone).abs() < 0.02,
            "large t converges to the masstone: {big}"
        );
        let (a, b, c) = (
            km_deposit(masstone, rg, 0.2),
            km_deposit(masstone, rg, 0.6),
            km_deposit(masstone, rg, 1.5),
        );
        assert!(
            rg > a && a > b && b > c && c > masstone,
            "monotone rg→masstone: {a} {b} {c}"
        );
        // White/clear pigment (no absorption) is a no-op at any depth.
        assert!(
            (km_deposit(1.0, rg, 3.0) - rg).abs() < 1e-6,
            "clear pigment deposits nothing"
        );
    }

    #[test]
    fn km_rim_darkens_and_saturates_toward_masstone() {
        // A transparent glaze (lighter than the pigment masstone) edge-darkens at the
        // shoulder TOWARD the masstone — darker AND more saturated (the hue shift a
        // gamma can't do) — while the centre is untouched.
        let (w, h) = (80u32, 80u32);
        let (mut canvas, cov) = disc_canvas(w, h, [150, 150, 200], 28.0); // light blue glaze
        let masstone = vec![[0.02f32, 0.02, 0.50]; (w * h) as usize]; // saturated dark blue
        let backdrop = vec![255u8; (w * h * 4) as usize]; // white paper
        let chroma = |canvas: &[u8], x: u32, y: u32| {
            let i = ((y * w + x) as usize) * 4;
            let (r, g, b) = (
                srgb_to_linear_byte(canvas[i]),
                srgb_to_linear_byte(canvas[i + 1]),
                srgb_to_linear_byte(canvas[i + 2]),
            );
            r.max(g).max(b) - r.min(g).min(b)
        };
        let centre_before = luma(&canvas, w, 40, 40);
        let chroma_before = chroma(&canvas, 40, 14);
        apply_wash_settle(
            &mut canvas,
            Some(&backdrop),
            &cov,
            Some(masstone.as_slice()),
            w,
            h,
            (0, 0, w, h),
            0.9,
            5,
            0.0, // isolate the rim (no granulation)
            7,
            EdgeStyle::Wet,
        );
        assert!(
            (luma(&canvas, w, 40, 40) - centre_before).abs() < 0.01,
            "centre stays put (rim ≈ 0)"
        );
        let (mut max_dark, mut max_chroma_gain) = (0.0f32, 0.0f32);
        for dy in 0..14u32 {
            let yy = 40 - 28 + 2 + dy;
            if canvas[((yy * w + 40) as usize) * 4 + 3] == 0 {
                continue;
            }
            max_dark = max_dark.max(centre_before - luma(&canvas, w, 40, yy));
            max_chroma_gain = max_chroma_gain.max(chroma(&canvas, 40, yy) - chroma_before);
        }
        assert!(max_dark > 0.02, "K–M rim darkens the shoulder: {max_dark}");
        assert!(
            max_chroma_gain > 0.03,
            "K–M rim adds saturation (hue shift): {max_chroma_gain}"
        );
    }

    #[test]
    fn granulation_mottles_a_flat_glaze() {
        // A FLAT wash (uniform glaze, no edge) must gain a mottled granular texture:
        // pigment sediments into the paper valleys (darker) and lifts off the crests
        // (lighter toward the white substrate) — mass-conserving, so it appears as
        // variance where there was none, with NO edge involved (strength 0).
        let (w, h) = (40u32, 40u32);
        let n = (w * h) as usize;
        let mut canvas = vec![0u8; n * 4];
        for p in 0..n {
            canvas[p * 4..p * 4 + 4].copy_from_slice(&[120, 120, 180, 255]); // flat glaze
        }
        let cov = vec![1.0f32; n];
        let masstone = vec![[0.02f32, 0.02, 0.40]; n]; // saturated dark-blue pigment
        let backdrop = vec![255u8; n * 4]; // white paper
        let variance = |c: &[u8]| {
            let vals: Vec<f32> = (0..n)
                .map(|p| luma(c, w, (p as u32) % w, (p as u32) / w))
                .collect();
            let m = vals.iter().sum::<f32>() / n as f32;
            vals.iter().map(|v| (v - m).powi(2)).sum::<f32>() / n as f32
        };
        let var_before = variance(&canvas);
        assert!(var_before < 1e-6, "starts perfectly flat");
        apply_wash_settle(
            &mut canvas,
            Some(&backdrop),
            &cov,
            Some(masstone.as_slice()),
            w,
            h,
            (0, 0, w, h),
            0.0, // no rim — granulation only
            4,
            0.9,
            321,
            EdgeStyle::Wet,
        );
        assert!(
            variance(&canvas) > 1e-3,
            "granulation introduces mottle on a flat wash: {}",
            variance(&canvas)
        );
    }

    #[test]
    fn outward_bleed_deposits_a_fringe_past_the_silhouette() {
        // The bloom: a feathered pigment fringe must appear OUTSIDE the painted disc,
        // where before settle there was only bare white paper — and it must taper
        // (nothing far out).
        let (w, h) = (96u32, 96u32);
        let n = (w * h) as usize;
        let mut canvas = vec![255u8; n * 4]; // white paper everywhere
        let backdrop = canvas.clone();
        let mut cov = vec![0.0f32; n];
        let (cx, cy, radius) = (48.0f32, 48.0f32, 20.0f32);
        let blue = [50u8, 80, 200];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                if d <= radius {
                    canvas[i * 4..i * 4 + 3].copy_from_slice(&blue);
                    cov[i] = 1.0;
                }
            }
        }
        let blue_lin = [
            srgb_to_linear_byte(blue[0]),
            srgb_to_linear_byte(blue[1]),
            srgb_to_linear_byte(blue[2]),
        ];
        let masstone = vec![blue_lin; n];
        apply_wash_settle(
            &mut canvas,
            Some(&backdrop),
            &cov,
            Some(masstone.as_slice()),
            w,
            h,
            (28, 28, 40, 40), // the disc bbox
            0.9,
            6,
            0.0, // isolate the bleed (no granulation)
            42,
            EdgeStyle::Wet,
        );
        // Count blue-tinted pixels in a fringe band just outside the disc, and confirm
        // far-out pixels stayed bare white (the fringe tapers).
        let (mut bloomed, mut far_tinted) = (0u32, 0u32);
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                let d = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt();
                let bluer = canvas[i * 4 + 2] as i32 > canvas[i * 4] as i32 + 4;
                if (21.0..30.0).contains(&d) && bluer {
                    bloomed += 1;
                }
                if d > 55.0 && bluer {
                    far_tinted += 1;
                }
            }
        }
        assert!(
            bloomed > 25,
            "outward bleed deposits a fringe past the disc: {bloomed} px"
        );
        assert_eq!(
            far_tinted, 0,
            "the fringe tapers — nothing blooms far out: {far_tinted}"
        );
    }

    #[test]
    fn coverage_bbox_tracks_painted_pixels() {
        let (w, h) = (32u32, 32u32);
        let (_canvas, cov) = disc_canvas(w, h, [255, 255, 255], 8.0);
        let bbox = coverage_bbox(&cov, w, h).expect("disc paints pixels");
        // Disc centred at (16,16) radius 8 → bbox roughly [8..24].
        assert!(bbox.0 >= 7 && bbox.0 <= 9, "bbox x near 8: {bbox:?}");
        assert!(bbox.2 >= 15 && bbox.2 <= 18, "bbox w near 16: {bbox:?}");
    }
}
