//! **O ajuste de Hobby** — a curva que PASSA pelos pontos que a mão deixou.
//!
//! É o motor do **lápis** (W1 do plano 25): o gesto grava amostras, o decimador as reduz a
//! *nós*, e este módulo devolve o `VecVertex` de cada nó — âncora + os dois handles — de uma
//! spline cúbica que **interpola** todos eles com variação mínima de curvatura (o `..` do
//! MetaPost; Hobby 1986, velocidade de Jackowski).
//!
//! # Por que interpolar e não aproximar
//!
//! O outro caminho é o *least-squares* de Schneider (o default do Inkscape), que **não passa**
//! pelas amostras: ele acha a cúbica mais próxima e o traço escorrega do lugar onde o artista o
//! desenhou. O `17_plano_de_implementacao.md` §5 já o marcava como o anti-padrão, e o motivo é
//! ergonômico, não estético: quando a curva não passa por onde a mão passou, corrigi-la é
//! adivinhar. (O Schneider **tem** o seu lugar neste repo — o `Simplify` e o *record* da timeline
//! o usam para REDUZIR uma curva que já existe, que é a pergunta oposta.)
//!
//! # Por que este arquivo é um PORT, e não uma aresta de dependência
//!
//! O solver já existia em `ph2d-vector-doc::hobby` — **533 LOC, testado, com ZERO chamadores**,
//! e o doc dele nomeia o consumidor que nunca chegou (*"the Pencil tool smooths a recorded
//! freehand stroke"*). Mas aquela crate é o modelo vetorial **CONGELADO** (§6, ADR-0056..0068) e
//! fala `glam::Vec2` (f32), enquanto a engine nova fala `[f64; 2]`. Duas escolhas ruins e uma boa:
//!
//! | rota | preço |
//! |---|---|
//! | `ph2d-vec-edit` depende da crate congelada | uma aresta de produção para um modelo legado que este módulo substituiu (ADR-0108), só para reusar 300 linhas de aritmética |
//! | conviver com o f32 na fronteira | o resto da engine é f64; converter de ida e volta perde precisão exatamente onde o artista amplia para conferir |
//! | **portar para f64 num módulo leaf** ✅ | o ajuste passa a falar a linguagem do consumidor (devolve `VecVertex`, não tangentes por segmento) e a crate congelada fica intocada |
//!
//! ⚠️ **O port é provado contra o original.** A crate congelada entra em `[dev-dependencies]`
//! (machete-safe: nenhuma linha de `src/` a usa) e o gate `hobby_tests` compara os dois lado a
//! lado, com o épsilon do `as f32` do original documentado. Um port de 300 linhas de aritmética
//! sem oráculo externo é uma reescrita torcendo por sorte.
//!
//! # Método (uma frase por linha do sistema)
//!
//! Para nós `z₀ … zₙ` as incógnitas são os ângulos de PARTIDA `αᵢ` (medidos a partir da corda);
//! os de CHEGADA saem de `βᵢ = −γᵢ₊₁ − αᵢ₊₁`, onde `γ` são os ângulos de virada do polígono — e
//! **é essa relação que dá a tangente contínua** (o mesmo ângulo serve os dois lados do nó), o
//! que faz de todo nó interior um [`VertexKind::Smooth`] por construção, não por arredondamento.
//! Os `αᵢ` satisfazem um sistema **tridiagonal** com dominância diagonal estrita (Thomas resolve
//! sem pivotar); o comprimento dos handles vem da *velocidade* de Jackowski
//! `ρ(α,β) = 2 / (1 + ⅔·cos β + ⅓·cos α)`, capada em [`MAX_VELOCITY`] para que um nó quase-cúspide
//! não produza um laço.
//!
//! # HR-5 (determinismo)
//!
//! Este ajuste chama `sin`/`cos`/`atan2` — e **pode**: ele roda no caminho de ESCRITA do editor,
//! sobre entrada de mão livre que não é reproduzível por definição. Nada aqui alcança o caminho
//! determinista (nenhuma sim, nenhum hash). A aritmética nunca panica e nunca devolve NaN para
//! entrada finita (entrada não-finita cai na cadeia reta, ver [`straight_thirds`]).

use ph2d_vec_scene::VecVertex;

/// *Curl* de ponta (`ω`) default. `0` = a condição relaxada, que deixa a spline endireitar
/// naturalmente rumo às pontas — o mais neutro para um traço de mão livre.
pub const DEFAULT_CURL: f64 = 0.0;

/// Teto da velocidade de Jackowski `ρ`. O MetaPost capa em 4 para que uma virada de quase 180°
/// num nó não estufe o braço do handle até um laço auto-intersectante. 4 já corresponde a um
/// handle de `4·d/3`, muito além do que qualquer traço suave precisa.
const MAX_VELOCITY: f64 = 4.0;

