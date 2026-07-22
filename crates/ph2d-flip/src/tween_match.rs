//! **A correspondência de traços** — quem vira quem no tween (Tween v2, `04 §2`).
//!
//! O GP pareia **por índice** (curva *i* ↔ curva *i*, `interpolate.cc:244-315`): zero
//! correspondência espacial, e o que salva o usuário é desenhar sempre na mesma ordem.
//! Aqui a ordem de desenho continua contando — **como um TERMO do custo**, não como a
//! resposta. Isso é subsunção, não substituição: quando tudo mais empata, o par ordinal
//! ganha e o v2 devolve exatamente o que o v1 devolvia.
//!
//! O custo (`04 §2`, com os pesos de lá):
//!
//! ```text
//! custo(i,j) = ∞  se aberta/fechada incompatíveis
//!            | 0.40·|Δcentróide|/D + 0.25·|ΔL|/max(L) + 0.20·Δeixo + 0.15·Δordem/span
//! ```
//!
//! Três decisões que separam isto de uma tradução literal da fórmula:
//!
//! 1. **As features são INTEGRADAS ao longo da polilinha**, nunca médias de vértice — dois
//!    traços com a MESMA forma e densidades de ponto diferentes têm de dar as mesmas
//!    features. É a lição que a linha do Vetor pagou no `ph2d-vec-blend` (*"picar uma aresta
//!    reta em 20 pedaços mudava a correspondência"*): âncora é **parametrização**, não
//!    geometria.
//! 2. **`Δeixo` é `1 − |û·v̂|`**, não `Δângulo/(π/2)`: transcendental-free (regra 9 do plano,
//!    e são O(n²) pares) e **quadrático perto de zero** — uma rotação pequena, que é o que
//!    um par de inbetween de fato tem, quase não custa; uma perpendicular custa cheio.
//! 3. **Termo indisponível é OMITIDO, nunca contado como zero** — uma forma isotrópica
//!    (ponto, círculo) não tem eixo, e contá-lo zero premiaria justamente quem não trouxe
//!    informação. O custo é a média PONDERADA dos termos que existem.
//!
//! A atribuição é **ótima** (Hungarian/Jonker-Volgenant O(n³), `assign`), não gulosa, e o
//! gate a compara com a busca exaustiva de permutações.
//!
//! **O custo é MEDIDO** (`the_plan_cost_ruler`), porque *"n é pequeno"* era uma afirmação
//! sobre um número que ninguém tinha olhado — um line-art denso tem centenas de traços:
//!
//! ```text
//!   traços     10      50     100     200     400     800
//!   plano    0,004   0,021   0,060   0,223   0,820   3,226  ms
//! ```
//!
//! Nesta faixa quem domina é a matriz de custo (`O(n·m)`), não o solver. E o plano é
//! construído **UMA vez por intervalo** — não por quadro e não por inbetween —, então
//! 3,2 ms num desenho de 800 traços é pago uma vez no clique do `Add`.

use crate::drawing::FlipDrawing;
use crate::stroke::FlipStroke;
use ph2d_core::Vec2;

/// Pesos do custo (`04 §2`). Somam 1 quando os quatro termos estão disponíveis.
const W_CENTROID: f32 = 0.40;
const W_LENGTH: f32 = 0.25;
const W_AXIS: f32 = 0.20;
const W_ORDER: f32 = 0.15;

/// **Custo acima do qual o par é RECUSADO** (os dois viram órfãos).
///
/// MEDIDO pela régua `the_cost_ruler` (tabela em `docs/Flip/11_tween_v2.md §3`), e a
/// medição **desmentiu o número que eu esperava**: as duas colunas SE CRUZAM.
///
/// ```text
/// LEGÍTIMOS                      ESPÚRIOS
///   anda 20            0.0370      braço × cotoco       0.2774
///   gira 45            0.0964      braço × perna        0.4261
///   gira 90            0.2653      braço × canto        0.5020
///   gira 90 + −30%     0.3352
/// ```
///
/// Nenhum limiar separa `0.3352` de `0.2774` — porque o "cotoco" **não é espúrio**: um
/// braço que encolhe muito é exatamente esse par. O que a tabela separa de fato é a zona
/// AMBÍGUA (0.27–0.34, onde os dois tipos convivem) do claramente-alheio (≥ 0.426), e é
/// aí que o limiar mora: `0.38` é o meio desse vão (+0.045 sobre o pior legítimo medido,
/// −0.046 sob o melhor alheio).
///
/// **A política, que a escolha do meio implementa: na dúvida, PAREAR.** Um par estranho é
/// um inbetween torto que o artista vê e corrige; um órfão é um traço que SOME (ou pisca)
/// no meio da animação — o erro mais caro dos dois, e o mais difícil de diagnosticar.
pub const PAIR_REJECT_COST: f32 = 0.38;

