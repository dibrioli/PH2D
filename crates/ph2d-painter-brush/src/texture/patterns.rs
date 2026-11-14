//! Procedural texture **patterns** — the per-coordinate samplers behind every [`super::TextureKind`].
//!
//! Clean-room from Blender's procedural texture set (Clouds / Marble / Wood
//! / Musgrave / Stucci / Magic / Distorted-Noise / Blend) plus painting-useful extras (paper grain,
//! crosshatch, halftone dots, grid, bricks). Every sampler returns a coverage multiplier in `[0, 1]`
//! and is **transcendental-free** (HR-5 determinism): only `+ - * /`, `floor`, `abs`, `sqrt` — the
//! `sin`/`cos` of Blender's Marble/Wood/Magic become the polynomial periodic [`wave01`].

use super::{ImageMask, TextureKind};

mod flakes;
mod layer;
/// ⚠️ **`pub(crate)` e nao `private`, e a razao esta nos quatro helpers ESCALARES que ele exporta**
/// (`hash2` · `smoothstep` · `lerp` · `ifloor`): eles respondem perguntas genericas — *hash um ponto
/// de rede*, *interpole suave* — e nao geometria de textura. O `stroke::roughen` os consome para o
/// campo 1-D do `Rough`; escrever um segundo `smoothstep` nesta crate seria a segunda resposta que
/// esta casa recusa em todo lado.
pub(crate) mod math;
mod paper_tile;
mod preview;
mod specs;
mod tileable;
pub use layer::render_texture_layer;
use math::*;
pub use preview::{render_texture_preview, render_tiled_preview};
pub use specs::{Escala, ParamSpec, param_escala, param_inerte, param_specs};
pub use tileable::{analytic_needs_hash_wrap, analytic_tile_period, lattice_tileable};

/// Evaluate `kind` at texture coords `tex` (after the mapping resolved them), shaped by the kind's
/// `params` (slots `0`/`1` = universal Contrast / Brightness, slot `2` = the kind's shape knob; see
/// [`super::param_specs`]). Shared by [`super::sample`] and [`super::sample_unit`].
#[must_use]
pub(super) fn sample_kind(
    kind: TextureKind,
    tex: [f32; 2],
    params: [f32; super::MAX_TEX_PARAMS],
    image: Option<&ImageMask>,
) -> f32 {
    sample_kind_t(kind, tex, params, image, [0, 0], false)
}

