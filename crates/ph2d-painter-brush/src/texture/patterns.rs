//! Procedural texture **patterns** — the per-coordinate samplers behind every [`super::TextureKind`].
//!
//! Clean-room from Blender's texture set (`render/texture/intern/texture.cc`: Clouds / Marble / Wood
//! / Musgrave / Stucci / Magic / Distorted-Noise / Blend) plus painting-useful extras (paper grain,
//! crosshatch, halftone dots, grid, bricks). Every sampler returns a coverage multiplier in `[0, 1]`
//! and is **transcendental-free** (HR-5 determinism): only `+ - * /`, `floor`, `abs`, `sqrt` — the
//! `sin`/`cos` of Blender's Marble/Wood/Magic become the polynomial periodic [`wave01`].

use super::{ImageMask, TextureKind};

/// Evaluate `kind` at texture coords `tex` (after the mapping resolved them). Shared by
/// [`super::sample`] (per-pixel canvas path) and [`super::sample_unit`] (the cached View stamp).
#[must_use]
pub(super) fn sample_kind(kind: TextureKind, tex: [f32; 2], image: Option<&ImageMask>) -> f32 {
    let (u, v) = (tex[0], tex[1]);
    match kind {
        TextureKind::None => 1.0,
        TextureKind::Noise => value_noise(u, v),
        TextureKind::Checker => checker(u, v),
        TextureKind::Voronoi => voronoi(u, v),
        TextureKind::Stripes => stripes(u),
        TextureKind::Clouds => clouds(u, v),
        TextureKind::DistortedNoise => distorted_noise(u, v),
        TextureKind::Magic => magic(u, v),
        TextureKind::Marble => marble(u, v),
        TextureKind::Musgrave => musgrave(u, v),
        TextureKind::Wood => wood(u, v),
        TextureKind::Stucci => stucci(u, v),
        TextureKind::Gradient => gradient(u),
        TextureKind::Grain => grain(u, v),
        TextureKind::Crosshatch => crosshatch(u, v),
        TextureKind::Dots => dots(u, v),
        TextureKind::Grid => grid(u, v),
        TextureKind::Bricks => bricks(u, v),
        TextureKind::Waves => waves(u, v),
        TextureKind::Chevron => chevron(u, v),
        TextureKind::Diamonds => diamonds(u, v),
        TextureKind::Triangles => triangles(u, v),
        TextureKind::Hexagons => hexagons(u, v),
        TextureKind::Scales => scales(u, v),
        TextureKind::Weave => weave(u, v),
        TextureKind::Image => match image {
            Some(img) => sample_image(img, u, v),
            None => 1.0, // kind is Image but no pixels supplied → inert
        },
    }
}

// ── Blender-parity procedural patterns ──────────────────────────────────────────────────────

/// One octave of value noise in `[0, 1]`: hashed lattice values, smoothstep-interpolated.
fn value_noise(u: f32, v: f32) -> f32 {
    let (x0, y0) = (ifloor(u), ifloor(v));
    let (sx, sy) = (smoothstep(u - u.floor()), smoothstep(v - v.floor()));
    let n00 = hash2(x0, y0);
    let n10 = hash2(x0 + 1, y0);
    let n01 = hash2(x0, y0 + 1);
    let n11 = hash2(x0 + 1, y0 + 1);
    lerp(lerp(n00, n10, sx), lerp(n01, n11, sx), sy)
}

/// Hard 2-colour checker: `0.0` / `1.0` by the parity of the integer cell.
fn checker(u: f32, v: f32) -> f32 {
    ((ifloor(u) ^ ifloor(v)) & 1) as f32
}

/// Soft parallel stripes along `u` — a unit-period triangle wave in `[0, 1]`.
fn stripes(u: f32) -> f32 {
    let f = u - u.floor();
    if f < 0.5 { 2.0 * f } else { 2.0 * (1.0 - f) }
}

/// Voronoi F1: nearest-feature distance over the 3×3 neighbour cells, mapped to `[0, 1]`.
fn voronoi(u: f32, v: f32) -> f32 {
    let (cx, cy) = (ifloor(u), ifloor(v));
    let mut best = f32::INFINITY;
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (gx, gy) = (cx + dx, cy + dy);
            let fx = gx as f32 + hash2(gx, gy);
            let fy = gy as f32 + hash2(gy, gx);
            let (ex, ey) = (fx - u, fy - v);
            best = best.min(ex * ex + ey * ey);
        }
    }
    best.sqrt().clamp(0.0, 1.0)
}