/// O custo de um par PROIBIDO (aberto × fechado).
///
/// Finito de propósito: o solver de atribuição trabalha com potenciais, e um `∞` os
/// envenena (`∞ − ∞`); um número grande dá o mesmo resultado — e o limiar de recusa o
/// descarta depois de qualquer jeito.
const BLOCKED: f32 = 1.0e6;

/// Abaixo desta anisotropia a forma NÃO tem eixo principal (círculo, ponto, blob).
///
/// MEDIDO por `the_cost_ruler`: círculo 48-gon **0.0000** · elipse 1.05:1 **0.0366** ·
/// 1.1:1 **0.0715** · 1.3:1 **0.1951** · 2:1 **0.4872** · reta com tremor de mão
/// **0.9992**. O piso fica entre a elipse de 5% e a de 10% de excentricidade: 5% é ruído
/// de mão e não dita direção nenhuma; 10% já é uma forma que aponta para algum lado.
const AXIS_MIN_ANISOTROPY: f32 = 0.05;

/// **O par é recusado?** — duas perguntas, e as DUAS têm de dizer sim.
///
/// 1. **Absoluta:** o par é implausível por si (`> PAIR_REJECT_COST`)?
/// 2. **Relativa:** ele é um OUTLIER entre os pares deste desenho (`> k × mediana`)?
///
/// ⚠️ **A segunda existe porque a primeira sozinha estava ERRADA, e o gate do buraco a
/// pegou** (um quadrado sozinho que viaja 5× o próprio tamanho era recusado, e nos
/// inbetweens ele ficava parado em A para saltar a B no fim). Pior: com o limiar absoluto
/// sozinho, uma **panorâmica** — a cena INTEIRA se deslocando — orfanaria *todo* traço do
/// desenho de uma vez, porque todos os custos sobem juntos. Um custo alto só significa
/// "isto não é o mesmo traço" quando os VIZINHOS não subiram junto.
///
/// A forma escolhida apaga o caso especial: com **um** par só, ele É a própria mediana, e
/// `c > k·c` é falso para `k ≥ 1` ⇒ pareia. Sem `if n == 1` em lugar nenhum.
fn rejects(cost: f32, median: f32) -> bool {
    cost > PAIR_REJECT_COST && cost > OUTLIER_FACTOR * median
}

/// Quantas vezes a mediana dos outros pares um custo precisa ser para virar outlier.
///
/// MEDIDO por `the_outlier_ruler`, e o vão é enorme: numa panorâmica (a cena inteira
/// andando, `dx` de 40 a 400) a razão dá **1,000 exato** — o deslocamento é comum a todos,
/// então todo par carrega o mesmo custo e a mediana É esse custo. Num desenho onde um traço
/// SOME e outro NASCE, o par espúrio dá **246,6×** a mediana.
///
/// Entre 1 e 246 qualquer coisa serve; `2.0` é conservador dos dois lados — a política é
/// *na dúvida, parear*.
const OUTLIER_FACTOR: f32 = 2.0;

/// A assinatura geométrica de um traço — o que a correspondência compara.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeFeatures {
    /// Centróide da POLILINHA (ponderado por comprimento de arco).
    pub centroid: Vec2,
    /// Comprimento de arco total (inclui a costura, se fechado).
    pub arclen: f32,
    /// Eixo principal (PCA) como vetor UNITÁRIO — uma RETA, não uma direção: o sinal não
    /// significa nada, e por isso o custo usa `|û·v̂|`. `None` = forma isotrópica.
    pub axis: Option<Vec2>,
    /// Aberta × fechada é incompatibilidade DURA (um contorno não vira uma linha).
    pub closed: bool,
    /// Canto mínimo do bbox dos PONTOS (a régua de `Δcentróide` sai da união destes).
    pub lo: Vec2,
    /// Canto máximo do bbox dos pontos.
    pub hi: Vec2,
}