/// [`sample_kind`] with a lattice **wrap** period `(pu, pv)` in cells (`[0, 0]` = no wrap ⇒ byte-identical):
/// the [`lattice_tileable`] kinds repeat seamlessly across a sprite span of exactly that many cells.
/// Non-lattice kinds ignore it (analytic / bitmap tiling is handled by their own period, not the hash).
#[must_use]
pub(super) fn sample_kind_t(
    kind: TextureKind,
    tex: [f32; 2],
    params: [f32; super::MAX_TEX_PARAMS],
    image: Option<&ImageMask>,
    period: [i32; 2],
    dupla: bool,
) -> f32 {
    let (u, v) = (tex[0], tex[1]);
    let (pu, pv) = (period[0], period[1]);
    let k = &params[2..]; // the kind's shape knobs (see `param_specs`)
    let raw = match kind {
        TextureKind::None => 1.0,
        TextureKind::Noise => noise_t(u, v, k, pu, pv),
        TextureKind::Checker => checker(u, v, k),
        TextureKind::Voronoi => voronoi_t(u, v, k, pu, pv),
        TextureKind::Stripes => stripes(u, v, k),
        TextureKind::Clouds => clouds_t(u, v, k, pu, pv),
        TextureKind::DistortedNoise => distorted_noise_t(u, v, k, pu, pv),
        TextureKind::Magic => magic(u, v, k),
        TextureKind::Marble => marble(u, v, k),
        TextureKind::Musgrave => musgrave_t(u, v, k, pu, pv),
        TextureKind::Wood => wood(u, v, k),
        TextureKind::Stucci => stucci_t(u, v, k, pu, pv),
        TextureKind::Gradient => gradient(u, k),
        TextureKind::Grain => grain_t(u, v, k, pu, pv),
        TextureKind::Crosshatch => crosshatch(u, v, k),
        TextureKind::Dots => dots_t(u, v, k, pu, pv, 0.0),
        TextureKind::Grid => grid(u, v, k),
        TextureKind::Bricks => bricks(u, v, k, 0.0),
        TextureKind::Waves => waves(u, v, k),
        TextureKind::Chevron => chevron(u, v, k),
        TextureKind::Diamonds => diamonds(u, v, k),
        TextureKind::Triangles => triangles(u, v, k),
        TextureKind::Hexagons => hexagons(u, v, k),
        TextureKind::Scales => scales_t(u, v, k, pu, pv),
        TextureKind::Weave => weave(u, v, k),
        TextureKind::Flakes => flakes::flakes(u, v, k),
        // Paper presets: pre-baked seamless 256² tooth tiles (GRAN-2 rota 2 — the old on-the-fly
        // generator's spectrum was a low-frequency blotch, ~90-98% of the energy at λ > 32 px,
        // measured by FFT: the direct cause of the "mottled" wash). One bilinear wrap-fetch per
        // sample; the fibre character (laid lines) is baked into each tile's ridge, keeping the
        // lookup isotropic. Two reads (B33): under Tiling the second one snaps to the sprite
        // `period` so the seam stays exact (seam-tested).
        TextureKind::PaperCold => paper_tile::sample_t(0, u, v, period, dupla),
        TextureKind::PaperRough => paper_tile::sample_t(1, u, v, period, dupla),
        TextureKind::PaperHot => paper_tile::sample_t(2, u, v, period, dupla),
        // No tone slots: the four sliders are the bristle knobs.
        TextureKind::Bristles => return super::bristles::sample_unit(&params, u, v),
        TextureKind::Image => match image {
            Some(img) => sample_image(img, u, v),
            None => 1.0, // kind is Image but no pixels supplied → inert
        },
    };
    apply_tone(raw, params[0], params[1])
}

/// [`sample_kind_t`] sabendo a largura `fw` de um píxel da tela em coordenadas da textura: os Dots leem a borda pela
/// ÁREA dele ([`dots_t`]), os Bricks também ([`bricks`]); os outros kinds, e `fw = 0`, são a amostra no centro ao bit.
#[must_use]
pub(super) fn sample_kind_px(
    kind: TextureKind,
    tex: [f32; 2],
    params: [f32; super::MAX_TEX_PARAMS],
    image: Option<&ImageMask>,
    period: [i32; 2],
    fw: f32,
    dupla: bool,
) -> f32 {
    if kind == TextureKind::Dots && fw > 0.0 {
        let raw = dots_t(tex[0], tex[1], &params[2..], period[0], period[1], fw);
        return apply_tone(raw, params[0], params[1]);
    }
    if kind == TextureKind::Bricks && fw > 0.0 {
        return apply_tone(
            bricks(tex[0], tex[1], &params[2..], fw),
            params[0],
            params[1],
        );
    }
    sample_kind_t(kind, tex, params, image, period, dupla)
}

/// Read kind knob `i` from the `&params[2..]` slice (default `0.5` if a kind reads beyond its specs).
#[inline]
pub(super) fn knob(k: &[f32], i: usize) -> f32 {
    k.get(i).copied().unwrap_or(0.5)
}

/// fBm persistence (amplitude falloff) from a `Roughness` knob: `0.30..=0.80` (`0.5` ≈ the classic
/// `0.55`). Higher = rougher (slower falloff → more high-frequency energy).
fn gain_from(rough: f32) -> f32 {
    0.30 + rough.clamp(0.0, 1.0) * 0.50
}

/// Distort `(u, v)` by a noise field, amount `0..1` (the `Warp` knob; `0` = identity), with the
/// displacement field wrapped at integer period `(pu, pv)` cells (`0` = no wrap). The offsets
/// `+5.2`/`+1.3`/… only phase-shift a field that is already `pu`-periodic, so the warped coord
/// `wu = u + a·dx(u)` still satisfies `wu(u + pu) = wu(u) + pu` — periodicity survives the warp.
fn warp_uv_t(u: f32, v: f32, amt: f32, pu: i32, pv: i32) -> (f32, f32) {
    if amt <= 1e-4 {
        return (u, v);
    }
    let a = amt * 2.5;
    let dx = value_noise_t(u + 5.2, v + 1.3, pu, pv) - 0.5;
    let dy = value_noise_t(u - 1.7, v + 8.4, pu, pv) - 0.5;
    (u + a * dx, v + a * dy)
}

