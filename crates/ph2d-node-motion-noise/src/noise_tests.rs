//! Gates de [`super`] — cortados para o irmao no teto de LOC (HR-18).

/// ⭐⭐ **O KERNEL DA GPU ESPELHA OS PICOS MEDIDOS, e agora há quem o diga.**
///
/// ⛔ O WGSL é uma STRING: os três números de [`CELL_PEAK`] vivem lá como literais
/// (`1.17`, `1.65`, `0.98`), copiados à mão. Mudar a medição só em Rust deixaria a placa
/// a desenhar **outro** campo celular — e a paridade CPU/GPU só reprovaria numa máquina
/// com adapter, em gates `#[ignore]` que o CI nunca corre.
///
/// ⚠️ **Textual de propósito**: corre sem GPU, em todo o lado, e apanha a divergência no
/// instante em que ela é escrita — que é a única altura em que ela é barata. É o irmão
/// do gate que o `force.buoyancy` ganhou pela mesma razão, no mesmo dia.
#[test]
fn the_gpu_kernel_mirrors_the_measured_cellular_peaks() {
    let wgsl = crate::kernel::NS_LIB;
    for (k, name) in [(0_usize, "Euclidiana"), (1, "Manhattan"), (2, "Chebyshev")] {
        let needle = format!("{}", CELL_PEAK[k]);
        assert!(
            wgsl.contains(&needle),
            "o WGSL nao contem `{needle}` -- o pico medido da metrica {name} divergiu \
                 entre o Rust e a copia da GPU, e o campo celular sai diferente nos dois"
        );
    }
}

/// ⭐⭐ **AS TRÊS BASES SÃO TRÊS CAMPOS**, e a métrica é um quarto eixo dentro da
/// celular — senão o param novo não compra nada.
///
/// ⚠️ **O CONTROLO é a base `0` ao bit:** ela tem de ser, literalmente, a função que este
/// nó sempre chamou. Sem essa metade, um gate que só medisse «as bases diferem» passaria
/// mesmo que a base de sempre tivesse mudado de valor — e aí toda arte já produzida por
/// este nó muda em silêncio.
#[test]
fn the_three_bases_are_three_different_fields() {
    let pts: Vec<(f32, f32)> = (0..400)
        .map(|k| (k as f32 * 0.137, (k / 20) as f32 * 0.211))
        .collect();
    let sample = |b: Basis| -> Vec<f32> {
        pts.iter()
            .map(|&(x, y)| base_noise_2d(x, y, 7, b))
            .collect()
    };
    // CONTROLE: a base de sempre, ao bit.
    for &(x, y) in &pts {
        assert_eq!(
            base_noise_2d(x, y, 7, Basis::GRADIENT).to_bits(),
            gradient_noise_2d(x, y, 7).to_bits(),
            "a base 0 tem de ser o ruido de gradiente de sempre, ao BIT"
        );
    }
    let g = sample(Basis::GRADIENT);
    let v = sample(Basis {
        kind: BASE_VALUE,
        metric: 0,
    });
    let c0 = sample(Basis {
        kind: BASE_CELLULAR,
        metric: 0,
    });
    let c1 = sample(Basis {
        kind: BASE_CELLULAR,
        metric: 1,
    });
    let c2 = sample(Basis {
        kind: BASE_CELLULAR,
        metric: 2,
    });
    let apart = |a: &[f32], b: &[f32]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (p - q).abs())
            .fold(0.0_f32, f32::max)
    };
    for (name, other) in [("value", &v), ("cell-euclid", &c0)] {
        assert!(
            apart(&g, other) > 0.3,
            "a base `{name}` desenha o mesmo campo do gradiente ({:.4})",
            apart(&g, other)
        );
    }
    assert!(apart(&v, &c0) > 0.3, "value e celular sao o mesmo campo");
    // ⛔⛔ **A MÉTRICA MEDE-SE NA PRÓPRIA MÉTRICA, e não no campo composto.** Uma
    // mutação que fazia a Manhattan calcular Euclidiana **SOBREVIVEU** a um gate que
    // comparava os campos: as três normalizações são diferentes (`1,17`/`1,65`/`0,98`),
    // então o campo sai diferente **mesmo com a distância ignorada**. *A régua estava a
    // medir a constante que eu escolhi, não a lei que eu queria afirmar.*
    //
    // O oráculo é o ponto onde as três divergem de cor: em `(1, 1)` a Euclidiana dá
    // `√2`, a Manhattan `2` e a Chebyshev `1`.
    assert!((metric_dist(1.0, 1.0, 0) - std::f32::consts::SQRT_2).abs() < 1e-6);
    assert!((metric_dist(1.0, 1.0, 1) - 2.0).abs() < 1e-6);
    assert!((metric_dist(1.0, 1.0, 2) - 1.0).abs() < 1e-6);
    // E o campo composto também difere — a metade que a mutação NÃO matava sozinha.
    assert!(
        apart(&c0, &c1) > 0.1 && apart(&c0, &c2) > 0.1 && apart(&c1, &c2) > 0.1,
        "as tres distancias tinham de dar tres campos: {:.4} {:.4} {:.4}",
        apart(&c0, &c1),
        apart(&c0, &c2),
        apart(&c1, &c2)
    );
}

