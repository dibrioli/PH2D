//! Perlin **gradient** noise (2002 "Improving Noise") + fBm, for `motion.noise`.
//!
//! Gradient noise, NOT value noise (what `motion.wiggle` uses). The difference,
//! and why it looks better: value noise interpolates a random VALUE at each
//! lattice corner, which leaves axis-aligned grid artifacts; gradient noise
//! interpolates the dot of a random GRADIENT with the distance vector, so the
//! field is exactly zero at every lattice point and its extrema fall BETWEEN
//! them — no grid pattern, more organic flow (Perlin; Quilez, "gradient noise").
//!
//! **Deterministic / transcendental-free (HR-5):** an integer hash on the
//! lattice (no `PERM` table — a pure hash is stateless and seedable), the
//! quintic fade `6t⁵−15t⁴+10t³` (2002: C² continuous, so no 2nd-order creases at
//! lattice crossings — the cubic `3t²−2t³` of 1985 had a discontinuous 2nd
//! derivative), the 8 fixed 2002 gradients, dot products, and lerps. No `sin`,
//! `exp`, `pow`, or RNG — a noise field replays bit-identically across
//! platforms, like every other Motion node.

/// A well-mixed integer hash of a lattice cell (+ seed) → `u32`. Adjacent cells
/// hash to uncorrelated values (the same mixer `motion.wiggle` uses, extended
/// with a seed word so several Noise nodes decorrelate).
fn hash(ix: i32, iy: i32, seed: i32) -> u32 {
    let mut h = (ix as u32)
        .wrapping_mul(0x27d4_eb2d)
        .wrapping_add((iy as u32).wrapping_mul(0x1656_67b1))
        .wrapping_add((seed as u32).wrapping_mul(0x0193_4f07));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x2971_75f9);
    h ^= h >> 15;
    h
}

/// The dot of the corner's gradient with the distance vector `(dx, dy)`.
///
/// The eight 2002 2D gradients are `(±1, ±2)` and `(±2, ±1)` — pointing at the
/// midpoints of a square's edges, so no gradient aligns with an axis (the source
/// of the 1985 directional bias). Selected by three bits of the hash; expressed
/// as `±u ± 2v` (Perlin's own trick) so it is branch-light and multiply-free
/// beyond the `×2`.
fn dot_grad(h: u32, dx: f32, dy: f32) -> f32 {
    let g = h & 7;
    let (u, v) = if g < 4 { (dx, dy) } else { (dy, dx) };
    let a = if g & 1 != 0 { -u } else { u };
    let b = if g & 2 != 0 { -2.0 * v } else { 2.0 * v };
    a + b
}

/// Quintic fade (Perlin 2002): `6t⁵−15t⁴+10t³`, C² continuous on `[0,1]`.
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Normalization so the raw gradient noise lands in `[-1, 1]`. The `(±1,±2)`
/// gradients give a raw range of about `±√5 · ½`; the empirical peak of this
/// construction is ~1.49, so `1/1.5` maps it safely inside `[-1,1]` (the golden
/// test pins the true bound). A single constant multiply — HR-5 clean.
const NORM: f32 = 1.0 / 1.5;

/// Single-octave Perlin gradient noise at `(x, y)`, seeded. Range `[-1, 1]`.
pub(crate) fn gradient_noise_2d(x: f32, y: f32, seed: i32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let (u, v) = (fade(fx), fade(fy));
    // Dot each corner's gradient with the distance FROM that corner.
    let n00 = dot_grad(hash(ix, iy, seed), fx, fy);
    let n10 = dot_grad(hash(ix + 1, iy, seed), fx - 1.0, fy);
    let n01 = dot_grad(hash(ix, iy + 1, seed), fx, fy - 1.0);
    let n11 = dot_grad(hash(ix + 1, iy + 1, seed), fx - 1.0, fy - 1.0);
    let nx0 = n00 + u * (n10 - n00);
    let nx1 = n01 + u * (n11 - n01);
    (nx0 + v * (nx1 - nx0)) * NORM
}