/// fBm with a tunable persistence `gain` (see [`fbm`], which fixes `gain = 0.5`), with the lattice
/// wrapped at integer base period `(pu, pv)` cells (`0` = no wrap). Each octave samples at `freq×` the
/// coords, so its lattice period is `pu·freq` — an integer while `freq` stays a power of two, keeping
/// every octave seam-free at the same sprite span. `(0, 0)` = byte-identical to the plain fBm.
fn fbm_g_t(u: f32, v: f32, octaves: u32, gain: f32, pu: i32, pv: i32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
    let mut fi = 1; // integer octave multiplier — doubles with `freq` so the period scales with it
    for _ in 0..octaves {
        sum += amp * value_noise_t(u * freq, v * freq, pu * fi, pv * fi);
        norm += amp;
        amp *= gain;
        freq *= 2.0;
        fi *= 2;
    }
    sum / norm.max(1e-6)
}

/// Polynomial smooth-min (transcendental-free): blends toward `min(a, b)` with a rounded valley of
/// width `k`. `k <= 0` is the hard `min`. The Voronoi `Smoothness` knob.
fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 1e-4 {
        return a.min(b);
    }
    let h = ((k - (a - b).abs()) / k).clamp(0.0, 1.0);
    (a.min(b) - h * h * k * 0.25).max(0.0)
}

/// Universal post-process: **Contrast** (`c`) steepens/flattens around `0.5`, **Brightness** (`b`)
/// shifts. Both normalized with `0.5` = neutral, so a fresh texture is unchanged.
fn apply_tone(x: f32, c: f32, b: f32) -> f32 {
    let gain = if c >= 0.5 {
        1.0 + (c - 0.5) * 8.0
    } else {
        c * 2.0
    };
    let bias = (b - 0.5) * 2.0;
    ((x - 0.5) * gain + 0.5 + bias).clamp(0.0, 1.0)
}

/// Map a normalized Detail knob `[0,1]` to an octave count `1..=8` (`0` → 1, `0.5` → ~5).
fn octaves_from(d: f32) -> u32 {
    (1.0 + d.clamp(0.0, 1.0) * 7.0).round() as u32
}

/// Map a `Frequency` knob to a coordinate multiplier `0.3..=2.3×` (`0.35` ≈ `1×`) — pattern repeats.
pub(super) fn freq_mul(f: f32) -> f32 {
    0.3 + f.clamp(0.0, 1.0) * 2.0
}

/// Threshold `x` into a band of normalized width `w` (the shared Stripes / Waves / Chevron "Width"
/// knob; wider `w` → more covered) with a tunable edge: `soft` `0..1` → edge `0.05..=0.65`.
fn band_soft(x: f32, w: f32, soft: f32) -> f32 {
    let thr = 1.0 - w.clamp(0.05, 0.95);
    let edge = 0.05 + soft.clamp(0.0, 1.0) * 0.6;
    smoothstep(((x - thr) / edge).clamp(0.0, 1.0))
}

// ── Blender-parity procedural patterns ──────────────────────────────────────────────────────

/// One octave of value noise in `[0, 1]`: hashed lattice values, smoothstep-interpolated.
fn value_noise(u: f32, v: f32) -> f32 {
    value_noise_t(u, v, 0, 0)
}

/// [`value_noise`] with the lattice hash wrapped at integer period `(pu, pv)` cells (`0` = no wrap),
/// so it tiles seamlessly across a sprite span of exactly that many cells: cell `pu` reuses cell `0`'s
/// hash, so `noise(u) == noise(u + pu)`. `(0, 0)` = byte-identical to the plain field.
fn value_noise_t(u: f32, v: f32, pu: i32, pv: i32) -> f32 {
    let (x0, y0) = (ifloor(u), ifloor(v));
    let (sx, sy) = (smoothstep(u - u.floor()), smoothstep(v - v.floor()));
    let n00 = hash2w(x0, y0, pu, pv);
    let n10 = hash2w(x0 + 1, y0, pu, pv);
    let n01 = hash2w(x0, y0 + 1, pu, pv);
    let n11 = hash2w(x0 + 1, y0 + 1, pu, pv);
    lerp(lerp(n00, n10, sx), lerp(n01, n11, sx), sy)
}