/// As features de um traço (ver o módulo: momentos integrados, não médias de vértice).
#[must_use]
pub fn features(s: &FlipStroke) -> StrokeFeatures {
    let (mut lo, mut hi) = (
        Vec2::new(f32::INFINITY, f32::INFINITY),
        Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for &p in s.positions() {
        lo = Vec2::new(lo.x.min(p.x), lo.y.min(p.y));
        hi = Vec2::new(hi.x.max(p.x), hi.y.max(p.y));
    }
    if !lo.x.is_finite() {
        (lo, hi) = (Vec2::ZERO, Vec2::ZERO); // traço sem ponto nenhum
    }
    let (mut len, mut m1, mut mxx, mut mxy, mut myy) = (0.0f32, Vec2::ZERO, 0.0, 0.0, 0.0);
    for (_, p, q) in s.segments() {
        let d = q - p;
        let l = (d.x * d.x + d.y * d.y).sqrt();
        if l <= 0.0 {
            continue;
        }
        len += l;
        m1 += (p + q) * (0.5 * l);
        // ∫ x·xᵀ ds sobre o segmento = L·[ p·pᵀ + (p·dᵀ + d·pᵀ)/2 + d·dᵀ/3 ] — EXATO
        // (a regra do ponto-médio erraria a variância ao longo do próprio segmento).
        mxx += l * (p.x * p.x + p.x * d.x + d.x * d.x / 3.0);
        mxy += l * (p.x * p.y + 0.5 * (p.x * d.y + d.x * p.y) + d.x * d.y / 3.0);
        myy += l * (p.y * p.y + p.y * d.y + d.y * d.y / 3.0);
    }
    if len <= 0.0 {
        // Traço degenerado (um ponto, ou todos os pontos coincidentes): não há arco para
        // integrar, então o centróide é a média dos vértices e não há eixo.
        let n = s.len().max(1) as f32;
        let c = s.positions().iter().fold(Vec2::ZERO, |a, &p| a + p) / n;
        return StrokeFeatures {
            centroid: c,
            arclen: 0.0,
            axis: None,
            closed: s.closed,
            lo,
            hi,
        };
    }
    let c = m1 / len;
    // Covariância em torno do centróide (teorema dos eixos paralelos).
    let (cxx, cxy, cyy) = (
        mxx / len - c.x * c.x,
        mxy / len - c.x * c.y,
        myy / len - c.y * c.y,
    );
    StrokeFeatures {
        centroid: c,
        arclen: len,
        axis: principal_axis(cxx, cxy, cyy),
        closed: s.closed,
        lo,
        hi,
    }
}

/// O autovetor do maior autovalor da covariância 2×2 — fechado, sem iteração e sem
/// transcendental. `None` quando a forma é isotrópica demais para ter direção.
fn principal_axis(cxx: f32, cxy: f32, cyy: f32) -> Option<Vec2> {
    let tr = cxx + cyy;
    if tr <= 0.0 {
        return None;
    }
    let disc = ((cxx - cyy) * (cxx - cyy) + 4.0 * cxy * cxy)
        .max(0.0)
        .sqrt();
    // anisotropia = (λ₁−λ₂)/(λ₁+λ₂) = disc/tr — 0 no círculo, 1 na reta.
    if disc / tr < AXIS_MIN_ANISOTROPY {
        return None;
    }
    let l1 = 0.5 * (tr + disc);
    // Dois candidatos a autovetor; o de maior norma é o numericamente estável (o outro
    // colapsa quando cxy → 0, que é justamente o caso alinhado aos eixos).
    let (u, v) = (Vec2::new(cxy, l1 - cxx), Vec2::new(l1 - cyy, cxy));
    let (nu, nv) = (u.x * u.x + u.y * u.y, v.x * v.x + v.y * v.y);
    let w = if nu >= nv { u } else { v };
    let n = (w.x * w.x + w.y * w.y).sqrt();
    (n > 0.0).then(|| w / n)
}

/// O que normaliza o custo — o mesmo para todos os pares de um plano.
#[derive(Clone, Copy, Debug)]
struct CostCtx {
    /// Diagonal do bbox da UNIÃO dos dois desenhos: `|Δcentróide|` vira fração da cena.
    diag: f32,
    /// `max(n_a, n_b) − 1`: `Δordem` vira fração do intervalo de índices possível.
    order_span: f32,
}

/// **O custo do par `(ia, ib)`** — a média ponderada dos termos DISPONÍVEIS.
fn pair_cost(fa: &StrokeFeatures, fb: &StrokeFeatures, ia: usize, ib: usize, ctx: CostCtx) -> f32 {
    if fa.closed != fb.closed {
        return BLOCKED;
    }
    let (mut num, mut den) = (0.0f32, 0.0f32);
    let mut term = |w: f32, d: f32| {
        num += w * d.clamp(0.0, 1.0);
        den += w;
    };

    let dc = fb.centroid - fa.centroid;
    term(W_CENTROID, (dc.x * dc.x + dc.y * dc.y).sqrt() / ctx.diag);

    let lmax = fa.arclen.max(fb.arclen);
    if lmax > 0.0 {
        term(W_LENGTH, (fa.arclen - fb.arclen).abs() / lmax);
    }

    if let (Some(u), Some(v)) = (fa.axis, fb.axis) {
        term(W_AXIS, 1.0 - (u.x * v.x + u.y * v.y).abs());
    }

    if ctx.order_span > 0.0 {
        let d = (ia as f32 - ib as f32).abs();
        term(W_ORDER, d / ctx.order_span);
    }

    if den > 0.0 { num / den } else { 0.0 }
}

/// **O plano de correspondência entre dois desenhos** — quem vira quem, e a que custo.
///
/// É função do PAR, não do fator `t`: buscá-la por inbetween seria refazer o mesmo
/// trabalho N vezes (a mesma lição que o `Plan` do `ph2d-vec-blend` pagou). O `tween` a
/// constrói UMA vez e a reusa em todos os quadros do intervalo — e a UI a lê para desenhar
/// as linhas de par.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TweenPlan {
    /// Para cada traço de A, o traço de B com que ele foi casado (`None` = órfão).
    a_to_b: Vec<Option<usize>>,
    /// O inverso (`None` = traço de B que ninguém reclamou: aparece do nada).
    b_to_a: Vec<Option<usize>>,
    /// O custo do par de cada traço de A (`None` onde não há par).
    cost: Vec<Option<f32>>,
}

