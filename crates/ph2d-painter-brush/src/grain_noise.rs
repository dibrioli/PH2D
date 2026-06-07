//! Procedural grain noise — the CPU reference for `GrainSource::Procedural`
//! ([`crate::procedural::ProceduralGrain`]). Mirrored bit-for-bit by the WGSL in
//! `shader/stamp.wgsl` (same discipline as the OKLab/pigment coefficients): the
//! live painter renders on the CPU, the GPU stamp path must agree.
//!
//! Each generator returns a coverage multiplier in `[0, 1]` for a world-space (or
//! stamp-space, when `GrainBehavior::Moving`) sample point. The render multiplies
//! the stamp's shape alpha by it — `1.0` = full ink, `0.0` = bare paper showing
//! through. Tiling-free, resolution-independent, no 64 MB atlas.
//!
//! The four `ProceduralGrain` variants are encoded into the frozen `Stamp` ABI as
//! a small type id (no per-pixel brush access on the GPU); octaves/persistence use
//! fixed presets at v1 (the artist dials only `scale` + `depth` so far).

/// `ProceduralGrain` type ids — the low byte of `Stamp.grain_layer` when
/// `FLAG_GRAIN_PROCEDURAL` is set. Order matches the enum variants.
pub const GRAIN_SIMPLEX: u32 = 0;
pub const GRAIN_GABOR: u32 = 1;
pub const GRAIN_PAPER_WEAVE: u32 = 2;
pub const GRAIN_SPRAY_DOT: u32 = 3;

/// Integer hash → `[0, 1)`. Deterministic (HR-5): same `(x, y, seed)` → same value
/// on every platform (pure integer ops, no float transcendentals).
#[inline]
fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x1656_67b1)
        ^ (y as u32).wrapping_mul(0x2787_6a13)
        ^ seed.wrapping_mul(0x85eb_ca77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    (h as f32) * (1.0 / 4_294_967_296.0)
}

/// Quintic smoothstep (Perlin's improved interpolant) — C2-continuous.
#[inline]
fn smooth(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Unit gradient at a lattice corner (angle from the hash).
#[inline]
fn grad(x: i32, y: i32, seed: u32) -> (f32, f32) {
    let a = hash2(x, y, seed) * std::f32::consts::TAU;
    (a.cos(), a.sin())
}

/// 2D gradient (Perlin) noise → remapped to `[0, 1]`.
fn perlin2(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor();
    let yi = y.floor();
    let (xa, ya) = (xi as i32, yi as i32);
    let xf = x - xi;
    let yf = y - yi;
    let u = smooth(xf);
    let v = smooth(yf);
    let dot = |cx: i32, cy: i32, dx: f32, dy: f32| {
        let (gx, gy) = grad(cx, cy, seed);
        gx * dx + gy * dy
    };
    let n00 = dot(xa, ya, xf, yf);
    let n10 = dot(xa + 1, ya, xf - 1.0, yf);
    let n01 = dot(xa, ya + 1, xf, yf - 1.0);
    let n11 = dot(xa + 1, ya + 1, xf - 1.0, yf - 1.0);
    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    let n = nx0 + v * (nx1 - nx0); // ≈ [-0.7, 0.7]
    (n * 0.7143 + 0.5).clamp(0.0, 1.0) // → full [0, 1] contrast
}

/// Fractal Brownian motion — `octaves` of `perlin2` at doubling frequency.
fn fbm(x: f32, y: f32, octaves: u32, persistence: f32, seed: u32) -> f32 {
    let mut total = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for o in 0..octaves.max(1) {
        total += perlin2(x * freq, y * freq, seed.wrapping_add(o)) * amp;
        norm += amp;
        amp *= persistence;
        freq *= 2.0;
    }
    (total / norm).clamp(0.0, 1.0)
}

/// Evaluate a procedural grain by type id at a sample point (already scaled).
/// Returns a coverage multiplier in `[0, 1]`. `seed` varies the pattern.
#[must_use]
pub fn grain_value(gtype: u32, sx: f32, sy: f32, seed: u32) -> f32 {
    match gtype {
        GRAIN_GABOR => {
            // Oriented fibres: a sinusoid along a fixed direction, its phase + the
            // ridge sharpness jittered by low-freq noise (anisotropic streaks).
            let ang = 0.4_f32; // ~23° fibre orientation
            let dir = sx * ang.cos() + sy * ang.sin();
            let jitter = perlin2(sx * 0.5, sy * 0.5, seed) * 2.0;
            (0.5 + 0.5 * (dir * std::f32::consts::TAU + jitter).sin()).clamp(0.0, 1.0)
        }
        GRAIN_PAPER_WEAVE => {
            // Canvas weave: two perpendicular fibre sets, multiplied → cross-hatch.
            let warp = 0.5 + 0.5 * (sx * std::f32::consts::TAU).sin();
            let weft = 0.5 + 0.5 * (sy * std::f32::consts::TAU).sin();
            let n = perlin2(sx, sy, seed) * 0.25;
            ((warp.max(weft) * 0.85 + 0.15) + n - 0.125).clamp(0.0, 1.0)
        }
        GRAIN_SPRAY_DOT => {
            // Per-cell pseudo-random dots: a soft disc whose presence + jitter come
            // from the cell hash (Poisson-ish without the relaxation cost).
            let cx = sx.floor();
            let cy = sy.floor();
            let present = hash2(cx as i32, cy as i32, seed);
            let jx = hash2(cx as i32, cy as i32, seed ^ 0x9e37) - 0.5;
            let jy = hash2(cx as i32, cy as i32, seed ^ 0x79b9) - 0.5;
            let dx = sx - cx - 0.5 - jx;
            let dy = sy - cy - 0.5 - jy;
            let d = (dx * dx + dy * dy).sqrt();
            let dot = (1.0 - (d / 0.35)).clamp(0.0, 1.0); // soft disc, radius ~0.35
            // Dots ADD ink (spray deposits); sparse where `present` is low.
            (dot * present).clamp(0.0, 1.0)
        }
        // GRAIN_SIMPLEX (default): multifractal paper grain.
        _ => fbm(sx, sy, 4, 0.5, seed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_same_input_same_output() {
        for gt in [
            GRAIN_SIMPLEX,
            GRAIN_GABOR,
            GRAIN_PAPER_WEAVE,
            GRAIN_SPRAY_DOT,
        ] {
            let a = grain_value(gt, 3.25, 7.5, 42);
            let b = grain_value(gt, 3.25, 7.5, 42);
            assert_eq!(a, b, "grain {gt} must be deterministic");
        }
    }

    #[test]
    fn output_in_unit_range() {
        for gt in [
            GRAIN_SIMPLEX,
            GRAIN_GABOR,
            GRAIN_PAPER_WEAVE,
            GRAIN_SPRAY_DOT,
        ] {
            for i in 0..200 {
                let x = (i as f32) * 0.37;
                let y = (i as f32) * 0.91 + 1.3;
                let v = grain_value(gt, x, y, 7);
                assert!(
                    (0.0..=1.0).contains(&v),
                    "grain {gt} out of [0,1]: {v} at ({x},{y})"
                );
            }
        }
    }

    #[test]
    fn simplex_has_variation_not_flat() {
        // A real texture varies across space (not a constant).
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;
        for i in 0..256 {
            let x = (i % 16) as f32 * 0.6;
            let y = (i / 16) as f32 * 0.6;
            let v = grain_value(GRAIN_SIMPLEX, x, y, 1);
            min = min.min(v);
            max = max.max(v);
        }
        assert!(
            max - min > 0.3,
            "simplex grain must vary across space: range {min}..{max}"
        );
    }
}