/// ⭐ **A CELULAR TEM CÉLULAS** — a propriedade que a distingue de «outro ruído».
///
/// ⚠️ **A régua não é «parece diferente», é a FORMA**: num campo celular o valor sobe
/// monotonamente até um máximo local por célula (o ponto-feição) e cai até às fronteiras.
/// O que se mede é que o campo alcança o topo da faixa **muitas vezes** — uma vez por
/// célula —, e o gradiente não: ele tem extremos raros e entre pontos do reticulado.
#[test]
fn the_cellular_base_actually_has_cells() {
    let count_near_top = |b: Basis| {
        let mut n = 0;
        for k in 0..10_000 {
            let x = (k % 100) as f32 * 0.1;
            let y = (k / 100) as f32 * 0.1;
            if base_noise_2d(x, y, 7, b) > 0.9 {
                n += 1;
            }
        }
        n
    };
    let cell = count_near_top(Basis {
        kind: BASE_CELLULAR,
        metric: 0,
    });
    let grad = count_near_top(Basis::GRADIENT);
    assert!(
        cell > grad * 5,
        "a celular tinha de tocar o topo muito mais vezes (uma por celula): \
             {cell} contra {grad}"
    );
}

/// ⭐⭐⭐ **TODA BASE CABE NA FAIXA DECLARADA** — a armadilha que a folha 06 registou.
///
/// ⛔ A `natural_range` do `type` alimenta o `gain_offset_for_range`, que é o que mapeia
/// a saída para o par `min`/`max` do artista. Uma base cuja faixa real transbordasse
/// **partiria esse par em silêncio**: o artista pediria `[0, 1]` e receberia outra coisa,
/// sem erro nenhum. É a mesma forma da armadilha que a onda `Custom` reabriu.
///
/// Medido (4 oitavas, 60×60 células × 4² sub-grid):
///
/// | base | fBm | Ridged |
/// |---|---|---|
/// | gradiente | `[−0,645 · 0,650]` | `[0,237 · 1,000]` |
/// | value | `[−0,932 · 0,901]` | `[0,003 · 0,962]` |
/// | celular (euclid) | `[−0,624 · 0,888]` | `[0,028 · 0,966]` |
///
/// ⚠️ **E o achado do lado: o gradiente é o que MENOS usa a própria faixa** (`65%` a 4
/// oitavas, contra `~90%` das bases novas). Ou seja, com os MESMOS `min`/`max` um campo
/// celular alcança mais da faixa pedida que um Perlin — não é defeito, é o que a média
/// de quatro camadas de gradiente faz, e vale saber antes de alguém chamar «mais forte»
/// a uma base.
#[test]
fn every_base_stays_inside_the_declared_natural_range() {
    for b in [
        Basis::GRADIENT,
        Basis {
            kind: BASE_VALUE,
            metric: 0,
        },
        Basis {
            kind: BASE_CELLULAR,
            metric: 0,
        },
        Basis {
            kind: BASE_CELLULAR,
            metric: 1,
        },
        Basis {
            kind: BASE_CELLULAR,
            metric: 2,
        },
    ] {
        for ty in [NoiseType::Fbm, NoiseType::Turbulence, NoiseType::Ridged] {
            let spec = ph2d_fbm::Spec {
                octaves: 4,
                roughness: 0.5,
                ty,
                ..ph2d_fbm::Spec::default()
            };
            let (lo, hi) = ty.natural_range();
            for k in 0..4000 {
                let x = (k % 80) as f32 * 0.37;
                let y = (k / 80) as f32 * 0.41;
                let v = fbm(x, y, 7, spec, b);
                assert!(
                    v >= lo && v <= hi,
                    "base {b:?} {ty:?}: {v} fora da faixa declarada [{lo},{hi}] -- \
                         o mapeamento min/max deste no' passa a mentir"
                );
            }
        }
    }
}