/// **Noise**: fractal value noise. `Detail` = octaves, `Roughness` = persistence, `Warp` = domain
/// distortion. At the defaults (Detail `0`) it is a single octave — the classic fine grain. Wrapped
/// at integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap ⇒ byte-identical).
fn noise_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let (wu, wv) = warp_uv_t(u, v, knob(k, 2), pu, pv);
    fbm_g_t(
        wu,
        wv,
        octaves_from(knob(k, 0)),
        gain_from(knob(k, 1)),
        pu,
        pv,
    )
}

/// **Checker**: 2-colour cells by integer-cell parity. `Softness` blurs the cell edges (`0` = hard).
fn checker(u: f32, v: f32, k: &[f32]) -> f32 {
    let soft = knob(k, 0);
    let (a, b) = (soft_parity(u, soft), soft_parity(v, soft));
    (a + b - 2.0 * a * b).clamp(0.0, 1.0) // soft XOR of the two axis parities
}

/// **Stripes** along `u`: `Width` = band fraction, `Frequency` = stripe count, `Softness` = edge.
fn stripes(u: f32, _v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let f = (u * g) - (u * g).floor();
    band_soft(1.0 - (2.0 * f - 1.0).abs(), knob(k, 0), knob(k, 2))
}

/// A blended distance metric for [`voronoi`]: `m` `0`→Euclidean, `0.5`→Chebyshev, `1`→Manhattan.
fn cell_distance(ex: f32, ey: f32, m: f32) -> f32 {
    let cheby = ex.max(ey);
    if m < 0.5 {
        lerp((ex * ex + ey * ey).sqrt(), cheby, m * 2.0)
    } else {
        lerp(cheby, ex + ey, (m - 0.5) * 2.0)
    }
}

/// **Voronoi** cellular noise over the 3×3 neighbour cells. `Randomness` jitters cell centres off the
/// grid; `Smoothness` rounds the cell joins (polynomial [`smin`]); `Metric` morphs the cell shape
/// (Euclidean→Chebyshev→Manhattan); `Edges` crossfades the F1 cell field to the F2−F1 crack field.
/// Wrapped at integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap): only the
/// cell-centre HASH wraps (cell `p` reuses cell `0`'s jitter), while the cell POSITION stays unwrapped,
/// so the seam glues the same jittered lattice without folding the coordinate.
fn voronoi_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let rand = knob(k, 0);
    let smooth = knob(k, 1) * 0.5;
    let metric = knob(k, 2);
    let edges = knob(k, 3);
    let (cx, cy) = (ifloor(u), ifloor(v));
    let (mut f1, mut f2) = (f32::INFINITY, f32::INFINITY);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (cx + dx, cy + dy);
            // Jitter the cell point toward random (rand 0 → grid centre, 1 → fully hashed); the hash
            // wraps at the period so a border cell matches its wrapped-around twin (swapped axes ⇒
            // swapped periods).
            let jx = lerp(0.5, hash2w(gx, gy, pu, pv), rand);
            let jy = lerp(0.5, hash2w(gy, gx, pv, pu), rand);
            let (ex, ey) = ((gx as f32 + jx - u).abs(), (gy as f32 + jy - v).abs());
            let d = cell_distance(ex, ey, metric);
            if d < f1 {
                f2 = f1;
                f1 = d;
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    let cell = smin(f1, f2, smooth).clamp(0.0, 1.0); // smoothness rounds the cell joins
    let crack = (1.0 - (f2 - f1) * 2.0).clamp(0.0, 1.0); // bright at the cell borders
    lerp(cell, crack, edges)
}

