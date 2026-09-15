//! ⭐⭐⭐ **A CURVATURA DA ARTE, LEVADA AO REFERENCIAL DA PEGADA** — irmão do [`super::canvas_warp`]
//! por RESPONSABILIDADE: ali mora *que elipse pintar*, aqui *como a dobra que sobra atravessa a
//! decomposição sem se perder pelo caminho*.
//!
//! # A conta, de uma ponta à outra
//!
//! O que o artista pede é um disco de raio `R` **no ecrã**, filtrado pela elipse autorada `E`. O que
//! a malha faz sobre a vizinhança do dab é
//!
//! ```text
//! ecrã(d) = R_f · [ L·u + K(u) ],    u = d / R_f
//! ```
//!
//! com `L` a deformação local (adimensional, identidade em repouso) e `K` os termos de grau `2` e
//! `3` ([`super::CanvasWarp::curve`]). O amostrador recebe `p` = offset ÷ raio **EMITIDO**, e o raio
//! emitido é `R·s₁`, logo `u = s₁·p` e o raio do footprint **CANCELA**:
//!
//! ```text
//! G(p) = s₁·E⁻¹·L·p  +  E⁻¹·K(s₁·p)
//! ```
//!
//! ⭐ **A parte linear de `G` é exactamente `s₁·A⁻¹`** (com `A = L⁻¹E`, a elipse que o
//! [`super::canvas_warp::warped_dab`] já decompõe) — é a mesma conta de sempre, agora escrita.
//!
//! # ⚠️ Porque é preciso o `V`, e porque ele NÃO é decoração
//!
//! A decomposição de hoje entrega `(raio, flatten, ângulo)`, de que o motor reconstrói
//! `Lᵈᵃᵇ = diag(1, 1/(1−f))·R(−θ)`. Ora `Lᵈᵃᵇ` e a parte linear de `G` descrevem a **mesma elipse**
//! e **não são a mesma matriz**: diferem por uma rotação `V` à ESQUERDA, que a norma não vê
//! (`|V·x| = |x|`) — e é por isso que ela nunca incomodou ninguém enquanto a pegada foi linear.
//!
//! ⛔⛔ **Com curvatura ela passa a incomodar:** somar `K` cru a `Lᵈᵃᵇ` seria somar um termo escrito
//! num referencial ao termo de outro, e o erro cresce com a dobra — exactamente onde esta wave
//! existe para acertar. ⇒ a curvatura entra **rodada de volta** por `Vᵀ`.
//!
//! ⚠️ **E o `V` sai da decomposição QUANTIZADA** (o ângulo é um inteiro de grau, que escolhe uma
//! entrada da tabela cozida): assim a curvatura absorve também o erro do arredondamento, em vez de
//! o herdar.
//!
//! # ⚠️ O argumento é o ponto JÁ RODADO
//!
//! A [`super::FootprintCurve`] avalia os monómios em `r = R(−θ)·p` — é isso que deixa o
//! `rotated_by` do dab ser uma composição de rotores sem mexer num coeficiente. Logo a entrada da
//! curvatura é pré-composta com `s₁·R(θ)`, que é a conta que a [`compoe`] faz.

use crate::FootprintCurve;

/// Uma matriz `2×2` por linhas.
type M2 = [[f32; 2]; 2];

fn mul(a: M2, b: M2) -> M2 {
    [
        [
            a[0][0] * b[0][0] + a[0][1] * b[1][0],
            a[0][0] * b[0][1] + a[0][1] * b[1][1],
        ],
        [
            a[1][0] * b[0][0] + a[1][1] * b[1][0],
            a[1][0] * b[0][1] + a[1][1] * b[1][1],
        ],
    ]
}