/// Abaixo deste comprimento de corda um par de nós é tratado como coincidente: o segmento
/// degenera numa cúbica reta de comprimento zero em vez de dividir por zero. O decimador já
/// deduplica amostras coincidentes — este é o piso defensivo.
const MIN_CHORD: f64 = 1e-6;

/// Ajusta uma spline cúbica **aberta** e suave que passa por `knots`, com o curl default.
///
/// Devolve **um [`VecVertex`] por nó** (`knots.len()` deles), com handles em coordenadas
/// ABSOLUTAS — a convenção do `VecVertex`. As pontas têm o handle de fora do traço parado na
/// própria âncora (não há segmento daquele lado). Menos de 2 nós devolve vazio.
#[must_use]
pub fn fit_hobby_open(knots: &[[f64; 2]]) -> Vec<VecVertex> {
    fit_hobby_open_with_curl(knots, DEFAULT_CURL)
}

/// [`fit_hobby_open`] com o curl de ponta explícito (`ω`; `0` = relaxado). Valores maiores
/// dobram a spline mais depressa rumo à direção da primeira/última corda.
#[must_use]
pub fn fit_hobby_open_with_curl(knots: &[[f64; 2]], curl: f64) -> Vec<VecVertex> {
    let tan = tangents(knots, curl);
    if tan.is_empty() {
        return Vec::new();
    }
    let n = tan.len();
    (0..=n)
        .map(|i| {
            let a = knots[i];
            // O handle de SAÍDA é a partida do segmento `i`; o de ENTRADA é a chegada do
            // segmento `i-1`, e a convenção do solver dá os dois como offsets a partir da
            // âncora do nó a que pertencem (`c₀ = start + out`, `c₁ = end + in`).
            let out = if i < n { add(a, tan[i].0) } else { a };
            let inh = if i > 0 { add(a, tan[i - 1].1) } else { a };
            VecVertex::smooth(a, inh, out)
        })
        .collect()
}

/// `(out_at_start, in_at_end)` por segmento, como OFFSETS a partir da âncora de cada ponta —
/// a forma em que o solver de Hobby naturalmente os produz. Vazio para menos de 2 nós.
fn tangents(knots: &[[f64; 2]], curl: f64) -> Vec<([f64; 2], [f64; 2])> {
    let count = knots.len();
    if count < 2 {
        return Vec::new();
    }
    // n = número de segmentos; os índices de nó correm 0..=n.
    let n = count - 1;
    let omega = curl;

    // Entrada não-finita envenena todo `<`/`atan2` a jusante: cai numa cadeia reta (finita, com
    // as pontas honradas) em vez de propagar NaN pela geometria do documento.
    if knots
        .iter()
        .any(|k| !(k[0].is_finite() && k[1].is_finite()))
    {
        return straight_thirds(knots);
    }

    // Cordas + comprimentos (um par por segmento). Um par coincidente pousa no MIN_CHORD para os
    // pesos do solver, então nunca se divide por zero.
    let chords: Vec<[f64; 2]> = (0..n)
        .map(|i| [knots[i + 1][0] - knots[i][0], knots[i + 1][1] - knots[i][1]])
        .collect();
    let d: Vec<f64> = chords.iter().map(|c| len(*c).max(MIN_CHORD)).collect();

    // Ângulos de virada γ em cada nó. γ₀ e γₙ são sentinelas de fronteira (0); o γᵢ interior é o
    // ângulo com sinal da corda i−1 para a corda i.
    let mut gamma = vec![0.0_f64; n + 1];
    for i in 1..n {
        gamma[i] = turning_angle(chords[i - 1], chords[i]);
    }

    // Sistema tridiagonal para α₀..αₙ (sub, diag, sup, rhs).
    let mut sub = vec![0.0_f64; n + 1];
    let mut diag = vec![0.0_f64; n + 1];
    let mut sup = vec![0.0_f64; n + 1];
    let mut rhs = vec![0.0_f64; n + 1];

    // Linha 0 — a fronteira de curl do começo.
    diag[0] = 2.0 + omega;
    sup[0] = 2.0 * omega + 1.0;
    rhs[0] = -sup[0] * gamma[1];

    // Linhas interiores 1..=n-1.
    for i in 1..n {
        sub[i] = 1.0 / d[i - 1];
        diag[i] = 2.0 / d[i - 1] + 2.0 / d[i];
        sup[i] = 1.0 / d[i];
        rhs[i] = -2.0 * gamma[i] / d[i - 1] - gamma[i + 1] / d[i];
    }

    // Linha n — a fronteira de curl do fim.
    sub[n] = 2.0 * omega + 1.0;
    diag[n] = 2.0 + omega;
    rhs[n] = 0.0;

    let alpha = thomas(&sub, &diag, &sup, &rhs);

    // Ângulos de chegada β (um por segmento) a partir dos de partida. **É esta linha que dá a
    // tangente contínua**: o β do segmento i é o negativo da virada mais o α do nó seguinte.
    let mut beta = vec![0.0_f64; n];
    for i in 0..n - 1 {
        beta[i] = -gamma[i + 1] - alpha[i + 1];
    }
    beta[n - 1] = -alpha[n];

    // Os vetores de handle, por segmento.
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = velocity(alpha[i], beta[i]) * d[i] / 3.0;
        let b = velocity(beta[i], alpha[i]) * d[i] / 3.0;
        let out_dir = normalize(rotate(chords[i], alpha[i]));
        let in_dir = normalize(rotate(chords[i], -beta[i]));
        out.push((
            [out_dir[0] * a, out_dir[1] * a],
            [-in_dir[0] * b, -in_dir[1] * b],
        ));
    }
    out
}

