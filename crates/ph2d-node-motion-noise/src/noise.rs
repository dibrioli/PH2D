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

/// **A BASE do ruído** (doc 89, folha 06 linha 21) — e ela **não** é o `type`.
///
/// ⚠️ **A célula tinha razão sobre uma distinção que é fácil de perder:** o `type` deste nó
/// escolhe a **rectificação por oitava** (fBm · turbulência · ridged), que é o que se faz
/// *com* o ruído; a BASE é o ruído em si. Trocar `type` muda como as camadas se somam;
/// trocar a base muda a FEIÇÃO — e nenhum valor de `type` produz uma célula.
///
/// ⛔ **E as três saídas que o catálogo parecia já ter não serviam**, o que é a razão de
/// isto ser família nova e não um knob: o `motion.voronoi` produz **células como geometria**
/// (não um campo escalar para deslocar um canal) e o `value.noise` é *value* noise no
/// domínio `v`, **sem posição**.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Basis {
    pub kind: i32,
    pub metric: i32,
}

impl Basis {
    /// O ruído de gradiente — o que este nó sempre foi, e o default.
    pub(crate) const GRADIENT: Self = Self {
        kind: BASE_GRADIENT,
        metric: 0,
    };

    /// A escada dos params, com o fora-de-alcance a cair no de sempre.
    pub(crate) fn from_params(kind: f32, metric: f32) -> Self {
        let k = match kind.round() as i32 {
            BASE_VALUE => BASE_VALUE,
            BASE_CELLULAR => BASE_CELLULAR,
            // ⚠️ Pela CONSTANTE e não pelo literal: ela é a que o doc-comment chama «o de
            // sempre», e um gate compara-a com a função de sempre ao bit.
            _ => Self::GRADIENT.kind,
        };
        Self {
            kind: k,
            metric: (metric.round() as i32).clamp(0, 2),
        }
    }
}

pub(crate) const BASE_GRADIENT: i32 = 0;
pub(crate) const BASE_VALUE: i32 = 1;
pub(crate) const BASE_CELLULAR: i32 = 2;

/// Um número em `[-1, 1]` a partir de um hash — os 24 bits de cima, que são os que o
/// misturador espalha melhor.
fn hash_unit(h: u32) -> f32 {
    f64::from(h >> 8) as f32 / 8_388_607.5 - 1.0
}

/// **VALUE noise** — o valor de cada canto do reticulado, interpolado pela MESMA quíntica.
///
/// ⚠️ **Ele tem o defeito que o cabeçalho deste módulo descreve, e é por isso que existe:**
/// os extremos caem *nos* pontos do reticulado, logo o campo mostra a grelha. Isso é uma
/// FEIÇÃO quando se quer um look mais duro e quadriculado — e é o que a referência ship ao
/// lado do Perlin em vez de o substituir.
pub(crate) fn value_noise_2d(x: f32, y: f32, seed: i32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (u, v) = (fade(x - x0), fade(y - y0));
    let c = |dx: i32, dy: i32| hash_unit(hash(ix + dx, iy + dy, seed));
    let nx0 = c(0, 0) + u * (c(1, 0) - c(0, 0));
    let nx1 = c(0, 1) + u * (c(1, 1) - c(0, 1));
    nx0 + v * (nx1 - nx0)
}

/// A distância entre dois pontos sob a métrica pedida.
///
/// ⚠️ **O vocabulário é o do `motion.voronoi`, LITERALMENTE** — as mesmas três palavras na
/// mesma ordem. Um artista que aprendeu «Chebyshev» num nó não pode encontrar «Máximo» no
/// outro; o censo que o afirma vive no `registry-init`, que é quem vê os dois.
fn metric_dist(dx: f32, dy: f32, metric: i32) -> f32 {
    match metric {
        1 => dx.abs() + dy.abs(),
        2 => dx.abs().max(dy.abs()),
        _ => (dx * dx + dy * dy).sqrt(),
    }
}