/// ⭐⭐⭐ **A PORTA**: os coeficientes que a [`crate::FootprintDeform`] vai carregar.
///
/// - `curve` — os graus `2` e `3` da malha, em `u` (offset ÷ raio do footprint).
/// - `e_inv` — o avesso da elipse autorada.
/// - `v_t` — a transposta da rotação que separa a parte linear de `G` da que o motor reconstrói.
/// - `s1` — o `radius_scale` já decidido.
/// - `rot` — o rotor `[cos θ, sin θ]` do ângulo QUANTIZADO do dab.
#[must_use]
pub(crate) fn compoe(
    curve: [[f32; 7]; 2],
    e_inv: M2,
    v_t: M2,
    s1: f32,
    rot: [f32; 2],
) -> FootprintCurve {
    // A entrada: `u = s₁·R(θ)·r`. As colunas de `R(θ)` são `(c, s)` e `(−s, c)`.
    let (a, b) = (s1 * rot[0], -s1 * rot[1]);
    let (c, d) = (s1 * rot[1], s1 * rot[0]);
    // Cada monómio de `u` reescrito nos monómios de `r`.
    // Quadráticos: colunas `r0²`, `r0·r1`, `r1²`.
    let q: [[f32; 3]; 3] = [
        [a * a, 2.0 * a * b, b * b],
        [a * c, a * d + b * c, b * d],
        [c * c, 2.0 * c * d, d * d],
    ];
    // Cúbicos: colunas `r0³`, `r0²·r1`, `r0·r1²`, `r1³`.
    let k: [[f32; 4]; 4] = [
        [a * a * a, 3.0 * a * a * b, 3.0 * a * b * b, b * b * b],
        [
            a * a * c,
            a * a * d + 2.0 * a * b * c,
            2.0 * a * b * d + b * b * c,
            b * b * d,
        ],
        [
            a * c * c,
            2.0 * a * c * d + b * c * c,
            a * d * d + 2.0 * b * c * d,
            b * d * d,
        ],
        [c * c * c, 3.0 * c * c * d, 3.0 * c * d * d, d * d * d],
    ];
    // A saída: `Vᵀ·E⁻¹` aplicado ao vector de coeficientes de cada monómio.
    let saida = mul(v_t, e_inv);
    let mut rows = [[0.0f32; 7]; 2];
    for (col, linha) in q.iter().enumerate() {
        for (alvo, peso) in linha.iter().enumerate() {
            let v = [curve[0][col] * peso, curve[1][col] * peso];
            rows[0][alvo] += saida[0][0] * v[0] + saida[0][1] * v[1];
            rows[1][alvo] += saida[1][0] * v[0] + saida[1][1] * v[1];
        }
    }
    for (col, linha) in k.iter().enumerate() {
        for (alvo, peso) in linha.iter().enumerate() {
            let v = [curve[0][3 + col] * peso, curve[1][3 + col] * peso];
            rows[0][3 + alvo] += saida[0][0] * v[0] + saida[0][1] * v[1];
            rows[1][3 + alvo] += saida[1][0] * v[0] + saida[1][1] * v[1];
        }
    }
    FootprintCurve::from_rows(rows)
}

/// O avesso de uma `2×2`, ou `None` se ela for degenerada.
#[must_use]
pub(crate) fn inverte(m: M2) -> Option<M2> {
    let det = m[0][0] * m[1][1] - m[0][1] * m[1][0];
    (det.is_finite() && det.abs() > 1e-12).then(|| {
        [
            [m[1][1] / det, -m[0][1] / det],
            [-m[1][0] / det, m[0][0] / det],
        ]
    })
}

/// `V = G₁ · (Lᵈᵃᵇ)⁻¹`, transposta — ver o cabeçalho. `g1` é a parte linear de `G`; `rot` o rotor do
/// ângulo quantizado e `inv_minor` o `1/(1−flatten)` que o motor vai reconstruir.
#[must_use]
pub(crate) fn v_transposta(g1: M2, rot: [f32; 2], inv_minor: f32) -> M2 {
    // `Lᵈᵃᵇ = [[c, s], [−s·im, c·im]]` ⇒ `(Lᵈᵃᵇ)⁻¹ = [[c, −s/im], [s, c/im]]`.
    let (c, s) = (rot[0], rot[1]);
    let l_inv = [[c, -s / inv_minor], [s, c / inv_minor]];
    let v = mul(g1, l_inv);
    [[v[0][0], v[1][0]], [v[0][1], v[1][1]]]
}

/// `a · b`, exposto ao irmão.
#[must_use]
pub(crate) fn produto(a: M2, b: M2) -> M2 {
    mul(a, b)
}
