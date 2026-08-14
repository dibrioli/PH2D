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

/// Normalization so the raw gradient noise lands in `[-1, 1]` **com 0,74% de
/// folga** — e as duas frases que estavam aqui eram falsas, medidas em 2026-08-13.
///
/// Diziam *"o pico empírico desta construção é ~1,49, então `1/1.5` mapeia-o com
/// segurança dentro de `[-1,1]` (o golden test pina o limite verdadeiro)"*.
/// Varrido por CÉLULA (180×180 células × sub-grid 24², a sonda
/// `measure_the_true_perlin_peak` do `value.noise`, que shipa este mesmo kernel)
/// o pico cru é **1,5111**, logo a saída de UMA oitava chega a **1,0074**.
///
/// ⚠️ E o "golden test" é o [`fbm_is_bounded_deterministic_and_uses_the_full_range`]
/// abaixo, que roda com **4 oitavas**: a média de quatro camadas não alcança o
/// pico de uma, então a fixture dele **não contém o fenómeno** que ele afirma
/// pinar. É por isso que a folga viveu com a suíte verde.
///
/// ⚠️ **A constante NÃO foi mexida, de propósito** — apertá-la mudaria em 1,3% o
/// desenho de toda arte que este nó já produziu, por uma violação de limite que
/// ninguém reportou. O que se corrige aqui é a AFIRMAÇÃO; apertar (ou não) é
/// decisão do dono deste nó, com o número agora na mão.
///
/// A single constant multiply — HR-5 clean.
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

/// ⚠️ **A LEI da soma fractal MUDOU-SE para a folha [`ph2d_fbm`]** — ela tinha
/// DUAS implementações neste repo (esta e a do `force.curl`) e elas já
/// divergiam; a família de forças ia ganhar a terceira ao herdar o cluster de
/// params. O que fica aqui é o **ruído de base**, que é de GRADIENTE de
/// propósito e não se compartilha (ver o topo do módulo).
///
/// ⚠️ E o `LACUNARITY` que morava aqui era uma **cerca**: *"expô-lo é um knob
/// avançado raramente tocado que toda ferramenta defaulta em 2, então fica
/// interno"*. A premissa é verdade e a conclusão não segue — Cavalry
/// (`Octaves, Lacunarity, Gain`), VFXG (`Octaves · Roughness · Lacunarity`) e
/// Houdini **expõem** e defaultam em 2. É um param (doc 89 folha 02), e o
/// default preserva o mundo de antes AO BIT.
pub(crate) use ph2d_fbm::NoiseType;

/// Uma oitava do ruído de base deste nó, com o seed deslocado pelo índice dela.
///
/// ⚠️ **O deslocamento por oitava é DESTE nó, não da lei** — sem ele as oitavas
/// são cópias escaladas de um campo só e batem visivelmente; com ele, dentro da
/// lei, o `force.curl` (que não o faz) mudaria de aparência. Por isso a folha
/// entrega o índice e não tem opinião.
pub(crate) fn octave(x: f32, y: f32, o: u32, seed: i32) -> f32 {
    gradient_noise_2d(x, y, seed.wrapping_add(o as i32 * 1013))
}

/// **A composição deste nó**: a lei da folha sobre o ruído de gradiente dele.
/// Uma porta, dois chamadores — o `eval` e os gates —, senão o que os gates
/// medem deixa de ser o que o produto faz.
pub(crate) fn fbm(x: f32, y: f32, seed: i32, spec: ph2d_fbm::Spec) -> f32 {
    ph2d_fbm::eval(spec, x, y, |px, py, o| octave(px, py, o, seed))
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
            let n = fbm(
                x,
                y,
                7,
                ph2d_fbm::Spec {
                    octaves: 4,
                    roughness: 0.5,
                    ty: NoiseType::Fbm,
                    ..ph2d_fbm::Spec::default()
                },
            );
            assert!((-1.0..=1.0).contains(&n), "fbm {n} out of range at {k}");
            assert_eq!(
                fbm(
                    x,
                    y,
                    7,
                    ph2d_fbm::Spec {
                        octaves: 4,
                        roughness: 0.5,
                        ty: NoiseType::Fbm,
                        ..ph2d_fbm::Spec::default()
                    }
                ),
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
                fbm(
                    x,
                    y,
                    3,
                    ph2d_fbm::Spec {
                        octaves: 1,
                        roughness: 0.5,
                        ty: NoiseType::Fbm,
                        ..ph2d_fbm::Spec::default()
                    }
                ),
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
            let t = fbm(
                x,
                y,
                5,
                ph2d_fbm::Spec {
                    octaves: 4,
                    roughness: 0.5,
                    ty: NoiseType::Turbulence,
                    ..ph2d_fbm::Spec::default()
                },
            );
            let r = fbm(
                x,
                y,
                5,
                ph2d_fbm::Spec {
                    octaves: 4,
                    roughness: 0.5,
                    ty: NoiseType::Ridged,
                    ..ph2d_fbm::Spec::default()
                },
            );
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
                fbm(
                    x,
                    y,
                    2,
                    ph2d_fbm::Spec {
                        octaves: 5,
                        roughness: 0.0,
                        ty: NoiseType::Fbm,
                        ..ph2d_fbm::Spec::default()
                    }
                ),
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