/// **Clouds**: fractal noise (soft, billowy). `Detail` = octaves, `Roughness` = persistence, `Warp`.
/// Wrapped at integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap).
fn clouds_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let (wu, wv) = warp_uv_t(u, v, knob(k, 2), pu, pv);
    fbm_g_t(
        wu,
        wv,
        octaves_from(knob(k, 0)),
        gain_from(knob(k, 1)),
        pu,
        pv,
    )
}

/// **Distorted Noise**: noise sampled at a noise-warped coordinate. `Distortion` = warp, `Detail` =
/// octaves. Wrapped at integer period `(pu, pv)` cells for seamless tiling (`0` = no wrap).
fn distorted_noise_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let amt = knob(k, 0) * 3.2;
    let dx = value_noise_t(u + 3.1, v + 1.7, pu, pv) - 0.5;
    let dy = value_noise_t(u - 2.3, v + 4.9, pu, pv) - 0.5;
    fbm_g_t(
        u + amt * dx,
        v + amt * dy,
        octaves_from(knob(k, 1)),
        0.55,
        pu,
        pv,
    )
}

/// **Magic**: nested periodic waves. `Distortion` = wave coupling, `Complexity` = adds a 4th term.
fn magic(u: f32, v: f32, k: &[f32]) -> f32 {
    let kk = knob(k, 0) * 2.0;
    let cx = knob(k, 1);
    let a = wave01(u + wave01(v * 0.7) * kk);
    let b = wave01(v - wave01(u * 1.3 + a) * kk);
    let c = wave01(u * 0.5 + v * 0.5 + a * b);
    let d = wave01(v * 0.3 - u * 0.6 + c * cx * 2.0);
    lerp((a + b + c) * (1.0 / 3.0), (a + b + c + d) * 0.25, cx)
}

/// **Marble**: turbulence-distorted diagonal veins. `Turbulence` = distortion, `Frequency` = vein count.
fn marble(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    wave01((u + v) * 1.5 * g + turbulence(u, v, 5) * (knob(k, 0) * 6.0))
}

/// **Musgrave**: ridged multifractal. `Detail` = octaves, `Roughness` = persistence, `Sharpness` = ridge
/// exponent. Wrapped at integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap).
fn musgrave_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let gain = gain_from(knob(k, 1));
    let sharp = knob(k, 2);
    let (mut value, mut amp, mut freq, mut norm, mut weight) = (0.0, 1.0, 1.0, 0.0, 1.0);
    let mut fi = 1; // integer octave multiplier — the lattice period scales with `freq`
    for _ in 0..octaves_from(knob(k, 0)) {
        let ridge = 1.0 - (2.0 * value_noise_t(u * freq, v * freq, pu * fi, pv * fi) - 1.0).abs();
        let r2 = ridge * ridge;
        value += lerp(r2, r2 * r2, sharp) * weight * amp;
        weight = (ridge * 2.0).clamp(0.0, 1.0);
        norm += amp;
        amp *= gain;
        freq *= 2.0;
        fi *= 2;
    }
    (value / norm.max(1e-6)).clamp(0.0, 1.0)
}

/// **Wood**: concentric growth rings. `Turbulence` = ring distortion, `Rings` = ring frequency.
fn wood(u: f32, v: f32, k: &[f32]) -> f32 {
    let rings = freq_mul(knob(k, 1));
    let r = (u * u + v * v).sqrt();
    wave01(r * 2.0 * rings + turbulence(u, v, 3) * (knob(k, 0) * 2.0))
}

/// **Stucci**: thresholded fractal noise — rough plaster. `Detail` = octaves, `Depth` = threshold
/// spread, `Warp` = domain distortion.
/// Wrapped at integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap).
fn stucci_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let (wu, wv) = warp_uv_t(u, v, knob(k, 2), pu, pv);
    let n = fbm_g_t(wu, wv, octaves_from(knob(k, 0)), 0.55, pu, pv);
    let depth = 0.2 + knob(k, 1) * 0.6;
    smoothstep(((n - 0.35) / depth).clamp(0.0, 1.0))
}

