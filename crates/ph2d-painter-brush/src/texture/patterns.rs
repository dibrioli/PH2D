//! Procedural texture **patterns** — the per-coordinate samplers behind every [`super::TextureKind`].
//!
//! Clean-room from Blender's texture set (`render/texture/intern/texture.cc`: Clouds / Marble / Wood
//! / Musgrave / Stucci / Magic / Distorted-Noise / Blend) plus painting-useful extras (paper grain,
//! crosshatch, halftone dots, grid, bricks). Every sampler returns a coverage multiplier in `[0, 1]`
//! and is **transcendental-free** (HR-5 determinism): only `+ - * /`, `floor`, `abs`, `sqrt` — the
//! `sin`/`cos` of Blender's Marble/Wood/Magic become the polynomial periodic [`wave01`].

use super::{ImageMask, TextureKind};

/// One tunable parameter a [`TextureKind`] exposes: its label (for the panel) and neutral default
/// (normalized `[0, 1]`). The slot index is the position in [`param_specs`] /
/// [`super::TextureSettings::params`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParamSpec {
    /// English label shown next to the slider (HR-15).
    pub label: &'static str,
    /// Default value (normalized `[0, 1]`) assigned when the kind is selected.
    pub default: f32,
}

const CONTRAST: ParamSpec = ParamSpec {
    label: "Contrast",
    default: 0.5,
};
const BRIGHTNESS: ParamSpec = ParamSpec {
    label: "Brightness",
    default: 0.5,
};

/// The parameters a `kind` exposes, in [`super::TextureSettings::params`] slot order. Slots `0`/`1`
/// are always the universal Contrast / Brightness ([`apply_tone`]); a third entry is the kind's shape
/// knob (consumed by that kind's sampler). `None` exposes nothing.
#[must_use]
pub fn param_specs(kind: TextureKind) -> &'static [ParamSpec] {
    use TextureKind::*;
    match kind {
        None => &[],
        Clouds | Grain | Stucci | Musgrave => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Detail",
                default: 0.5,
            },
        ],
        Marble | Wood => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Turbulence",
                default: 0.5,
            },
        ],
        DistortedNoise | Magic => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Distortion",
                default: 0.5,
            },
        ],
        Stripes | Waves | Chevron => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Width",
                default: 0.5,
            },
        ],
        Dots | Scales => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Radius",
                default: 0.5,
            },
        ],
        Grid | Crosshatch => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Thickness",
                default: 0.4,
            },
        ],
        Bricks | Weave => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Gap",
                default: 0.4,
            },
        ],
        Hexagons => &[
            CONTRAST,
            BRIGHTNESS,
            ParamSpec {
                label: "Rim",
                default: 0.5,
            },
        ],
        _ => &[CONTRAST, BRIGHTNESS],
    }
}

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
    let (u, v) = (tex[0], tex[1]);
    let p = params[2]; // the kind's shape knob
    let raw = match kind {
        TextureKind::None => 1.0,
        TextureKind::Noise => value_noise(u, v),
        TextureKind::Checker => checker(u, v),
        TextureKind::Voronoi => voronoi(u, v),
        TextureKind::Stripes => stripes(u, p),
        TextureKind::Clouds => clouds(u, v, p),
        TextureKind::DistortedNoise => distorted_noise(u, v, p),
        TextureKind::Magic => magic(u, v, p),
        TextureKind::Marble => marble(u, v, p),
        TextureKind::Musgrave => musgrave(u, v, p),
        TextureKind::Wood => wood(u, v, p),
        TextureKind::Stucci => stucci(u, v, p),
        TextureKind::Gradient => gradient(u),
        TextureKind::Grain => grain(u, v, p),
        TextureKind::Crosshatch => crosshatch(u, v, p),
        TextureKind::Dots => dots(u, v, p),
        TextureKind::Grid => grid(u, v, p),
        TextureKind::Bricks => bricks(u, v, p),
        TextureKind::Waves => waves(u, v, p),
        TextureKind::Chevron => chevron(u, v, p),
        TextureKind::Diamonds => diamonds(u, v),
        TextureKind::Triangles => triangles(u, v),
        TextureKind::Hexagons => hexagons(u, v, p),
        TextureKind::Scales => scales(u, v, p),
        TextureKind::Weave => weave(u, v, p),
        TextureKind::Image => match image {
            Some(img) => sample_image(img, u, v),
            None => 1.0, // kind is Image but no pixels supplied → inert
        },
    };
    apply_tone(raw, params[0], params[1])
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

