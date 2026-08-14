//! Gates do catálogo de kernels + a adopção da lei fractal.
//!
//! A wave tem duas metades que se provam de maneiras diferentes: o **default é o
//! mundo anterior AO BIT** (oráculo congelado) e cada kernel novo **desenha outra
//! coisa** (propriedades medidas, nunca um literal escolhido).

use super::*;

/// A lei fractal **COMO ELA SHIPAVA**, congelada sob `cfg(test)` — o oráculo da
/// adopção da folha [`ph2d_fbm`].
///
/// ⚠️ Um `pub` sem chamador seria uma **segunda resposta** à espera de alguém a
/// chamar; congelada aqui, ela só pode ser o que é: o código de antes, verbatim,
/// para o de agora ser comparado contra ele.
fn fbm_2d_frozen(x: f32, y: f32, octaves: u32, roughness: f32) -> f32 {
    const MIN_NORM: f32 = 1e-6;
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

fn val(x: f32, y: f32) -> f32 {
    value_noise_2d(x, y)
}

/// **A adopção da folha é BYTE-IDÊNTICA na faixa que shipa.**
///
/// A lei tinha uma quarta cópia neste repo e passou a vir do [`ph2d_fbm`]. As
/// duas formas coincidem *exactamente*, e o porquê importa: a folha escala a
/// coordenada por multiplicação repetida (`px *= 2`) e a cópia daqui escalava a
/// frequência (`x * freq`, `freq *= 2`) — com lacunarity **2** as duas são a
/// mesma potência de dois e o IEEE-754 não arredonda nenhuma.
///
/// FALSIFICADO por qualquer mudança de ordem, de normalização, ou uma lacunarity
/// que deixe de ser potência de dois.
#[test]
fn adopting_the_fractal_law_leaf_is_byte_identical() {
    for k in 0..400 {
        let (x, y) = (k as f32 * 0.137 - 20.0, k as f32 * 0.071 - 9.0);
        for oct in 1..=MAX_OCTAVES {
            for r in [0.0f32, 0.25, 0.5, 0.9, 1.0] {
                assert_eq!(
                    fbm_2d(x, y, oct, r, val).to_bits(),
                    fbm_2d_frozen(x, y, oct, r).to_bits(),
                    "oct {oct} rough {r} at ({x}, {y})"
                );
            }
        }
    }
}

/// **O tecto de oitavas é DESTE nó, não da folha.** O [`ph2d_fbm::eval`] faz
/// `octaves.max(1)` e não tem tecto — um param `f32` não confiável dirigiria o
/// laço. FALSIFICADO por uma chamada nua à folha.
#[test]
fn a_wild_octave_param_cannot_unbound_the_loop() {
    let big = fbm_2d(3.5, 1.2, 999, 0.5, val);
    assert_eq!(big, fbm_2d(3.5, 1.2, MAX_OCTAVES, 0.5, val), "clamps down");
    let zero = fbm_2d(3.5, 1.2, 0, 0.5, val);
    assert_eq!(zero, fbm_2d(3.5, 1.2, 1, 0.5, val), "0 octaves clamps up");
}

#[test]
fn one_octave_fbm_is_the_bare_value_noise() {
    // The fBm at octaves=1 is a single layer of weight 1 — the SAME field a
    // single `motion.wiggle` sample reads.
    for k in 0..200 {
        let (x, y) = (k as f32 * 0.137, k as f32 * 0.311);
        assert_eq!(fbm_2d(x, y, 1, 0.5, val), value_noise_2d(x, y));
        assert_eq!(fbm_2d(x, y, 1, 0.9, val), value_noise_2d(x, y));
    }
}

/// **Os três kernels ficam na faixa nominal `[-1, 1]` e são determinísticos.** É
/// o que torna `amplitude` uma escala e não uma surpresa: trocar de kernel não
/// muda a ordem de grandeza do que sai, muda o DESENHO.
///
/// ⚠️ **A varredura é por CÉLULA e o limite do Perlin é 1,008, não 1,0** — as
/// duas coisas andam juntas. O gate irmão do `motion.noise` afirma `[-1, 1]`
/// exacto sobre uma fBm de **4 oitavas**, e a média de quatro camadas não
/// alcança o pico de UMA: a fixture dele **não contém o fenómeno**, e é por isso
/// que a folga de 0,74% viveu com a suíte verde. Aqui a varredura é
/// single-octave, sobre células suficientes para o pico aparecer, e o número que
/// se afirma é o que a sonda mede.
#[test]
fn every_kernel_is_bounded_and_deterministic() {
    for kernel in [Kernel::Value, Kernel::Perlin, Kernel::Cellular] {
        for feature in [CellFeature::Cells, CellFeature::Cracks] {
            let f = |x: f32, y: f32| base(kernel, feature, 1.0, x, y);
            // Value e Cellular são limitados POR CONSTRUÇÃO (combinação convexa
            // de hashes em `[-1,1)` · um `clamp` explícito); o Perlin é medido.
            let bound = if kernel == Kernel::Perlin {
                PERLIN_PEAK
            } else {
                1.0
            };
            let mut worst = 0.0f32;
            for cy in -30..30 {
                for cx in -30..30 {
                    for sub in 0..9 {
                        let x = cx as f32 + (sub % 3) as f32 / 3.0;
                        let y = cy as f32 + (sub / 3) as f32 / 3.0;
                        for oct in [1u32, MAX_OCTAVES] {
                            let n = fbm_2d(x, y, oct, 0.5, f);
                            worst = worst.max(n.abs());
                            assert!(
                                n.abs() <= bound,
                                "{kernel:?}/{feature:?} out of range: {n} (oct {oct})"
                            );
                            assert_eq!(fbm_2d(x, y, oct, 0.5, f), n, "deterministic");
                        }
                    }
                }
            }
            // CONTROLE: o campo usa a faixa: uma varredura que só encontrasse
            // valores pequenos passaria no limite sem provar nada.
            assert!(
                worst > 0.7,
                "{kernel:?}/{feature:?} never approached the range: {worst}"
            );
        }
    }
}

#[test]
fn adjacent_instances_decorrelate_and_zero_roughness_is_one_octave() {
    assert_ne!(value_noise_2d(3.5, 0.0), value_noise_2d(3.5, 1.0));
    assert_eq!(fbm_2d(2.5, 4.0, 4, 0.0, val), value_noise_2d(2.5, 4.0));
}

/// **O Perlin não tem o padrão de grade que o Value tem** — é o que o kernel
/// novo COMPRA, e é uma propriedade medida, não um adjetivo.
///
/// O ruído de VALOR põe um extremo em cada ponto da grade (o valor da célula
/// *é* a hash), então amostrado exactamente nos inteiros ele devolve a hash
/// crua e a variância ali é a variância cheia do campo. O de GRADIENTE é
/// **exactamente zero** em cada ponto da grade — os extremos caem ENTRE eles.
///
/// FALSIFICADO por um `perlin_2d` que devolva o valor da quina (isto é, pelo
/// kernel de valor com outro nome).
#[test]
fn the_gradient_kernel_is_zero_on_the_lattice_and_the_value_kernel_is_not() {
    let mut worst_perlin = 0.0f32;
    let mut value_energy = 0.0f32;
    for iy in -8..8 {
        for ix in -8..8 {
            let (x, y) = (ix as f32, iy as f32);
            worst_perlin = worst_perlin.max(perlin_2d(x, y).abs());
            value_energy += value_noise_2d(x, y).abs();
        }
    }
    let value_mean = value_energy / 256.0;
    assert!(
        worst_perlin < 1e-6,
        "gradient noise must vanish on the lattice, worst |n| = {worst_perlin:e}"
    );
    assert!(
        value_mean > 0.3,
        "control — value noise carries its full amplitude on the lattice \
         (mean |n| = {value_mean})"
    );
}

/// **O celular desenha CÉLULAS: o campo cai perto de uma semente e sobe longe
/// dela.** A propriedade que separa Worley dos dois irmãos é ter um MÍNIMO
/// isolado por célula, e é isso que se afirma — não um número de pico.
///
/// FALSIFICADO por um kernel que devolva a distância ao ponto da grade (jitter
/// ignorado), porque então o mínimo estaria sempre no centro exacto da célula.
#[test]
fn the_cellular_kernel_puts_one_minimum_per_cell_and_the_jitter_moves_it() {
    // Onde está o mínimo dentro da célula (0,0)? Amostrado num 64×64.
    let argmin = |jitter: f32| {
        let (mut best, mut at) = (f32::INFINITY, (0.0f32, 0.0f32));
        for gy in 0..64 {
            for gx in 0..64 {
                let (x, y) = (gx as f32 / 64.0, gy as f32 / 64.0);
                let v = cellular_2d(x, y, CellFeature::Cells, jitter);
                if v < best {
                    best = v;
                    at = (x, y);
                }
            }
        }
        (best, at)
    };
    let (v0, at0) = argmin(0.0);
    let (v1, at1) = argmin(1.0);
    // Jitter 0 põe a semente no CENTRO exacto da célula — a grade regular.
    assert!(
        (at0.0 - 0.5).abs() < 0.02 && (at0.1 - 0.5).abs() < 0.02,
        "jitter 0 must seed the cell centre, got {at0:?}"
    );
    // E o mínimo é o fundo da faixa: a distância ali é zero.
    assert!(v0 < -0.98, "the seed itself must bottom out, got {v0}");
    // Jitter cheio move-a para outro sítio da mesma célula.
    let moved = (at1.0 - at0.0).hypot(at1.1 - at0.1);
    assert!(
        moved > 0.05,
        "full jitter must move the seed, moved {moved}"
    );
    assert!(v1 < -0.98, "and it still bottoms out, got {v1}");
}

/// **`Cracks` é zero na FRONTEIRA e `Cells` não** — as duas metades da família
/// desenham coisas diferentes, o que é a razão de a opção existir.
#[test]
fn the_crack_feature_bottoms_out_on_the_cell_boundary_not_at_the_seed() {
    // O ponto onde `Cells` é mínimo (a semente) tem `Cracks` ALTO: ali a folga
    // até ao segundo vizinho é máxima.
    let (mut seed_at, mut best) = ((0.0f32, 0.0f32), f32::INFINITY);
    let (mut crack_at, mut worst_crack) = ((0.0f32, 0.0f32), f32::INFINITY);
    for gy in 0..96 {
        for gx in 0..96 {
            let (x, y) = (gx as f32 / 96.0, gy as f32 / 96.0);
            let c = cellular_2d(x, y, CellFeature::Cells, 1.0);
            if c < best {
                best = c;
                seed_at = (x, y);
            }
            let k = cellular_2d(x, y, CellFeature::Cracks, 1.0);
            if k < worst_crack {
                worst_crack = k;
                crack_at = (x, y);
            }
        }
    }
    let apart = (seed_at.0 - crack_at.0).hypot(seed_at.1 - crack_at.1);
    assert!(
        apart > 0.1,
        "the crack line and the seed are different places, apart = {apart}"
    );
    assert!(
        worst_crack < -0.95,
        "a boundary point has near-zero F2-F1, got {worst_crack}"
    );
    assert!(
        cellular_2d(seed_at.0, seed_at.1, CellFeature::Cracks, 1.0) > -0.5,
        "at the seed the gap to the runner-up is wide, not a crack"
    );
}

/// **Um índice fora do catálogo cai no kernel que sempre shipou**, nunca noutro
/// desenho e nunca num pânico — a mesma lei do `NoiseType::from_index`.
#[test]
fn an_unknown_kernel_index_falls_back_to_the_one_that_always_shipped() {
    for i in [-3.0f32, -0.4, 0.0, 0.49, 3.0, 99.0] {
        assert_eq!(Kernel::from_index(i), Kernel::Value, "index {i}");
    }
    assert_eq!(Kernel::from_index(0.5), Kernel::Perlin, "round half away");
    assert_eq!(Kernel::from_index(1.0), Kernel::Perlin);
    assert_eq!(Kernel::from_index(2.0), Kernel::Cellular);
    assert_eq!(CellFeature::from_index(1.0), CellFeature::Cracks);
    for i in [-1.0f32, 0.0, 2.0, 7.0] {
        assert_eq!(CellFeature::from_index(i), CellFeature::Cells, "index {i}");
    }
}

/// **SONDA — o defeito que o Simplex existe para curar, medido no Perlin que
/// temos.** A folha 15 lista `Simplex` ao lado de Perlin/Value/Cellular; o que
/// ele compra em 2D é ISOTROPIA (a grade triangular não tem eixos preferidos).
/// Se o nosso Perlin-2002 já for isotrópico dentro do ruído da medição, uma row
/// `Simplex` é um item de menu que o artista não distingue do vizinho.
///
/// Mede a correlação do campo a um passo fixo, por direcção, e reporta a
/// anisotropia `(max − min) / média`.
#[test]
#[ignore = "sonda — roda com --ignored"]
fn measure_the_directional_bias_each_kernel_has() {
    let dirs = 24;
    let lag = 0.35f32;
    for (name, f) in [
        (
            "value  ",
            &base as &dyn Fn(Kernel, CellFeature, f32, f32, f32) -> f32,
        ),
        ("perlin ", &base),
        ("cellular", &base),
    ]
    .iter()
    .zip([Kernel::Value, Kernel::Perlin, Kernel::Cellular])
    .map(|((n, f), k)| {
        (*n, move |x: f32, y: f32| {
            f(k, CellFeature::Cells, 1.0, x, y)
        })
    }) {
        let mut per_dir = Vec::new();
        for d in 0..dirs {
            // Direcções por rotação repetida de um vector unitário — sem `sin`.
            let a = d as f32 / dirs as f32;
            let (dx, dy) = unit_from_turn(a);
            let (mut sxy, mut sxx, mut syy) = (0.0f64, 0.0f64, 0.0f64);
            for k in 0..4000 {
                let (x, y) = ((k % 97) as f32 * 0.41 - 20.0, (k / 97) as f32 * 0.37 - 8.0);
                let a = f(x, y) as f64;
                let b = f(x + dx * lag, y + dy * lag) as f64;
                sxy += a * b;
                sxx += a * a;
                syy += b * b;
            }
            per_dir.push(sxy / (sxx.sqrt() * syy.sqrt()));
        }
        let lo = per_dir.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = per_dir.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = per_dir.iter().sum::<f64>() / per_dir.len() as f64;
        println!(
            "{name}: corr a lag {lag}  min {lo:.4}  max {hi:.4}  media {mean:.4}  \
             ANISOTROPIA {:.2}%",
            (hi - lo) / mean * 100.0
        );
    }
}

/// SONDA — o pico real de cada kernel, o número que justifica (ou derruba) o
/// `CELL_SPAN` e o `PERLIN_NORM`.
#[test]
#[ignore = "sonda — roda com --ignored"]
fn measure_the_peak_each_kernel_reaches() {
    for (name, k, feat) in [
        ("value", Kernel::Value, CellFeature::Cells),
        ("perlin", Kernel::Perlin, CellFeature::Cells),
        ("cell F1", Kernel::Cellular, CellFeature::Cells),
        ("cell F2-F1", Kernel::Cellular, CellFeature::Cracks),
    ] {
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        let mut saturated = 0u32;
        let n = 400;
        for gy in 0..n {
            for gx in 0..n {
                let (x, y) = (gx as f32 * 0.083 - 16.0, gy as f32 * 0.071 - 14.0);
                let v = base(k, feat, 1.0, x, y);
                lo = lo.min(v);
                hi = hi.max(v);
                if v >= 0.999 {
                    saturated += 1;
                }
            }
        }
        let total = n * n;
        println!(
            "{name:>10}: min {lo:+.4}  max {hi:+.4}  saturado {saturated}/{total} \
             ({:.3}%)",
            saturated as f32 / total as f32 * 100.0
        );
    }
}

/// Um vector unitário a `turn ∈ [0,1)` de volta, sem transcendentais: dobra o
/// ângulo por meio-ângulo a partir de `(1,0)`. Só a sonda o usa.
#[cfg(test)]
fn unit_from_turn(turn: f32) -> (f32, f32) {
    // 24 direcções = passos de 15°; construídas por bisseção repetida do ângulo
    // recto, que é exacta o suficiente para uma sonda.
    let steps = (turn * 24.0).round() as i32;
    let mut v = (1.0f32, 0.0f32);
    // Meia-volta de 15° por rotação de um complexo unitário pré-computado.
    const C15: (f32, f32) = (0.965_925_8, 0.258_819_04);
    for _ in 0..steps {
        v = (v.0 * C15.0 - v.1 * C15.1, v.0 * C15.1 + v.1 * C15.0);
    }
    v
}

/// SONDA — o PICO verdadeiro do Perlin, com o `PERLIN_NORM` já aplicado.
///
/// ⚠️ A varredura é por CÉLULA (o pico depende de que gradientes a hash sorteou
/// para as quatro quinas) com um sub-grid fino dentro de cada uma. A primeira
/// versão desta sonda varria um quadrado ESTREITO com passo fino e reportava
/// 0,895 — ela não continha o fenómeno, porque o pico vive noutras células.
#[test]
#[ignore = "sonda — roda com --ignored"]
fn measure_the_true_perlin_peak() {
    const CELLS: i32 = 90;
    const SUB: i32 = 24;
    let (mut hi, mut at) = (0.0f32, (0.0f32, 0.0f32));
    for cy in -CELLS..CELLS {
        for cx in -CELLS..CELLS {
            for sy in 0..SUB {
                for sx in 0..SUB {
                    let x = cx as f32 + sx as f32 / SUB as f32;
                    let y = cy as f32 + sy as f32 / SUB as f32;
                    let v = perlin_2d(x, y).abs();
                    if v > hi {
                        hi = v;
                        at = (x, y);
                    }
                }
            }
        }
    }
    println!(
        "perlin |pico| = {hi:.6} em {at:?}  =>  cru = {:.6}  (NORM = 1/1.5)",
        hi * 1.5
    );
}

/// **O `base` ROTEIA — cada índice chega ao seu kernel, e os três desenham
/// coisas DIFERENTES.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE.** Os irmãos acima chamam
/// `perlin_2d` e `cellular_2d` **directamente**, então colapsar a tabela de
/// despacho (`Kernel::Perlin => value_noise_2d(...)`) passava na suíte inteira:
/// os kernels estavam certos e ninguém afirmava que o selector os alcança. Na
/// GPU a fixture de paridade apanha-o — porque ela passa pelo NÓ —, mas um
/// defeito que só o device vê é um defeito que só se descobre com adaptador.
///
/// A segunda metade (`assert_ne` entre pares) é a que torna o selector uma
/// ESCOLHA: três índices que devolvem o mesmo campo são um menu decorativo.
#[test]
fn the_base_door_routes_each_index_to_its_own_kernel() {
    let pts = [(0.31f32, 0.77f32), (-4.2, 8.9), (12.05, -3.33), (0.5, 0.5)];
    for (x, y) in pts {
        assert_eq!(
            base(Kernel::Value, CellFeature::Cells, 1.0, x, y).to_bits(),
            value_noise_2d(x, y).to_bits(),
            "Value at ({x}, {y})"
        );
        assert_eq!(
            base(Kernel::Perlin, CellFeature::Cells, 1.0, x, y).to_bits(),
            perlin_2d(x, y).to_bits(),
            "Perlin at ({x}, {y})"
        );
        assert_eq!(
            base(Kernel::Cellular, CellFeature::Cracks, 0.4, x, y).to_bits(),
            cellular_2d(x, y, CellFeature::Cracks, 0.4).to_bits(),
            "Cellular at ({x}, {y}) — a feature e o jitter TEM de atravessar"
        );
    }
    // E os três são campos distintos: uma amostra grande onde nenhum par
    // coincide em quase-toda parte.
    let mut same_vp = 0u32;
    let mut same_vc = 0u32;
    let n = 40;
    for gy in 0..n {
        for gx in 0..n {
            let (x, y) = (gx as f32 * 0.31 - 6.0, gy as f32 * 0.27 - 5.0);
            let v = base(Kernel::Value, CellFeature::Cells, 1.0, x, y);
            let p = base(Kernel::Perlin, CellFeature::Cells, 1.0, x, y);
            let c = base(Kernel::Cellular, CellFeature::Cells, 1.0, x, y);
            if (v - p).abs() < 1e-6 {
                same_vp += 1;
            }
            if (v - c).abs() < 1e-6 {
                same_vc += 1;
            }
        }
    }
    let total = n * n;
    assert!(
        same_vp * 20 < total,
        "Value e Perlin coincidem em {same_vp}/{total}"
    );
    assert!(
        same_vc * 20 < total,
        "Value e Cellular coincidem em {same_vc}/{total}"
    );
}

/// SONDA — a distribuição CRUA de cada feature do celular (antes do mapeamento
/// para `[-1,1]`), que é o número que decide o `CELL_SPAN`.
#[test]
#[ignore = "sonda — roda com --ignored"]
fn measure_the_raw_cellular_distribution() {
    for (name, feat) in [("F1", CellFeature::Cells), ("F2-F1", CellFeature::Cracks)] {
        for jitter in [0.0f32, 0.5, 1.0] {
            let (mut lo, mut hi, mut sum) = (f32::MAX, f32::MIN, 0.0f64);
            let mut n = 0u32;
            for gy in 0..600 {
                for gx in 0..600 {
                    let (x, y) = (gx as f32 * 0.037 - 11.0, gy as f32 * 0.041 - 12.0);
                    // Desfaz o mapeamento para ler o CRU.
                    let v = (cellular_2d(x, y, feat, jitter) + 1.0) * 0.5;
                    lo = lo.min(v);
                    hi = hi.max(v);
                    sum += v as f64;
                    n += 1;
                }
            }
            println!(
                "{name:>6} jitter {jitter:.1}: cru min {lo:.4} max {hi:.4} media {:.4}",
                sum / n as f64
            );
        }
    }
}