/// **Gradient** (Blender's `Blend`): a ramp `0→1` per tile. `Curve` = ease shape, `Repeat` = tile count.
fn gradient(u: f32, k: &[f32]) -> f32 {
    let rep = 1.0 + knob(k, 1) * 5.0;
    let f = (u * rep) - (u * rep).floor();
    let curve = knob(k, 0);
    if curve < 0.5 {
        lerp(f, smoothstep(f), curve * 2.0) // linear → smoothstep
    } else {
        let s = smoothstep(f);
        lerp(s, smoothstep(s), (curve - 0.5) * 2.0) // smoothstep → extra-eased
    }
}

// ── Painting-useful extras ──────────────────────────────────────────────────────────────────

/// **Grain**: fine fractal noise — paper / canvas tooth. `Detail`, `Roughness`, `Warp`. Wrapped at
/// integer period `(pu, pv)` cells for seamless any-size tiling (`0` = no wrap). Grain samples at `6×`
/// the coords, so the lattice period across the same sprite span is `6·p` cells.
fn grain_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let (pu, pv) = (pu * 6, pv * 6);
    let (wu, wv) = warp_uv_t(u * 6.0, v * 6.0, knob(k, 2), pu, pv);
    fbm_g_t(
        wu,
        wv,
        octaves_from(knob(k, 0)),
        gain_from(knob(k, 1)),
        pu,
        pv,
    )
}

/// **Crosshatch**: crossed diagonal hatch lines. `Thickness` = line width, `Frequency` = line count.
fn crosshatch(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let w = 0.02 + knob(k, 0) * 0.18;
    ridge_line((u + v) * g, w).max(ridge_line((u - v) * g, w))
}

/// **Dots** (halftone): a round dot per tile. `Radius` = dot size, `Softness` = edge, `Randomness` = jitter.
/// The `Randomness` jitter hashes each cell, so seamless tiling needs the hash wrapped at integer period
/// `(pu, pv)` cells (`analytic_needs_hash_wrap`); `(0, 0)` = no wrap ⇒ byte-identical to the plain field.
/// `fw` = a largura de um píxel em células: a fracção do píxel dentro do disco (uma rampa de um píxel na borda, nunca
/// mais que a área do ponto); `0` = o degrau num ponto. O Halftone tem raio 2,7 px: o degrau eram pontos em blocos.
fn dots_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32, fw: f32) -> f32 {
    let rand = knob(k, 2);
    let (cu, cv) = ((u + 0.5).floor(), (v + 0.5).floor());
    // The jitter hash wraps per axis so cell `p` reuses cell `0`'s offset (the second read swaps the axes,
    // so it swaps the periods too) — the same trick as the lattice / Voronoi hash-wrap.
    let ju = (hash2w(cu as i32, cv as i32, pu, pv) - 0.5) * rand;
    let jv = (hash2w(cv as i32 + 7, cu as i32 + 3, pv, pu) - 0.5) * rand;
    let (du, dv) = (u - cu - ju, v - cv - jv);
    let d = (du * du + dv * dv).sqrt();
    let r = (0.12 + knob(k, 0) * 0.46).max(0.02);
    let edge = (1.0 - (d / r).min(1.0)).max(0.0);
    let disco = if fw > 0.0 {
        ((r - d) / fw + 0.5)
            .clamp(0.0, 1.0)
            .min(std::f32::consts::PI * r * r / (fw * fw))
    } else {
        f32::from(d < r)
    };
    lerp(disco, smoothstep(edge), knob(k, 1)) // Softness: hard disk ↔ soft
}

/// **Grid**: thin lattice lines (mesh / graph-paper). `Thickness` = line width, `Frequency` = cell count.
fn grid(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let w = 0.02 + knob(k, 0) * 0.12;
    ridge_line(u * g, w).max(ridge_line(v * g, w))
}

/// **Bricks**: rectangles separated by mortar gaps. `Gap` = mortar width, `Bond` = per-row offset.
/// `fw` = a largura de um píxel em células: a fracção do píxel dentro do tijolo; `0` = o degrau num ponto.
fn bricks(u: f32, v: f32, k: &[f32], fw: f32) -> f32 {
    let row = ifloor(v);
    let offset = if row & 1 == 0 { 0.0 } else { knob(k, 1) };
    let mortar = (knob(k, 0) * 0.2).clamp(0.02, 0.45);
    if fw > 0.0 {
        return super::pela_area::dentro_do_periodo(u + offset, mortar, fw)
            * super::pela_area::dentro_do_periodo(v, mortar, fw);
    }
    let fu = (u + offset) - (u + offset).floor();
    let fv = v - v.floor();
    let inside = |f: f32| f > mortar && f < 1.0 - mortar;
    if inside(fu) && inside(fv) { 1.0 } else { 0.0 }
}