/// Map a normalized Detail knob `[0,1]` to an octave count `2..=8` (`0.5` → 5).
fn octaves_from(d: f32) -> u32 {
    (2.0 + d.clamp(0.0, 1.0) * 6.0).round() as u32
}

/// Threshold `x` into a band of normalized width `w` with a soft edge (the shared Stripes / Waves /
/// Chevron "Width" knob; wider `w` → more covered).
fn band(x: f32, w: f32) -> f32 {
    let thr = 1.0 - w.clamp(0.05, 0.95);
    smoothstep(((x - thr) / 0.15).clamp(0.0, 1.0))
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

/// Parallel stripes along `u` — `Width` sets the band fraction.
fn stripes(u: f32, w: f32) -> f32 {
    let f = u - u.floor();
    band(1.0 - (2.0 * f - 1.0).abs(), w) // triangle (0 edges → 1 centre) thresholded to a band
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

/// **Clouds**: fractal Brownian noise (soft, billowy). `Detail` = octaves.
fn clouds(u: f32, v: f32, detail: f32) -> f32 {
    fbm(u, v, octaves_from(detail))
}

/// **Distorted Noise**: value noise sampled at a noise-warped coordinate. `Distortion` = warp amount.
fn distorted_noise(u: f32, v: f32, dist: f32) -> f32 {
    let amt = dist * 3.2;
    let dx = value_noise(u + 3.1, v + 1.7) - 0.5;
    let dy = value_noise(u - 2.3, v + 4.9) - 0.5;
    value_noise(u + amt * dx, v + amt * dy)
}

/// **Magic**: nested periodic waves — a swirly interference pattern. `Distortion` = wave coupling.
fn magic(u: f32, v: f32, dist: f32) -> f32 {
    let k = dist * 2.0;
    let a = wave01(u + wave01(v * 0.7) * k);
    let b = wave01(v - wave01(u * 1.3 + a) * k);
    let c = wave01(u * 0.5 + v * 0.5 + a * b);
    (a + b + c) * (1.0 / 3.0)
}

/// **Marble**: turbulence-distorted diagonal veins. `Turbulence` = vein distortion.
fn marble(u: f32, v: f32, turb: f32) -> f32 {
    wave01((u + v) * 1.5 + turbulence(u, v, 5) * (turb * 6.0))
}

/// **Musgrave**: a ridged multifractal — sharp creases. `Detail` = octaves.
fn musgrave(u: f32, v: f32, detail: f32) -> f32 {
    let mut value = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    let mut weight = 1.0;
    for _ in 0..octaves_from(detail) {
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

/// **Wood**: concentric growth rings. `Turbulence` = ring distortion.
fn wood(u: f32, v: f32, turb: f32) -> f32 {
    let r = (u * u + v * v).sqrt();
    wave01(r * 2.0 + turbulence(u, v, 3) * (turb * 2.0))
}

/// **Stucci**: thresholded fractal noise — rough plaster. `Detail` = octaves.
fn stucci(u: f32, v: f32, detail: f32) -> f32 {
    let n = fbm(u, v, octaves_from(detail));
    smoothstep(((n - 0.35) / 0.5).clamp(0.0, 1.0))
}

/// **Gradient** (Blender's `Blend`): a smooth linear ramp `0→1` repeating each tile.
fn gradient(u: f32) -> f32 {
    smoothstep(u - u.floor())
}

// ── Painting-useful extras ──────────────────────────────────────────────────────────────────

/// **Grain**: fine fractal noise — the paper / canvas tooth for dry media. `Detail` = octaves.
fn grain(u: f32, v: f32, detail: f32) -> f32 {
    fbm(u * 6.0, v * 6.0, octaves_from(detail))
}

/// **Crosshatch**: crossed diagonal hatch lines. `Thickness` = line width.
fn crosshatch(u: f32, v: f32, thick: f32) -> f32 {
    let w = 0.02 + thick * 0.18;
    ridge_line(u + v, w).max(ridge_line(u - v, w))
}

/// **Dots** (halftone): a soft round dot centred in each tile. `Radius` = dot size.
fn dots(u: f32, v: f32, radius: f32) -> f32 {
    let du = u - (u + 0.5).floor();
    let dv = v - (v + 0.5).floor();
    let d = (du * du + dv * dv).sqrt();
    let r = (0.12 + radius * 0.46).max(0.02);
    smoothstep((1.0 - (d / r).min(1.0)).max(0.0))
}

/// **Grid**: thin lines along the integer lattice (mesh / graph-paper). `Thickness` = line width.
fn grid(u: f32, v: f32, thick: f32) -> f32 {
    let w = 0.02 + thick * 0.12;
    ridge_line(u, w).max(ridge_line(v, w))
}

/// **Bricks**: running-bond rectangles separated by mortar gaps. `Gap` = mortar width.
fn bricks(u: f32, v: f32, gap: f32) -> f32 {
    let row = ifloor(v);
    let offset = if row & 1 == 0 { 0.0 } else { 0.5 };
    let fu = (u + offset) - (u + offset).floor();
    let fv = v - v.floor();
    let mortar = (gap * 0.2).clamp(0.02, 0.45);
    let inside = |f: f32| f > mortar && f < 1.0 - mortar;
    if inside(fu) && inside(fv) { 1.0 } else { 0.0 }
}

// ── Vector-app geometric patterns ───────────────────────────────────────────────────────────

/// **Waves**: horizontal bands rippled along `x` (water / silk). `Width` = band fraction.
fn waves(u: f32, v: f32, w: f32) -> f32 {
    band(wave01(v + (wave01(u) - 0.5) * 0.6), w)
}

/// **Chevron**: V-shaped zigzag bands. `Width` = band fraction.
fn chevron(u: f32, v: f32, w: f32) -> f32 {
    let zig = (2.0 * (u - (u + 0.5).floor())).abs(); // triangle 0→1 across each unit
    band(wave01(v + zig * 0.5), w)
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

/// **Hexagons** (honeycomb): Voronoi cells of a triangular lattice. `Rim` = rim softness/width.
fn hexagons(u: f32, v: f32, rim: f32) -> f32 {
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
    smoothstep((hd / (0.1 + rim * 0.8)).min(1.0))
}

/// **Scales** (fish-scale): overlapping rows of ringed discs. `Radius` = scale size.
fn scales(u: f32, v: f32, radius: f32) -> f32 {
    let row = v.floor();
    let off = if (row as i32) & 1 == 0 { 0.0 } else { 0.5 };
    let cx = (u - off + 0.5).floor() + off; // nearest scale centre on this row
    let (dx, dy) = (u - cx, v - row);
    smoothstep(((dx * dx + dy * dy).sqrt() / (0.4 + radius * 0.6)).min(1.0))
}

/// **Weave** (basketweave): over-under woven bands. `Gap` = spacing between bands.
fn weave(u: f32, v: f32, gap: f32) -> f32 {
    let over_h = (ifloor(u) ^ ifloor(v)) & 1 == 0;
    let (fu, fv) = (u - u.floor(), v - v.floor());
    let m = (0.1 + gap * 0.25).clamp(0.05, 0.45);
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
    fn stripes_width_controls_coverage() {
        assert!(stripes(0.5, 0.5) > 0.9, "band centre is covered");
        let cov = |w: f32| (0..100).map(|i| stripes(i as f32 / 100.0, w)).sum::<f32>();
        assert!(
            cov(0.8) > cov(0.3),
            "a wider Width paints more of the period"
        );
        assert!(
            (stripes(0.25, 0.5) - stripes(1.25, 0.5)).abs() < 1e-6,
            "periodic"
        );
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
        // Neutral params and a non-neutral set (high contrast, dark, extreme shape knob).
        for params in [[0.5; 4], [0.9, 0.3, 0.8, 0.5], [0.1, 0.7, 0.2, 0.5]] {
            for k in 0..TextureKind::COUNT {
                let kind = TextureKind::from_u8(k);
                for i in 0..40 {
                    let (u, v) = (i as f32 * 0.31 - 3.0, i as f32 * -0.27 + 1.5);
                    let a = sample_kind(kind, [u, v], params, None);
                    assert!(
                        (0.0..=1.0).contains(&a),
                        "{} out of range: {a}",
                        kind.name()
                    );
                    assert_eq!(
                        a,
                        sample_kind(kind, [u, v], params, None),
                        "{} must be pure",
                        kind.name()
                    );
                }
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
                    let a = sample_kind(kind, [i as f32 * 0.17, j as f32 * 0.19], [0.5; 4], None);
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
