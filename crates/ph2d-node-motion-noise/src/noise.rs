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
#[path = "noise_tests.rs"]
mod tests;