// ── Vector-app geometric patterns ───────────────────────────────────────────────────────────

/// **Waves**: horizontal bands rippled along `x`. `Width` = band fraction, `Frequency`, `Ripple`.
fn waves(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let ripple = knob(k, 2) * 1.2;
    band_soft(
        wave01(v * g + (wave01(u) - 0.5) * ripple),
        knob(k, 0),
        2.0 / 3.0,
    )
}

/// **Chevron**: V-shaped zigzag bands. `Width` = band fraction, `Frequency`, `Softness`.
fn chevron(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let zig = (2.0 * (u * g - (u * g + 0.5).floor())).abs(); // triangle 0→1 across each unit
    band_soft(wave01(v + zig * 0.5), knob(k, 0), knob(k, 2))
}

/// Soft parity of `floor(x)` (`0`/`1`) — the per-axis building block of the soft Checker / Diamonds.
/// Flat at this cell's parity in the interior; crossfades to the neighbour's at the borders (`soft`
/// `0` = hard step).
fn soft_parity(x: f32, soft: f32) -> f32 {
    let cp = (ifloor(x) & 1) as f32;
    let f = x - x.floor();
    let w = 0.002 + soft.clamp(0.0, 1.0) * 0.5;
    let mid = (smoothstep((f / w).min(1.0)) * smoothstep(((1.0 - f) / w).min(1.0))).clamp(0.0, 1.0);
    lerp(1.0 - cp, cp, mid)
}

/// **Diamonds** (harlequin): a 45°-rotated checker. `Softness` blurs the diamond edges.
fn diamonds(u: f32, v: f32, k: &[f32]) -> f32 {
    let soft = knob(k, 0);
    let (a, b) = (soft_parity(u + v, soft), soft_parity(u - v, soft));
    (a + b - 2.0 * a * b).clamp(0.0, 1.0)
}

/// **Triangles**: an equilateral triangular grid (sheared onto the lattice: `q.y = v/(√3/2)`,
/// `q.x = u − v/√3`), two-toned by the rhombus anti-diagonal. `Softness` blurs the split.
fn triangles(u: f32, v: f32, k: &[f32]) -> f32 {
    const INV_H: f32 = 1.154_700_5; // 1 / (√3/2) — unit-triangle row height
    const SHEAR: f32 = 0.577_350_3; // 1 / √3
    let qx = u - v * SHEAR;
    let qy = v * INV_H;
    let (fx, fy) = (qx - qx.floor(), qy - qy.floor());
    let w = 0.002 + knob(k, 0).clamp(0.0, 1.0) * 0.4;
    smoothstep((((fx + fy) - (1.0 - w)) / w).clamp(0.0, 1.0))
}

/// **Hexagons** (honeycomb): Voronoi cells of a triangular lattice. `Rim` = rim softness/width,
/// `Frequency` = cell count.
fn hexagons(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let (u, v) = (u * g, v * g);
    const GX: f32 = 1.0;
    const GY: f32 = 1.732_05; // √3
    // Two interleaved square grids whose union is the hex-centre (triangular) lattice.
    let a = [(u / GX).round() * GX, (v / GY).round() * GY];
    let b = [
        ((u - GX * 0.5) / GX).round() * GX + GX * 0.5,
        ((v - GY * 0.5) / GY).round() * GY + GY * 0.5,
    ];
    let d2 = |c: [f32; 2]| (u - c[0]) * (u - c[0]) + (v - c[1]) * (v - c[1]);
    let c = if d2(a) < d2(b) { a } else { b };
    let (px, py) = ((u - c[0]).abs(), (v - c[1]).abs());
    let hd = (px * 0.866_025 + py * 0.5).max(py); // hexagon distance, 0 centre → ~0.5 rim
    smoothstep((hd / (0.1 + knob(k, 0) * 0.8)).min(1.0))
}

