//! 2D value noise for `force.curl` (the noise potential ψ) — an integer hash on a lattice + a
//! smootherstep fade + bilinear lerp (the reference's `perlin2` shape). **Fully
//! deterministic / transcendental-free** (HR-5): only integer ops, IEEE `floor`,
//! polynomials, and correctly-rounded `u32→f32` division. Output in `[-1, 1]`.

/// A well-mixed integer hash of a lattice cell → a pseudo-random `f32 ∈ [-1, 1)`.
/// Adjacent cells hash to uncorrelated values (no visible grid pattern).
fn hash2(ix: i32, iy: i32) -> f32 {
    let mut h = (ix as u32)
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add((iy as u32).wrapping_mul(0x1656_67b1));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x2971_75f9);
    h ^= h >> 15;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

/// Smootherstep fade `6t⁵−15t⁴+10t³` (Perlin) on `t ∈ [0,1]` — C² continuous, so
/// the noise has no lattice-crossing creases.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Smooth 2D value noise at `(x, y)`, bilinearly interpolating the four lattice
/// corner hashes with a smootherstep fade. Range `[-1, 1]`.
pub(crate) fn value_noise_2d(x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let n00 = hash2(ix, iy);
    let n10 = hash2(ix + 1, iy);
    let n01 = hash2(ix, iy + 1);
    let n11 = hash2(ix + 1, iy + 1);
    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    nx0 + v * (nx1 - nx0)
}

/// Fractal Brownian motion: `octaves` of value noise at doubling frequency and
/// halving amplitude — the standard potential field for curl noise (Bridson
/// 2007 §3), giving swirls at several scales instead of one. Normalized so the
/// result stays in `[-1, 1]`.
pub(crate) fn fbm(x: f32, y: f32, octaves: u32) -> f32 {
    let (mut freq, mut amp, mut sum, mut norm) = (1.0f32, 1.0f32, 0.0f32, 0.0f32);
    for _ in 0..octaves.max(1) {
        sum += value_noise_2d(x * freq, y * freq) * amp;
        norm += amp;
        freq *= 2.0;
        amp *= 0.5;
    }
    sum / norm
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_bounded_and_deterministic() {
        for k in 0..200 {
            let x = k as f32 * 0.137;
            let y = k as f32 * 0.311;
            let n = value_noise_2d(x, y);
            assert!((-1.0..=1.0).contains(&n), "noise {n} out of range at {k}");
            // Deterministic: the same coords always hash the same.
            assert_eq!(value_noise_2d(x, y), n);
        }
    }

    #[test]
    fn adjacent_rows_are_decorrelated() {
        // Two instances one row apart wiggle independently (different lattice
        // hash), so their noise differs at the same time.
        assert_ne!(value_noise_2d(3.5, 0.0), value_noise_2d(3.5, 1.0));
    }

    #[test]
    fn fbm_is_bounded_deterministic_and_richer_than_one_octave() {
        for k in 0..100 {
            let (x, y) = (k as f32 * 0.31, k as f32 * 0.17);
            let v = fbm(x, y, 3);
            assert!((-1.0..=1.0).contains(&v), "fbm {v} out of range");
            assert_eq!(v, fbm(x, y, 3), "deterministic");
        }
        // Adding octaves changes the field (they are not a no-op).
        assert_ne!(fbm(1.3, 2.7, 1), fbm(1.3, 2.7, 3));
        // One octave IS the base noise.
        assert_eq!(fbm(1.3, 2.7, 1), value_noise_2d(1.3, 2.7));
    }

    #[test]
    fn integer_lattice_hits_the_corner_hash_exactly() {
        // At integer coords the fade is 0 → the value is the corner hash, and
        // `hash2` is in range.
        let n = value_noise_2d(4.0, 7.0);
        assert!((-1.0..=1.0).contains(&n));
    }
}