/// **Clouds**: fractal Brownian noise (soft, billowy) — Blender's classic `Clouds`.
fn clouds(u: f32, v: f32) -> f32 {
    fbm(u, v, 5)
}

/// **Distorted Noise**: value noise sampled at a noise-warped coordinate (domain warp).
fn distorted_noise(u: f32, v: f32) -> f32 {
    const AMT: f32 = 1.6;
    let dx = value_noise(u + 3.1, v + 1.7) - 0.5;
    let dy = value_noise(u - 2.3, v + 4.9) - 0.5;
    value_noise(u + AMT * dx, v + AMT * dy)
}

/// **Magic**: nested periodic waves — a swirly, organic interference pattern (luminance of Blender's
/// colourful `Magic`).
fn magic(u: f32, v: f32) -> f32 {
    let a = wave01(u + wave01(v * 0.7));
    let b = wave01(v - wave01(u * 1.3 + a));
    let c = wave01(u * 0.5 + v * 0.5 + a * b);
    (a + b + c) * (1.0 / 3.0)
}

/// **Marble**: turbulence-distorted diagonal bands (veins).
fn marble(u: f32, v: f32) -> f32 {
    wave01((u + v) * 1.5 + turbulence(u, v, 5) * 3.0)
}

/// **Musgrave**: a ridged multifractal — sharp creases at multiple scales (terrain-like).
fn musgrave(u: f32, v: f32) -> f32 {
    let mut value = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    let mut weight = 1.0;
    for _ in 0..5 {
        let ridge = 1.0 - (2.0 * value_noise(u * freq, v * freq) - 1.0).abs();
        let n = ridge * ridge * weight;
        value += n * amp;
        weight = (ridge * 2.0).clamp(0.0, 1.0);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    (value / norm.max(1e-6)).clamp(0.0, 1.0)
}

/// **Wood**: concentric rings (tree growth rings) warped by a little turbulence.
fn wood(u: f32, v: f32) -> f32 {
    let r = (u * u + v * v).sqrt();
    wave01(r * 2.0 + turbulence(u, v, 3))
}

/// **Stucci**: thresholded fractal noise — a rough plaster / wall relief.
fn stucci(u: f32, v: f32) -> f32 {
    let n = fbm(u, v, 3);
    smoothstep(((n - 0.35) / 0.5).clamp(0.0, 1.0))
}

/// **Gradient** (Blender's `Blend`): a smooth linear ramp `0→1` repeating each tile.
fn gradient(u: f32) -> f32 {
    smoothstep(u - u.floor())
}

// ── Painting-useful extras ──────────────────────────────────────────────────────────────────

/// **Grain**: fine multi-frequency noise — the paper / canvas tooth for dry media.
fn grain(u: f32, v: f32) -> f32 {
    let n = 0.6 * value_noise(u * 6.0, v * 6.0)
        + 0.3 * value_noise(u * 13.0, v * 13.0)
        + 0.1 * value_noise(u * 27.0, v * 27.0);
    n.clamp(0.0, 1.0)
}

/// **Crosshatch**: crossed diagonal hatch lines (the painted ink-hatch look).
fn crosshatch(u: f32, v: f32) -> f32 {
    ridge_line(u + v, 0.09).max(ridge_line(u - v, 0.09))
}

/// **Dots** (halftone): a soft round dot centred in each tile.
fn dots(u: f32, v: f32) -> f32 {
    let du = u - (u + 0.5).floor();
    let dv = v - (v + 0.5).floor();
    let d = (du * du + dv * dv).sqrt();
    smoothstep((1.0 - (d / 0.35).min(1.0)).max(0.0))
}

/// **Grid**: thin lines along the integer lattice (mesh / graph-paper).
fn grid(u: f32, v: f32) -> f32 {
    ridge_line(u, 0.06).max(ridge_line(v, 0.06))
}

/// **Bricks**: running-bond rectangles separated by mortar gaps.
fn bricks(u: f32, v: f32) -> f32 {
    let row = ifloor(v);
    let offset = if row & 1 == 0 { 0.0 } else { 0.5 };
    let fu = (u + offset) - (u + offset).floor();
    let fv = v - v.floor();
    const MORTAR: f32 = 0.08;
    let inside = |f: f32| f > MORTAR && f < 1.0 - MORTAR;
    if inside(fu) && inside(fv) { 1.0 } else { 0.0 }
}

// ── Vector-app geometric patterns ───────────────────────────────────────────────────────────

/// **Waves**: smooth horizontal bands rippled along `x` (water / silk).
fn waves(u: f32, v: f32) -> f32 {
    wave01(v + (wave01(u) - 0.5) * 0.6)
}

/// **Chevron**: V-shaped zigzag bands.
fn chevron(u: f32, v: f32) -> f32 {
    let zig = (2.0 * (u - (u + 0.5).floor())).abs(); // triangle 0→1 across each unit
    wave01(v + zig * 0.5)
}

/// **Diamonds** (harlequin): a 45°-rotated checker of diamonds.
fn diamonds(u: f32, v: f32) -> f32 {
    ((ifloor(u + v) ^ ifloor(u - v)) & 1) as f32
}

/// **Triangles**: a two-tone triangular tiling (each square split along its diagonal).
fn triangles(u: f32, v: f32) -> f32 {
    let (cu, cv) = (ifloor(u), ifloor(v));
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let upper = i32::from(fu + fv > 1.0);
    (((cu ^ cv) & 1) ^ upper) as f32
}

/// **Hexagons** (honeycomb): Voronoi cells of a triangular lattice, with bright rims.
fn hexagons(u: f32, v: f32) -> f32 {
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
    smoothstep((hd / 0.5).min(1.0))
}

/// **Scales** (fish-scale): overlapping rows of ringed discs.
fn scales(u: f32, v: f32) -> f32 {
    let row = v.floor();
    let off = if (row as i32) & 1 == 0 { 0.0 } else { 0.5 };
    let cx = (u - off + 0.5).floor() + off; // nearest scale centre on this row
    let (dx, dy) = (u - cx, v - row);
    smoothstep(((dx * dx + dy * dy).sqrt() / 0.7).min(1.0))
}

/// **Weave** (basketweave): over-under woven bands.
fn weave(u: f32, v: f32) -> f32 {
    let over_h = (ifloor(u) ^ ifloor(v)) & 1 == 0;
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let (horiz, vert) = ((0.2..0.8).contains(&fv), (0.2..0.8).contains(&fu));
    if (over_h && horiz) || (!over_h && vert) {
        1.0 // the band on top
    } else if horiz || vert {
        0.4 // the band underneath
    } else {
        0.0 // gap
    }
}

// ── Image-backed sampling ───────────────────────────────────────────────────────────────────

/// Bilinear sample of `img`'s luminance at tile coords `(u, v)` (1 unit = one image), tiled via
/// `fract`, centre-coord convention (memory `feedback_pixel_center_vs_edge_coord`). Returns `[0, 1]`;
/// transcendental-free. A malformed buffer reads `1.0` (inert) rather than panicking.
fn sample_image(img: &ImageMask, u: f32, v: f32) -> f32 {
    let (w, h) = (img.width.max(1), img.height.max(1));
    if img.lum.len() < (w as usize) * (h as usize) {
        return 1.0;
    }
    let x = (u - u.floor()) * w as f32 - 0.5;
    let y = (v - v.floor()) * h as f32 - 0.5;
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

// ── Shared noise / math helpers (transcendental-free) ───────────────────────────────────────

/// Fractal Brownian motion: octaves of [`value_noise`] at doubling frequency, halving amplitude,
/// normalised to `[0, 1]`. The soft, self-similar base of Clouds / Stucci.
fn fbm(u: f32, v: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        sum += amp * value_noise(u * freq, v * freq);
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm.max(1e-6)
}

/// Turbulence: octaves of the *absolute* signed noise `|2·noise−1|` — sharper, veiny (Marble / Wood).
fn turbulence(u: f32, v: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        sum += amp * (2.0 * value_noise(u * freq, v * freq) - 1.0).abs();
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm.max(1e-6)
}

/// Smooth period-1 wave in `[0, 1]` — `0` at integers, `1` at half-integers — the dependency-free
/// stand-in for `(1−cos 2πx)/2`: a triangle wave run through [`smoothstep`].
fn wave01(x: f32) -> f32 {
    let t = x - x.floor();
    smoothstep(1.0 - (2.0 * t - 1.0).abs())
}

/// Coverage of a thin line at each integer of `x`: `1` on the line, ramping to `0` at distance `w`.
fn ridge_line(x: f32, w: f32) -> f32 {
    let d = (x - (x + 0.5).floor()).abs(); // distance to the nearest integer, `[0, 0.5]`
    (1.0 - (d / w).min(1.0)).max(0.0)
}

/// Hash an integer lattice point to `[0, 1)` — the value-noise / Voronoi randomness.
fn hash2(ix: i32, iy: i32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    ((h >> 8) as f32) / ((1u32 << 24) as f32)
}

/// Hermite smoothstep `3t² − 2t³` on a value already in `[0, 1]` (polynomial — no transcendental).
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolate.
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `floor` to `i32` (integer cell index) — avoids the `as i32` truncation-toward-zero bug for
/// negative coordinates.
fn ifloor(x: f32) -> i32 {
    x.floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checker_alternates_by_cell_parity() {
        assert_eq!(checker(0.5, 0.5), 0.0);
        assert_eq!(checker(1.5, 0.5), 1.0);
        assert_eq!(checker(1.5, 1.5), 0.0);
        assert_eq!(checker(-0.5, 0.5), 1.0); // floor (not truncation): cell -1 has parity 1
    }

    #[test]
    fn stripes_is_a_unit_triangle_wave() {
        assert!((stripes(0.0) - 0.0).abs() < 1e-6);
        assert!((stripes(0.5) - 1.0).abs() < 1e-6);
        assert!((stripes(1.0) - 0.0).abs() < 1e-6);
        assert!((stripes(0.25) - stripes(1.25)).abs() < 1e-6); // periodic
    }

    #[test]
    fn value_noise_is_bounded_and_deterministic() {
        for i in 0..50 {
            let (u, v) = (i as f32 * 0.37, i as f32 * -0.21);
            let a = value_noise(u, v);
            assert!((0.0..=1.0).contains(&a), "noise out of range: {a}");
            assert_eq!(a, value_noise(u, v), "noise must be a pure function");
        }
        assert_ne!(value_noise(0.5, 0.5), value_noise(10.5, 10.5));
    }

    #[test]
    fn voronoi_is_bounded() {
        for i in 0..50 {
            let d = voronoi(i as f32 * 0.61, i as f32 * 0.43);
            assert!((0.0..=1.0).contains(&d), "voronoi out of range: {d}");
        }
    }

    #[test]
    fn sample_image_is_bilinear_centre_coord_and_tiles() {
        let lum = [0u8, 255, 128, 64]; // 2×2: [0,255; 128,64]
        let img = ImageMask {
            lum: &lum,
            width: 2,
            height: 2,
        };
        assert!(
            (sample_image(&img, 0.25, 0.25) - 0.0).abs() < 1e-6,
            "texel (0,0)=0"
        );
        assert!(
            (sample_image(&img, 0.75, 0.25) - 1.0).abs() < 1e-6,
            "texel (1,0)=255"
        );
        assert!((sample_image(&img, 1.25, 0.25) - sample_image(&img, 0.25, 0.25)).abs() < 1e-6);
        for k in 0..30 {
            assert!((0.0..=1.0).contains(&sample_image(&img, k as f32 * 0.17, -(k as f32) * 0.17)));
        }
    }

    #[test]
    fn every_kind_is_bounded_and_deterministic() {
        for k in 0..TextureKind::COUNT {
            let kind = TextureKind::from_u8(k);
            for i in 0..40 {
                let (u, v) = (i as f32 * 0.31 - 3.0, i as f32 * -0.27 + 1.5);
                let a = sample_kind(kind, [u, v], None);
                assert!(
                    (0.0..=1.0).contains(&a),
                    "{} out of range: {a}",
                    kind.name()
                );
                assert_eq!(
                    a,
                    sample_kind(kind, [u, v], None),
                    "{} must be pure",
                    kind.name()
                );
            }
        }
    }

    #[test]
    fn every_procedural_kind_varies_across_the_plane() {
        // Each procedural (not None / Image) must produce a non-flat field.
        for k in 1..TextureKind::COUNT {
            let kind = TextureKind::from_u8(k);
            if matches!(kind, TextureKind::Image) {
                continue;
            }
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            for i in 0..72 {
                for j in 0..72 {
                    let a = sample_kind(kind, [i as f32 * 0.17, j as f32 * 0.19], None);
                    lo = lo.min(a);
                    hi = hi.max(a);
                }
            }
            assert!(
                hi - lo > 0.05,
                "{} should vary (lo={lo} hi={hi})",
                kind.name()
            );
        }
    }
}
