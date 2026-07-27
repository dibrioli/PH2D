//! 2D value noise + fBm for `value.noise` — an integer hash on a lattice + a
//! smootherstep fade + bilinear lerp, summed over octaves. A **leaf-local mirror**
//! of `motion.wiggle`'s `noise.rs` (the same `hash2`/`fade`/`value_noise_2d`,
//! copied per drop-crate — the shared vocabulary is the *behaviour*, not a shared
//! symbol), so `value.noise` and `motion.wiggle` sample the SAME field. **Fully
//! deterministic / transcendental-free** (HR-5): only integer ops, IEEE `floor`,
//! polynomials, and correctly-rounded `u32→f32` division.

/// Hard ceiling on octaves — an untrusted `f32` param drives the fBm loop count.
/// 8 is past the point of visible return (each octave halves the feature size);
/// the WGSL loop guards on the same bound.
pub(crate) const MAX_OCTAVES: u32 = 8;
/// Lacunarity — the frequency multiplier per octave. Fixed at 2 (the near-
/// universal standard: Blender/Houdini/Cavalry all default here). Exposing it is
/// a rare tweak that would widen the param surface for little gain; named in
/// doc 69 as a deliberate limit, not an oversight.
const LACUNARITY: f32 = 2.0;
/// Below this the octave-weight sum is treated as zero (roughness 0 with 1 octave
/// still sums 1, so this only guards a pathological all-zero weight) — the fBm
/// never divides by (near-)zero into `NaN`.
const MIN_NORM: f32 = 1e-6;

/// A well-mixed integer hash of a lattice cell → a pseudo-random `f32 ∈ [-1, 1)`.
/// Adjacent cells hash to uncorrelated values (no visible grid pattern). Byte-
/// identical to `motion.wiggle`'s `hash2` (and its WGSL twin `vn_hash2`).
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
fn value_noise_2d(x: f32, y: f32) -> f32 {
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

/// Fractal (fBm) value noise: sum `octaves` layers of [`value_noise_2d`], each at
/// double the frequency (lacunarity 2) and `roughness×` the amplitude of the
/// last, then **normalize by the weight sum** so the output stays in `[-1, 1]`
/// regardless of octave count (the CHOP/wiggle convention — the range does not
/// grow when you add detail). `octaves` is clamped to `1..=MAX_OCTAVES`.
///
/// At `octaves == 1` this is exactly `value_noise_2d(x, y)` (one layer, weight 1),
/// so a single-octave `value.noise` samples the *same* field as `motion.wiggle`.
pub(crate) fn fbm_2d(x: f32, y: f32, octaves: u32, roughness: f32) -> f32 {
    let oct = octaves.clamp(1, MAX_OCTAVES);
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut freq = 1.0;
    let mut norm = 0.0;
    for _ in 0..oct {
        sum += amp * value_noise_2d(x * freq, y * freq);
        norm += amp;
        amp *= roughness;
        freq *= LACUNARITY;
    }
    sum / norm.max(MIN_NORM)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_octave_fbm_is_the_bare_value_noise() {
        // The fBm at octaves=1 is a single layer of weight 1 — the SAME field a
        // single `motion.wiggle` sample reads. Any roughness is irrelevant (it
        // only scales layers 2..): one octave, one answer.
        for k in 0..200 {
            let (x, y) = (k as f32 * 0.137, k as f32 * 0.311);
            assert_eq!(fbm_2d(x, y, 1, 0.5), value_noise_2d(x, y));
            assert_eq!(fbm_2d(x, y, 1, 0.9), value_noise_2d(x, y));
        }
    }

    #[test]
    fn fbm_is_bounded_and_deterministic() {
        for k in 0..500 {
            let (x, y) = (k as f32 * 0.137, k as f32 * 0.071);
            for oct in 1..=MAX_OCTAVES {
                let n = fbm_2d(x, y, oct, 0.5);
                assert!((-1.0..=1.0).contains(&n), "fbm {n} out of range oct {oct}");
                assert_eq!(fbm_2d(x, y, oct, 0.5), n, "deterministic");
            }
        }
    }

    #[test]
    fn octaves_are_clamped_never_diverge() {
        // A wild octave param (0, or huge) is clamped into `1..=MAX_OCTAVES`, so
        // the loop count is bounded and the output stays in range.
        let n0 = fbm_2d(3.5, 1.2, 0, 0.5);
        assert!((-1.0..=1.0).contains(&n0));
        assert_eq!(n0, fbm_2d(3.5, 1.2, 1, 0.5), "0 octaves clamps up to 1");
        let big = fbm_2d(3.5, 1.2, 999, 0.5);
        assert!((-1.0..=1.0).contains(&big));
        assert_eq!(
            big,
            fbm_2d(3.5, 1.2, MAX_OCTAVES, 0.5),
            "clamps down to MAX"
        );
    }

    #[test]
    fn adjacent_instances_decorrelate_and_zero_roughness_is_one_octave() {
        // Two rows one lattice unit apart sample different cells → different value.
        assert_ne!(value_noise_2d(3.5, 0.0), value_noise_2d(3.5, 1.0));
        // Roughness 0 zeroes every layer past the first → identical to 1 octave.
        assert_eq!(fbm_2d(2.5, 4.0, 4, 0.0), value_noise_2d(2.5, 4.0));
    }
}
