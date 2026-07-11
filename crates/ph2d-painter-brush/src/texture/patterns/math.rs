//! Shared **transcendental-free** noise / hash / math leaves behind the [`super`] patterns (HR-5): only
//! `+ - * /`, `floor`, `abs`, `sqrt`. Split from `patterns.rs` for the workspace LOC cap; imported back
//! with `use math::*`. The lattice hash is period-aware ([`hash2w`]/[`wrapi`]) so a value-noise pattern
//! can tile seamlessly at any size (doc 13 #2c).

/// Turbulence: octaves of the *absolute* signed noise `|2·noise−1|` — sharper, veiny (Marble / Wood).
pub(super) fn turbulence(u: f32, v: f32, octaves: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for _ in 0..octaves {
        sum += amp * (2.0 * super::value_noise(u * freq, v * freq) - 1.0).abs();
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm.max(1e-6)
}

/// Smooth period-1 wave in `[0, 1]` — `0` at integers, `1` at half-integers — the dependency-free
/// stand-in for `(1−cos 2πx)/2`: a triangle wave run through [`smoothstep`].
pub(super) fn wave01(x: f32) -> f32 {
    let t = x - x.floor();
    smoothstep(1.0 - (2.0 * t - 1.0).abs())
}

/// Coverage of a thin line at each integer of `x`: `1` on the line, ramping to `0` at distance `w`.
pub(super) fn ridge_line(x: f32, w: f32) -> f32 {
    let d = (x - (x + 0.5).floor()).abs(); // distance to the nearest integer, `[0, 0.5]`
    (1.0 - (d / w).min(1.0)).max(0.0)
}

/// Wrap a lattice index into `[0, p)` when `p > 0` (the seamless-tiling period, in cells); `p == 0`
/// (untiled) returns it verbatim — the identity that keeps the no-wrap path byte-identical.
#[inline]
pub(super) fn wrapi(i: i32, p: i32) -> i32 {
    if p > 0 { i.rem_euclid(p) } else { i }
}

/// [`hash2`] with each axis wrapped at its period (`0` = no wrap) — the seam-free lattice hash that
/// makes cell `p` alias cell `0`, so a value-noise field repeats exactly every `p` cells.
#[inline]
pub(super) fn hash2w(ix: i32, iy: i32, pu: i32, pv: i32) -> f32 {
    hash2(wrapi(ix, pu), wrapi(iy, pv))
}

/// Hash an integer lattice point to `[0, 1)` — the value-noise / Voronoi randomness.
pub(super) fn hash2(ix: i32, iy: i32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x9E37_79B1) ^ (iy as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    ((h >> 8) as f32) / ((1u32 << 24) as f32)
}

/// Hermite smoothstep `3t² − 2t³` on a value already in `[0, 1]` (polynomial — no transcendental).
pub(super) fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Linear interpolate.
pub(super) fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `floor` to `i32` (integer cell index) — avoids the `as i32` truncation-toward-zero bug for
/// negative coordinates.
pub(super) fn ifloor(x: f32) -> i32 {
    x.floor() as i32
}