/// **Scales** (fish-scale): overlapping rows of ringed discs. `Radius` = scale size, `Softness` = edge,
/// `Randomness` = per-scale jitter. The jitter hashes each scale, so seamless tiling needs the hash wrapped
/// at integer period `(pu, pv)` cells (`analytic_needs_hash_wrap`; `(0, 0)` = no wrap ⇒ byte-identical). The
/// row-parity offset (`off`) repeats only every 2 rows, so a seamless `pv` must be EVEN — the caller's
/// Size-snap guarantees it (Scales' `v` period is `2`, so the snapped span is a whole number of 2-row
/// periods → `pv` even → the wrapped row keeps its parity).
fn scales_t(u: f32, v: f32, k: &[f32], pu: i32, pv: i32) -> f32 {
    let rand = knob(k, 2);
    let row = v.floor();
    let off = if (row as i32) & 1 == 0 { 0.0 } else { 0.5 };
    let base = (u - off + 0.5).floor() + off; // nearest scale centre on this row
    let cx = base + (hash2w(base as i32, row as i32, pu, pv) - 0.5) * rand;
    let (dx, dy) = (u - cx, v - row);
    let d = (dx * dx + dy * dy).sqrt();
    let r = 0.4 + knob(k, 0) * 0.6;
    lerp(f32::from(d >= r), smoothstep((d / r).min(1.0)), knob(k, 1)) // bright at the rim
}

/// **Weave** (basketweave): over-under woven bands. `Gap` = spacing, `Frequency` = band count.
fn weave(u: f32, v: f32, k: &[f32]) -> f32 {
    let g = freq_mul(knob(k, 1));
    let (u, v) = (u * g, v * g);
    let over_h = (ifloor(u) ^ ifloor(v)) & 1 == 0;
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let m = (0.1 + knob(k, 0) * 0.25).clamp(0.05, 0.45);
    let (horiz, vert) = ((m..1.0 - m).contains(&fv), (m..1.0 - m).contains(&fu));
    if (over_h && horiz) || (!over_h && vert) {
        1.0 // the band on top
    } else if horiz || vert {
        0.4 // the band underneath
    } else {
        0.0 // gap
    }
}

// ── Image-backed sampling ───────────────────────────────────────────────────────────────────

/// Bilinear sample of `img`'s luminance at footprint coords `(u, v)`. Unlike a procedural pattern (which
/// is continuous and tiles seamlessly), an imported image must read as ONE **centred** picture that
/// fills the dab at `size = 1`: the View-plane footprint runs `−1..1` across the dab, so we map it with
/// `u·0.5 + 0.5` → `u = 0` is the image CENTRE, `|u| = 1` its edge (Enio 2026-06-24, was repeated +
/// corner-anchored). Outside `[0, 1]` (size > 1) it still tiles via `fract`; centre-coord bilinear
/// (memory `feedback_pixel_center_vs_edge_coord`). A malformed buffer reads `1.0` (inert).
fn sample_image(img: &ImageMask, u: f32, v: f32) -> f32 {
    let (w, h) = (img.width.max(1), img.height.max(1));
    if img.lum.len() < (w as usize) * (h as usize) {
        return 1.0;
    }
    let (cu, cv) = (u * 0.5 + 0.5, v * 0.5 + 0.5);
    let x = (cu - cu.floor()) * w as f32 - 0.5;
    let y = (cv - cv.floor()) * h as f32 - 0.5;
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let wrap = |i: i32, n: u32| (i.rem_euclid(n as i32)) as usize;
    let (xi0, xi1) = (wrap(x0 as i32, w), wrap(x0 as i32 + 1, w));
    let (yi0, yi1) = (wrap(y0 as i32, h), wrap(y0 as i32 + 1, h));
    let at = |xi: usize, yi: usize| f32::from(img.lum[yi * w as usize + xi]) / 255.0;
    let top = lerp(at(xi0, yi0), at(xi1, yi0), tx);
    let bot = lerp(at(xi0, yi1), at(xi1, yi1), tx);
    lerp(top, bot, ty)
}

#[cfg(test)]
mod prancha_b33_tests;
#[cfg(test)]
mod tests;