/// SONDA — **a FAIXA NATURAL de cada base**, que é a armadilha que a folha 06 já
/// registou: ela alimenta o mapeamento `min`/`max` deste nó, e uma base que não a
/// respeite parte esse par **em silêncio**.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_natural_range_per_base() {
    for (name, b) in [
        ("gradiente", Basis::GRADIENT),
        (
            "value",
            Basis {
                kind: BASE_VALUE,
                metric: 0,
            },
        ),
        (
            "cell-euclid",
            Basis {
                kind: BASE_CELLULAR,
                metric: 0,
            },
        ),
        (
            "cell-manhattan",
            Basis {
                kind: BASE_CELLULAR,
                metric: 1,
            },
        ),
        (
            "cell-chebyshev",
            Basis {
                kind: BASE_CELLULAR,
                metric: 2,
            },
        ),
    ] {
        for ty in [NoiseType::Fbm, NoiseType::Turbulence, NoiseType::Ridged] {
            let spec = ph2d_fbm::Spec {
                octaves: 4,
                roughness: 0.5,
                ty,
                ..ph2d_fbm::Spec::default()
            };
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for cy in 0..60 {
                for cx in 0..60 {
                    for k in 0..16 {
                        let x = cx as f32 + (k % 4) as f32 / 4.0;
                        let y = cy as f32 + (k / 4) as f32 / 4.0;
                        let v = fbm(x, y, 7, spec, b);
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
            }
            let want = ty.natural_range();
            println!("{name:15} {ty:?}: medido [{lo:7.4},{hi:7.4}]  ·  declarado {want:?}");
        }
    }
}

/// SONDA — **o pico do F1 celular, por métrica.** A normalização de [`CELL_PEAK`] sai
/// daqui, e não de uma conta de cabeça: a «bola» unitária tem forma diferente em cada
/// métrica, logo a maior distância que a busca 3×3 alcança também é diferente.
///
/// ⚠️ **Varre por CÉLULA e não pelo campo inteiro**: o extremo de um ruído de reticulado
/// vive DENTRO de uma célula, e uma varredura global grossa passa ao lado dele.
#[test]
#[ignore = "sonda de medicao, nao gate"]
fn measure_the_cellular_peak() {
    for metric in 0..3 {
        for seed in [7, 101, 4242] {
            let mut peak = 0.0_f32;
            for cy in 0..40 {
                for cx in 0..40 {
                    for sy in 0..24 {
                        for sx in 0..24 {
                            let x = cx as f32 + sx as f32 / 24.0;
                            let y = cy as f32 + sy as f32 / 24.0;
                            peak = peak.max(cellular_f1(x, y, seed, metric));
                        }
                    }
                }
            }
            println!("metrica {metric} seed {seed}: F1 maximo {peak:.4}");
        }
    }
}
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
            Basis::GRADIENT,
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
                },
                Basis::GRADIENT
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
                },
                Basis::GRADIENT
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
            Basis::GRADIENT,
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
            Basis::GRADIENT,
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
                },
                Basis::GRADIENT
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