impl TweenPlan {
    /// Constrói o plano: features → matriz de custo → atribuição ótima → limiar.
    #[must_use]
    pub fn build(a: &FlipDrawing, b: &FlipDrawing) -> Self {
        let fa: Vec<StrokeFeatures> = a.strokes.iter().map(features).collect();
        let fb: Vec<StrokeFeatures> = b.strokes.iter().map(features).collect();
        Self::from_features(&fa, &fb)
    }

    /// A metade testável: o plano a partir das features já extraídas.
    #[must_use]
    pub fn from_features(fa: &[StrokeFeatures], fb: &[StrokeFeatures]) -> Self {
        let (n, m) = (fa.len(), fb.len());
        let mut plan = Self {
            a_to_b: vec![None; n],
            b_to_a: vec![None; m],
            cost: vec![None; n],
        };
        if n == 0 || m == 0 {
            return plan;
        }
        let ctx = CostCtx {
            diag: union_diag(fa, fb),
            order_span: (n.max(m) - 1) as f32,
        };
        let mut costs = vec![0.0f32; n * m];
        for (i, f) in fa.iter().enumerate() {
            for (j, g) in fb.iter().enumerate() {
                costs[i * m + j] = pair_cost(f, g, i, j, ctx);
            }
        }
        // O solver casa TODOS que puder (é uma atribuição); quem decide se o par SIGNIFICA
        // alguma coisa é a recusa abaixo. Sem ela, o último traço sobrando de A seria
        // casado com o último de B por eliminação, e o inbetween mostraria um braço virando
        // um pé do outro lado da tela.
        let picked = assign(&costs, n, m);
        let mut sorted: Vec<f32> = picked
            .iter()
            .map(|&(i, j)| costs[i * m + j])
            .filter(|c| *c < BLOCKED)
            .collect();
        sorted.sort_by(f32::total_cmp);
        let median = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
        for (i, j) in picked {
            let c = costs[i * m + j];
            if !rejects(c, median) {
                plan.a_to_b[i] = Some(j);
                plan.b_to_a[j] = Some(i);
                plan.cost[i] = Some(c);
            }
        }
        plan
    }

    /// O par do traço `i` de A (`None` = órfão de A).
    #[must_use]
    pub fn pair_of_a(&self, i: usize) -> Option<usize> {
        self.a_to_b.get(i).copied().flatten()
    }

    /// O par do traço `j` de B (`None` = órfão de B: nasce no meio do caminho).
    #[must_use]
    pub fn pair_of_b(&self, j: usize) -> Option<usize> {
        self.b_to_a.get(j).copied().flatten()
    }

    /// O custo do par do traço `i` de A — o que a UI mostra para dizer *quão confiante*
    /// a correspondência está (a lição CACANi: o matcher erra, e o artista precisa VER).
    #[must_use]
    pub fn cost_of_a(&self, i: usize) -> Option<f32> {
        self.cost.get(i).copied().flatten()
    }

    /// Quantos pares o plano tem.
    #[must_use]
    pub fn pairs(&self) -> usize {
        self.a_to_b.iter().filter(|p| p.is_some()).count()
    }
}