/// Cúbica reta em cada segmento: handles nos terços da corda. O fallback finito para entrada
/// não-finita (espelho do que o solver original faz).
fn straight_thirds(knots: &[[f64; 2]]) -> Vec<([f64; 2], [f64; 2])> {
    let n = knots.len().saturating_sub(1);
    (0..n)
        .map(|i| {
            let chord = [knots[i + 1][0] - knots[i][0], knots[i + 1][1] - knots[i][1]];
            let third = if chord[0].is_finite() && chord[1].is_finite() {
                [chord[0] / 3.0, chord[1] / 3.0]
            } else {
                [0.0, 0.0]
            };
            (third, [-third[0], -third[1]])
        })
        .collect()
}

/// Velocidade de Jackowski `ρ(α, β) = 2 / (1 + ⅔·cos β + ⅓·cos α)`, capada em
/// [`MAX_VELOCITY`] e com piso 0. O denominador é afastado do zero para que uma quase-cúspide
/// (`α ≈ β ≈ π`) dê um handle limitado em vez de `∞`.
#[inline]
fn velocity(alpha: f64, beta: f64) -> f64 {
    const C: f64 = 2.0 / 3.0;
    let denom = 1.0 + C * beta.cos() + (1.0 - C) * alpha.cos();
    // denom ∈ [0, 2]; segura a ponta pequena para ρ ficar finito.
    let denom = denom.max(1e-6);
    (2.0 / denom).clamp(0.0, MAX_VELOCITY)
}

/// Ângulo de virada com sinal de `prev` para `cur` (CCW positivo), em radianos. `atan2(cross,
/// dot)` é robusto para qualquer par não-nulo e devolve 0 para cordas paralelas.
#[inline]
fn turning_angle(prev: [f64; 2], cur: [f64; 2]) -> f64 {
    let cross = prev[0] * cur[1] - prev[1] * cur[0];
    let dot = prev[0] * cur[0] + prev[1] * cur[1];
    cross.atan2(dot)
}

/// Roda um vetor 2D por `ang` radianos (CCW positivo).
#[inline]
fn rotate(v: [f64; 2], ang: f64) -> [f64; 2] {
    let (s, c) = ang.sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c]
}

/// Normaliza; devolve `[0, 0]` para um vetor (quase) nulo.
#[inline]
fn normalize(v: [f64; 2]) -> [f64; 2] {
    let m = len(v);
    if m > 1e-12 {
        [v[0] / m, v[1] / m]
    } else {
        [0.0, 0.0]
    }
}

#[inline]
fn len(v: [f64; 2]) -> f64 {
    // `sqrt` é corretamente arredondado (HR-5); `f64::hypot` é libm da plataforma.
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}

#[inline]
fn add(p: [f64; 2], v: [f64; 2]) -> [f64; 2] {
    [p[0] + v[0], p[1] + v[1]]
}

/// Algoritmo de Thomas para um sistema tridiagonal (`sub`, `diag`, `sup`, `rhs` todos de
/// comprimento `m`; `sub[0]` e `sup[m-1]` são ignorados). Estável sem pivotar para a matriz de
/// Hobby, que é diagonalmente dominante; um épsilon defensivo mantém um pivô degenerado finito.
fn thomas(sub: &[f64], diag: &[f64], sup: &[f64], rhs: &[f64]) -> Vec<f64> {
    let m = diag.len();
    let mut c_prime = vec![0.0_f64; m];
    let mut d_prime = vec![0.0_f64; m];

    let mut denom = diag[0];
    if denom.abs() < 1e-12 {
        denom = denom.signum().max(1.0) * 1e-12;
    }
    c_prime[0] = sup[0] / denom;
    d_prime[0] = rhs[0] / denom;

    for i in 1..m {
        let mut den = diag[i] - sub[i] * c_prime[i - 1];
        if den.abs() < 1e-12 {
            den = if den < 0.0 { -1e-12 } else { 1e-12 };
        }
        c_prime[i] = sup[i] / den;
        d_prime[i] = (rhs[i] - sub[i] * d_prime[i - 1]) / den;
    }

    let mut x = vec![0.0_f64; m];
    x[m - 1] = d_prime[m - 1];
    for i in (0..m - 1).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }
    x
}

#[cfg(test)]
#[path = "hobby_tests.rs"]
mod tests;