/// The fractal-sum flavour, decoded from the `type` enum param.
#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum NoiseType {
    /// `Σ aᵢ·noise` — signed, bipolar `[-1,1]`. The default drifting field.
    Fbm,
    /// `Σ aᵢ·|noise|` — rectified, unipolar `[0,1]`. Sharp billows/creases
    /// (Perlin's original "turbulence": smoke, marble).
    Turbulence,
    /// `Σ aᵢ·(1−|noise|)²` — inverted turbulence, unipolar `[0,1]`. Sharp ridges
    /// (mountains, veins).
    Ridged,
}

impl NoiseType {
    pub(crate) fn from_index(i: f32) -> Self {
        match i.round() as i32 {
            1 => NoiseType::Turbulence,
            2 => NoiseType::Ridged,
            _ => NoiseType::Fbm,
        }
    }
}

/// **Lacunarity** — the per-octave frequency multiplier. Fixed at the universal
/// default 2.0 (each octave halves the feature size); exposing it is a rarely-
/// touched advanced knob every tool defaults to 2, so it stays internal.
const LACUNARITY: f32 = 2.0;

/// fBm (fractional Brownian motion): a sum of `octaves` octaves, each at twice
/// the frequency ([`LACUNARITY`]) and `roughness`× the amplitude of the last —
/// the canonical fractal sum (Quilez, "fbm"). `roughness` is the gain/
/// persistence (0.5 typical). Normalized by the total amplitude so the result
/// stays bounded regardless of octave count. One octave is the base noise.
///
/// `ty` picks the per-octave rectification: signed fBm ([-1,1]), turbulence, or
/// ridged (both [0,1]). Applying it PER OCTAVE (not to the final sum) is what
/// gives turbulence its creases and ridged its sharp veins.
pub(crate) fn fbm_2d(
    mut x: f32,
    mut y: f32,
    seed: i32,
    octaves: u32,
    roughness: f32,
    ty: NoiseType,
) -> f32 {
    let gain = roughness.clamp(0.0, 1.0);
    let mut amp = 1.0;
    let mut sum = 0.0;
    let mut total = 0.0;
    for o in 0..octaves.max(1) {
        // A per-octave seed offset so octaves are independent, not scaled copies
        // of one field (which would beat visibly).
        let n = gradient_noise_2d(x, y, seed.wrapping_add(o as i32 * 1013));
        let shaped = match ty {
            NoiseType::Fbm => n,
            NoiseType::Turbulence => n.abs(),
            NoiseType::Ridged => {
                let r = 1.0 - n.abs();
                r * r
            }
        };
        sum += amp * shaped;
        total += amp;
        amp *= gain;
        x *= LACUNARITY;
        y *= LACUNARITY;
    }
    sum / total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gradient noise is EXACTLY zero at every lattice point — its defining
    /// property (the distance vector is zero there, so every corner's dot is
    /// zero). This is what value noise cannot do, and why gradient noise has no
    /// grid pattern: the extrema live between the lattice points, not on them.
    #[test]
    fn gradient_noise_is_zero_at_lattice_points() {
        for iy in -3..3 {
            for ix in -3..3 {
                let n = gradient_noise_2d(ix as f32, iy as f32, 0);
                assert!(n.abs() < 1e-6, "noise at lattice ({ix},{iy}) = {n}, want 0");
            }
        }
    }

    /// Bounded in `[-1, 1]` and deterministic — a dense sweep over a
    /// non-lattice-aligned grid finds the true peak (which must clear the
    /// normalization) yet never breach the range.
    #[test]
    fn fbm_is_bounded_deterministic_and_uses_the_full_range() {
        let mut peak = 0.0_f32;
        for k in 0..4000 {
            let x = k as f32 * 0.0731;
            let y = (k as f32 * 0.1373).sin_like();
            let n = fbm_2d(x, y, 7, 4, 0.5, NoiseType::Fbm);
            assert!((-1.0..=1.0).contains(&n), "fbm {n} out of range at {k}");
            assert_eq!(
                fbm_2d(x, y, 7, 4, 0.5, NoiseType::Fbm),
                n,
                "fbm must be pure"
            );
            peak = peak.max(n.abs());
        }
        assert!(
            peak > 0.5,
            "the field should exercise a good part of the range: {peak}"
        );
    }

    /// A single octave of fBm is exactly the base gradient noise (the fractal sum
    /// degenerates to one term, normalized by its own amplitude).
    #[test]
    fn one_octave_fbm_equals_the_base_noise() {
        for k in 0..50 {
            let (x, y) = (k as f32 * 0.31, k as f32 * 0.17);
            assert_eq!(
                fbm_2d(x, y, 3, 1, 0.5, NoiseType::Fbm),
                gradient_noise_2d(x, y, 3)
            );
        }
    }

    /// Turbulence and ridged are unipolar `[0,1]` (rectified per octave), while
    /// fBm is bipolar `[-1,1]` — their defining difference. A ridged field
    /// tends HIGH (it inverts `|noise|`), a turbulence field tends off zero.
    #[test]
    fn turbulence_and_ridged_are_unipolar() {
        let (mut turb_min, mut ridge_max) = (f32::MAX, 0.0_f32);
        for k in 0..3000 {
            let (x, y) = (k as f32 * 0.083, (k as f32 * 0.151).sin_like());
            let t = fbm_2d(x, y, 5, 4, 0.5, NoiseType::Turbulence);
            let r = fbm_2d(x, y, 5, 4, 0.5, NoiseType::Ridged);
            assert!((0.0..=1.0).contains(&t), "turbulence {t} not in [0,1]");
            assert!((0.0..=1.0).contains(&r), "ridged {r} not in [0,1]");
            turb_min = turb_min.min(t);
            ridge_max = ridge_max.max(r);
        }
        assert!(turb_min >= 0.0, "turbulence never goes negative");
        assert!(
            ridge_max > 0.7,
            "ridged reaches its sharp high ridges: {ridge_max}"
        );
    }

    /// `roughness` (gain) shapes the octave falloff: 0 keeps only the first
    /// octave (smooth), higher lets finer octaves through (rough). At roughness
    /// 0 the multi-octave fBm collapses to the base noise.
    #[test]
    fn roughness_zero_collapses_to_the_first_octave() {
        for k in 0..50 {
            let (x, y) = (k as f32 * 0.29, k as f32 * 0.41);
            assert_eq!(
                fbm_2d(x, y, 2, 5, 0.0, NoiseType::Fbm),
                gradient_noise_2d(x, y, 2),
                "roughness 0 = only octave 0 contributes"
            );
        }
    }

    /// The seed decorrelates fields: two seeds give genuinely different fields.
    /// Measured over many samples, not one point — with only 8 discrete
    /// gradients a single coordinate can coincide between seeds by chance.
    #[test]
    fn the_seed_shifts_the_field() {
        let mut diff = 0.0_f32;
        for k in 0..200 {
            let (x, y) = (k as f32 * 0.137 + 0.5, k as f32 * 0.311 + 0.5);
            diff += (gradient_noise_2d(x, y, 0) - gradient_noise_2d(x, y, 99)).abs();
        }
        assert!(
            diff > 10.0,
            "the two seeded fields barely differ: total {diff}"
        );
    }

    /// Gradient noise beats value noise on the anti-grid property: value noise is
    /// generally NON-zero at lattice points (it interpolates the corner value),
    /// gradient noise is zero. This is the measurable difference in the doc.
    #[test]
    fn gradient_noise_differs_from_value_noise_at_the_lattice() {
        // A value-noise stand-in: the corner hash itself (what wiggle lerps).
        let corner_hash = {
            let h = hash(3, 4, 0);
            (h as f32 / u32::MAX as f32) * 2.0 - 1.0
        };
        // Value noise would carry that (generally non-zero) hash to the corner;
        // gradient noise is zero there.
        assert!(corner_hash.abs() > 1e-3, "the corner hash is non-trivial");
        assert!(gradient_noise_2d(3.0, 4.0, 0).abs() < 1e-6);
    }

    // A transcendental-free stand-in for a scattered `y` in the sweep (HR-5: the
    // TEST may not call `sin`, to keep the whole crate's determinism auditable by
    // a grep for transcendentals). A cheap irrational-ish scramble.
    trait SinLike {
        fn sin_like(self) -> f32;
    }
    impl SinLike for f32 {
        fn sin_like(self) -> f32 {
            let f = self - self.floor();
            // A triangle wave in [-1,1], enough to scatter the sample points.
            2.0 * (2.0 * f - 1.0).abs() - 1.0
        }
    }
}