/// A diagonal do bbox da união dos dois DESENHOS — a régua de `Δcentróide`.
///
/// ⚠️ **É o bbox dos PONTOS, não o dos centróides.** Com centróides, um desenho de um traço
/// só teria a diagonal igual ao próprio deslocamento ⇒ o termo saturaria em `1.0` para
/// QUALQUER movimento e o caso mais simples que existe — um traço que anda — seria recusado
/// como par espúrio. O bbox dos pontos dá a escala da CENA, que é o que a fração quer dizer.
///
/// Nunca zero: um desenho de um ponto só teria diagonal 0 e o termo viraria `NaN`. O piso é
/// 1 unidade de documento, e aí o termo mede deslocamento ABSOLUTO — a resposta honesta
/// quando não há cena para servir de escala.
fn union_diag(fa: &[StrokeFeatures], fb: &[StrokeFeatures]) -> f32 {
    let (mut lo, mut hi) = (
        Vec2::new(f32::INFINITY, f32::INFINITY),
        Vec2::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
    );
    for f in fa.iter().chain(fb) {
        lo = Vec2::new(lo.x.min(f.lo.x), lo.y.min(f.lo.y));
        hi = Vec2::new(hi.x.max(f.hi.x), hi.y.max(f.hi.y));
    }
    let d = hi - lo;
    if !d.x.is_finite() || !d.y.is_finite() {
        return 1.0;
    }
    (d.x * d.x + d.y * d.y).sqrt().max(1.0)
}

/// **Atribuição de custo mínimo** (Hungarian / Jonker-Volgenant, `O(n³)`).
///
/// `costs` é `n × m` em row-major. Devolve `min(n, m)` pares `(linha, coluna)`, ordenados
/// por linha. É ÓTIMA — o gate a compara com a enumeração exaustiva de permutações para
/// `n ≤ 6`, que é o único oráculo honesto para um solver de atribuição (um espelho da
/// própria implementação provaria só que ela é igual a si mesma).
///
/// Determinística: os empates caem no menor índice, porque a varredura é crescente e a
/// comparação é estrita.
fn assign(costs: &[f32], n: usize, m: usize) -> Vec<(usize, usize)> {
    if n == 0 || m == 0 {
        return Vec::new();
    }
    // O algoritmo exige linhas ≤ colunas; se não for o caso, resolve o TRANSPOSTO e
    // devolve os pares trocados (a atribuição ótima é a mesma).
    if n > m {
        let mut t = vec![0.0f32; n * m];
        for i in 0..n {
            for j in 0..m {
                t[j * n + i] = costs[i * m + j];
            }
        }
        let mut out: Vec<(usize, usize)> =
            assign(&t, m, n).into_iter().map(|(j, i)| (i, j)).collect();
        out.sort_unstable();
        return out;
    }
    // Formulação clássica com potenciais + caminho aumentante mais curto, 1-indexada
    // (o índice 0 é a sentinela do caminho).
    let a = |i: usize, j: usize| costs[(i - 1) * m + (j - 1)];
    let (mut u, mut v) = (vec![0.0f32; n + 1], vec![0.0f32; m + 1]);
    let mut p = vec![0usize; m + 1];
    let mut way = vec![0usize; m + 1];

    for i in 1..=n {
        p[0] = i;
        let mut j0 = 0usize;
        let mut minv = vec![f32::INFINITY; m + 1];
        let mut used = vec![false; m + 1];
        loop {
            used[j0] = true;
            let i0 = p[j0];
            let (mut delta, mut j1) = (f32::INFINITY, 0usize);
            for j in 1..=m {
                if used[j] {
                    continue;
                }
                let cur = a(i0, j) - u[i0] - v[j];
                if cur < minv[j] {
                    minv[j] = cur;
                    way[j] = j0;
                }
                if minv[j] < delta {
                    delta = minv[j];
                    j1 = j;
                }
            }
            for j in 0..=m {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    minv[j] -= delta;
                }
            }
            j0 = j1;
            if p[j0] == 0 {
                break;
            }
        }
        // Desfaz o caminho, trocando as atribuições ao longo dele.
        loop {
            let j1 = way[j0];
            p[j0] = p[j1];
            j0 = j1;
            if j0 == 0 {
                break;
            }
        }
    }
    let mut out: Vec<(usize, usize)> = (1..=m)
        .filter(|&j| p[j] != 0)
        .map(|j| (p[j] - 1, j - 1))
        .collect();
    out.sort_unstable();
    out
}

#[cfg(test)]
#[path = "tween_match_tests.rs"]
mod tests;