/// **A normalização de cada métrica** — ⚠️ MEDIDA, não escolhida (sonda
/// `measure_the_cellular_peak`): é o F1 máximo que a busca 3×3 alcança, e ele é **diferente
/// por métrica** porque a «bola» unitária tem forma diferente em cada uma.
///
/// | métrica | seed 7 | seed 101 | seed 4242 | fica |
/// |---|---|---|---|---|
/// | Euclidiana | `1,1685` | `1,1347` | `1,1190` | **`1,17`** |
/// | Manhattan | `1,6469` | `1,6013` | `1,5319` | **`1,65`** |
/// | Chebyshev | `0,9750` | `0,9518` | `0,9534` | **`0,98`** |
///
/// ⚠️ **Três sementes, e não uma** — a variação entre elas é ~5%, o que torna o número uma
/// LEI e não uma amostra. Fica o maior observado, e o `clamp` da saída é a rede para a cauda
/// que três varreduras não visitaram.
///
/// ⛔ **Uma normalização única teria estragado duas das três:** a Manhattan chega a `1,65` e
/// sairia lavada (nunca alcançaria os extremos da faixa), e a Chebyshev pára em `0,98` e
/// saturaria contra o `clamp`.
const CELL_PEAK: [f32; 3] = [1.17, 1.65, 0.98];

/// **CELLULAR / Worley F1** — a distância ao ponto-feição mais próximo, em `[-1, 1]`.
///
/// ⚠️ **O centro de cada célula lê `+1` e as fronteiras lêem `−1`**, que é a convenção que
/// põe esta base na MESMA faixa do gradiente — e isso não é cosmético: a faixa natural
/// alimenta o mapeamento `min`/`max` deste nó, e uma base de outra faixa partiria esse par
/// **em silêncio** (a armadilha que a folha 06 já registou com a onda `Custom`).
pub(crate) fn cellular_2d(x: f32, y: f32, seed: i32, metric: i32) -> f32 {
    let norm = CELL_PEAK[metric.clamp(0, 2) as usize];
    (1.0 - 2.0 * (cellular_f1(x, y, seed, metric) / norm)).clamp(-1.0, 1.0)
}

/// **O F1 CRU** — a distância ao ponto-feição, sem normalizar nem aparar.
///
/// ⚠️ **Existe porque a sonda que mede o pico não pode ler a saída aparada.** A 1.ª versão
/// dela desfazia a normalização de `cellular_2d` e imprimia `1,0000` nas três métricas — que
/// é exactamente o tecto do `clamp`, e não o pico. *Uma régua construída sobre uma grandeza
/// já limitada não consegue exprimir a resposta que procura.*
pub(crate) fn cellular_f1(x: f32, y: f32, seed: i32, metric: i32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (ix, iy) = (x0 as i32, y0 as i32);
    let (fx, fy) = (x - x0, y - y0);
    let mut best = f32::MAX;
    for gy in -1..=1 {
        for gx in -1..=1 {
            let h = hash(ix + gx, iy + gy, seed);
            // O ponto-feição da célula, em `[0,1)²` — dois campos do mesmo hash.
            let px = (h & 0xffff) as f32 / 65_536.0;
            let py = ((h >> 16) & 0xffff) as f32 / 65_536.0;
            let d = metric_dist(gx as f32 + px - fx, gy as f32 + py - fy, metric);
            best = best.min(d);
        }
    }
    best
}

/// O ruído de base pedido, em `[-1, 1]`.
pub(crate) fn base_noise_2d(x: f32, y: f32, seed: i32, b: Basis) -> f32 {
    match b.kind {
        BASE_VALUE => value_noise_2d(x, y, seed),
        BASE_CELLULAR => cellular_2d(x, y, seed, b.metric),
        _ => gradient_noise_2d(x, y, seed),
    }
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
pub(crate) fn octave(x: f32, y: f32, o: u32, seed: i32, b: Basis) -> f32 {
    base_noise_2d(x, y, seed.wrapping_add(o as i32 * 1013), b)
}

/// **A composição deste nó**: a lei da folha sobre o ruído de gradiente dele.
/// Uma porta, dois chamadores — o `eval` e os gates —, senão o que os gates
/// medem deixa de ser o que o produto faz.
pub(crate) fn fbm(x: f32, y: f32, seed: i32, spec: ph2d_fbm::Spec, b: Basis) -> f32 {
    ph2d_fbm::eval(spec, x, y, |px, py, o| octave(px, py, o, seed, b))
}

#[cfg(test)]
mod tests {

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
}
