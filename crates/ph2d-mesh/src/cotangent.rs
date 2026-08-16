//! **A NORMAL DE CURVATURA MÉDIA POR COTANGENTES** — o operador de
//! Laplace-Beltrami discreto de Meyer, Desbrun, Schröder & Barr 2003
//! (*Discrete Differential-Geometry Operators for Triangulated 2-Manifolds*).
//!
//! ```text
//! K(x_i) = (1 / (2·A_mixed)) · Σ_j (cot α_ij + cot β_ij) · (x_i − x_j) = 2·κ_H·n
//! ```
//!
//! `α_ij` e `β_ij` são os dois ângulos **OPOSTOS** à aresta `(i,j)`, um em cada
//! triângulo que a compartilha; `A_mixed` é a área de Voronoi mista da §3.3 do
//! paper. O vetor resultante carrega as duas metades de uma vez: a **direção** é
//! a normal e o **módulo** é `2|κ_H|`.
//!
//! # Por que ele existe, quando já há uma normal e já há uma curvatura
//!
//! As duas que já existem são estimadores por **contagem**, e o preço delas está
//! escrito nos próprios docs:
//!
//! - [`crate::normals`] soma as normais das faces do anel **sem peso** e o
//!   doc-comment dele já nomeava a dívida (*"ponderar por ÁREA seria estritamente
//!   melhor em malha irregular … fica nomeado e não feito"*);
//! - [`crate::curvature`] projeta o laplaciano **uniforme** na normal, e o
//!   doc-comment dele nomeia exatamente esta função como a alternativa
//!   geométrica (*"para uma medida geométrica seria o cotangente de Meyer et al.
//!   2003 — que custa dois ângulos por aresta e uma área mista por vértice"*).
//!
//! ⚠️ **A objeção que ISENTA a curvatura NÃO isenta a normal**, e é essa
//! assimetria que faz este módulo existir. Lá a parte dependente da distribuição
//! é **tangencial** e a projeção na normal a joga fora de graça (num plano
//! irregular todo vizinho está no plano, logo o produto interno é exatamente
//! zero). Aqui a grandeza procurada **É** a direção — não há para onde projetar
//! o erro.
//!
//! # O que ele NÃO responde, e por que isso é uma AFIRMAÇÃO
//!
//! Um vértice de **BORDA** devolve [`None`]. A construção do paper precisa dos
//! **dois** ângulos opostos a cada aresta, e uma aresta de beira só tem um
//! triângulo, logo só tem um. Inventar o que falta seria pôr um número onde a
//! fonte não tem nenhum — a regra do §4 do plano —, e o custo de não o fazer é
//! zero: o chamador cai na normal que já shipa, exatamente como o
//! [`crate::smooth::ring_average`] restringe a média à própria borda em vez de
//! adivinhar o miolo.
//!
//! Também devolve [`None`] onde a resposta não existe: anel vazio, área mista
//! nula (todo triângulo degenerado) e resultado não-finito.
//!
//! # ⛔ O `l-mode` do INFLATE foi RECUSADO por medição, e isto não é ele
//!
//! O §3 do plano `docs/3D/21` dá ao **Inflate** a *normal de curvatura média* como
//! direção. Medido (`ph2d-sculpt3d`, `tests/measure_curvature_normal.rs`), **não
//! ship** — e as duas leituras recusam por razões diferentes:
//!
//! | fixture | côncavos | eixo médio | eixo p95 |
//! |---|---|---|---|
//! | `sculpt_sphere` (a malha DEFAULT) | 0,0 % | **0,003°** | 0,020° |
//! | `uv_sphere` 24×32 | 0,0 % | 0,213° | 0,933° |
//! | depois de 4 traços | 3,3 % | **0,709°** | 2,030° |
//! | `uv_sphere_shuffled` | 12,6 % | 26,3° | **87,9°** |
//!
//! **(1)** Na malha que o artista de facto tem o eixo diverge **três milésimos de
//! grau** da normal que já shipa — um chip que não move um pixel. **(2)** Onde
//! ele diverge, diverge **por deixar de ser uma normal**: `p95 = 87,9°` é quase
//! TANGENTE, porque numa malha ruidosa o vetor de curvatura segue a ruga.
//! **(3)** E `K = 2·κ_H·n` carrega a curvatura **com sinal**: numa cova ele
//! aponta para DENTRO, então caminhar por ele não é *inflar*, é **afiar** — que é
//! outro verbo, e que já tem `l-mode` próprio (o μ do Taubin).
//!
//! ⇒ O que este módulo serve é a outra metade da mesma célula do §4: *"o
//! operador dos dois acima"*, isto é, o laplaciano sobre o qual o par λ|μ corre.
//!
//! # Sem transcendental, e não por gosto
//!
//! `cot θ = cos θ / sin θ = (u·v) / |u × v|`, então os ângulos **nunca são
//! materializados**: não há `acos`, não há `atan2`, e a única raiz é a do
//! produto vetorial — que a área do triângulo já precisa. É a mesma disciplina
//! que o `libm` cobra nas crates cujo número atravessa um hash, e aqui ela sai
//! de graça da álgebra.
//!
//! # Paralelismo
//!
//! Mesma forma das normais e da curvatura, pelo mesmo motivo: **gather** sobre
//! CSR de ordem fixa, cada vértice escreve só o seu, e a ordem da soma dentro de
//! um vértice não muda com o escalonamento ⇒ byte-idêntico por construção, o que
//! o ADR-0109 exige. Reusa o [`crate::normals::PAR_MIN`] — mesma varredura,
//! mesma estrutura, mesmo laço.

use rayon::prelude::*;

use crate::adjacency::Adjacency;
use crate::face::Face;
use crate::normals::PAR_MIN;

/// `cot` do ângulo entre `u` e `v`, **sem materializar o ângulo**.
///
/// Devolve `None` num canto degenerado (`|u × v|` nulo), que é onde a cotangente
/// diverge — e é exatamente o triângulo que não tem opinião sobre curvatura
/// nenhuma.
fn cot(u: [f32; 3], v: [f32; 3]) -> Option<(f32, f32)> {
    let dot = u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let cross = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len2 = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
    if len2 <= f32::MIN_POSITIVE {
        return None;
    }
    // `|u × v|` é DUAS vezes a área do triângulo — a segunda metade do par sai
    // desta mesma raiz, e é por isso que ela é devolvida em vez de recomputada
    // por quem precisa da área.
    let sin = len2.sqrt();
    Some((dot / sin, 0.5 * sin))
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn len2(v: [f32; 3]) -> f32 {
    v[0] * v[0] + v[1] * v[1] + v[2] * v[2]
}

/// **O que UMA travessia do anel colhe** — as três somas de Meyer, juntas.
///
/// ⚠️ **Uma travessia, não três, e é a mesma razão do
/// [`crate::curvature_pair_at`]:** as três grandezas saem dos MESMOS cotangentes
/// sobre os MESMOS triângulos, e recolhê-las em passes separados seria três
/// respostas a *"como é o anel deste vértice?"*, que divergem no dia em que
/// alguém mexer no ramo obtuso.
#[derive(Clone, Copy, Debug)]
pub struct RingWeights {
    /// `Σ_j (cot α + cot β) · (x_i − x_j)` — o numerador do operador.
    pub flow: [f32; 3],
    /// `Σ_j (cot α + cot β)` — o denominador de uma MÉDIA ponderada.
    ///
    /// ⚠️ **É POSITIVO por identidade, e eu escrevi o contrário primeiro.** A
    /// fama do operador é que *"uma cotangente é negativa num ângulo obtuso,
    /// logo um anel mal-condicionado soma para trás"* — a primeira metade é
    /// verdade e a segunda **não segue**. Cada triângulo contribui com o PAR
    /// `cot q + cot r`, e
    ///
    /// ```text
    /// cot q + cot r = sin(q + r) / (sin q · sin r) = sin p / (sin q · sin r) > 0
    /// ```
    ///
    /// para todo triângulo não-degenerado. Um triângulo tem no máximo **um**
    /// ângulo obtuso, então o par nunca é dominado por ele. Gate:
    /// `the_weight_sum_is_positive_by_identity_not_by_luck`.
    ///
    /// ⚠️ **A instabilidade real do operador não mora aqui, mora nos pesos
    /// INDIVIDUAIS** (`cot α + cot β` de UMA aresta, que fica negativo quando os
    /// dois ângulos opostos são obtusos) — quem os usa um a um, e não a soma, é
    /// que tem de os encarar.
    pub weight: f32,
    /// A área de Voronoi mista, §3.3 do paper.
    pub area: f32,
}

/// **A colheita crua** — as três somas de [`RingWeights`] numa travessia.
///
/// [`None`] onde a construção não tem resposta: **borda**, anel vazio, canto
/// degenerado.
#[must_use]
pub fn ring_weights_at(
    positions: &[[f32; 3]],
    faces: &[Face],
    adj: &Adjacency,
    v: usize,
) -> Option<RingWeights> {
    // A beira não tem os dois ângulos que a fórmula pede — ver o módulo.
    if adj.is_border(v) {
        return None;
    }
    let ring = adj.vert_faces.neighbours(v);
    if ring.is_empty() {
        return None;
    }
    let vi = v as u32;
    let p = positions[v];
    let mut acc = [0.0f32; 3];
    let mut weight = 0.0f32;
    let mut area = 0.0f32;

    for &fi in ring {
        let face = faces[fi as usize];
        for t in 0..face.tri_count() {
            let tri = face.tri_at(t);
            // Um quad toca `v` com UM dos seus dois triângulos quando `v` é `b`
            // ou `d`; com os DOIS quando é `a` ou `c`. Filtrar aqui é o que
            // mantém a soma sobre os triângulos que de facto rodeiam `v`.
            let Some(k) = tri.iter().position(|&x| x == vi) else {
                continue;
            };
            let q = positions[tri[(k + 1) % 3] as usize];
            let r = positions[tri[(k + 2) % 3] as usize];

            // Os três ângulos do triângulo, cada um pelo par de arestas que sai
            // do seu próprio canto.
            let (cot_p, tri_area) = cot(sub(q, p), sub(r, p))?;
            let (cot_q, _) = cot(sub(p, q), sub(r, q))?;
            let (cot_r, _) = cot(sub(p, r), sub(q, r))?;

            // O ângulo em `r` é oposto à aresta `(p,q)`; o em `q`, à `(p,r)`.
            // Somado sobre os dois triângulos de cada aresta, isto É o
            // `cot α + cot β` do paper.
            let dq = sub(p, q);
            let dr = sub(p, r);
            for c in 0..3 {
                acc[c] += cot_r * dq[c] + cot_q * dr[c];
            }
            weight += cot_r + cot_q;

            // A área MISTA, §3.3 do paper: Voronoi onde o triângulo é acutângulo,
            // e a partição por metade/quarto onde ele é obtuso — que é onde a
            // região de Voronoi sairia do próprio triângulo.
            //
            // ⚠️ **O teste de obtusidade é o SINAL da cotangente**, não um
            // ângulo: `cot θ < 0` ⟺ `θ > 90°`. Materializar o ângulo para
            // compará-lo com π/2 seria pagar um transcendental para responder o
            // que o sinal já responde.
            area += if cot_p < 0.0 {
                tri_area * 0.5
            } else if cot_q < 0.0 || cot_r < 0.0 {
                tri_area * 0.25
            } else {
                0.125 * (len2(dr) * cot_q + len2(dq) * cot_r)
            };
        }
    }

    if area <= f32::MIN_POSITIVE {
        return None;
    }
    let w = RingWeights {
        flow: acc,
        weight,
        area,
    };
    (w.flow.iter().all(|c| c.is_finite()) && weight.is_finite()).then_some(w)
}

/// **A normal de curvatura média de um vértice** — `K = 2·κ_H·n`, ver o módulo.
///
/// [`None`] pelas razões do [`ring_weights_at`].
///
/// ⚠️ **O SINAL segue a convenção do paper e é MEDIDO, não afirmado:** numa
/// esfera de raio `R` com normais para fora, `K` aponta **para fora** e mede
/// `2/R` (gate `the_operator_measures_the_curvature_of_a_sphere`). Quem quiser a
/// direção normaliza; quem quiser a curvatura toma metade do módulo.
///
/// ⚠️ **E o sinal é a CURVATURA, não uma escolha de orientação:** `κ_H` é
/// assinado, então numa COVA o `K` aponta para DENTRO. Ele não é um estimador de
/// normal — ver a §*"o l-mode do Inflate foi recusado"* do módulo.
#[must_use]
pub fn mean_curvature_normal_at(
    positions: &[[f32; 3]],
    faces: &[Face],
    adj: &Adjacency,
    v: usize,
) -> Option<[f32; 3]> {
    let w = ring_weights_at(positions, faces, adj, v)?;
    let inv = 1.0 / (2.0 * w.area);
    let out = [w.flow[0] * inv, w.flow[1] * inv, w.flow[2] * inv];
    out.iter().all(|c| c.is_finite()).then_some(out)
}

/// **A MÉDIA DO ANEL PONDERADA POR COTANGENTES** — o alvo que o `l-mode` do
/// Smooth persegue, e o irmão geométrico do [`crate::smooth::ring_average`].
///
/// ```text
/// alvo = x_i − (Σ w_ij (x_i − x_j)) / (Σ w_ij)
/// ```
///
/// ⚠️ **A guarda do divisor é PROVAVELMENTE INALCANÇÁVEL, e isso está medido e
/// provado em vez de suposto.** A mutação que a apaga **sobreviveu** à suíte
/// inteira, e a explicação honesta não é *"falta fixture"*: `Σ w` é positivo por
/// identidade (ver [`RingWeights::weight`]), varrido de razão de aspecto 1 a
/// 1000 com **zero** ocorrências. Ela fica como leitura do divisor — o piso
/// RELATIVO ainda apanha cancelamento numérico que a álgebra não cobre — e o
/// gate `the_weight_sum_is_positive_by_identity_not_by_luck` é o que nomeia o
/// que se perde no dia em que alguém trocar a acumulação.
#[must_use]
pub fn cotangent_ring_average_at(
    positions: &[[f32; 3]],
    faces: &[Face],
    adj: &Adjacency,
    v: usize,
) -> Option<[f32; 3]> {
    let w = ring_weights_at(positions, faces, adj, v)?;
    // O piso é RELATIVO à escala dos próprios pesos: um `Σw` minúsculo ao lado
    // de termos grandes é cancelamento, e dividir por ele amplifica o ruído.
    let scale = w.flow.iter().fold(0.0f32, |a, c| a + c.abs());
    if w.weight <= f32::MIN_POSITIVE || w.weight * 64.0 < scale {
        return None;
    }
    let inv = 1.0 / w.weight;
    let p = positions[v];
    let out = [
        p[0] - w.flow[0] * inv,
        p[1] - w.flow[1] * inv,
        p[2] - w.flow[2] * inv,
    ];
    out.iter().all(|c| c.is_finite()).then_some(out)
}

/// **A DIREÇÃO** — a normal de curvatura média, unitária.
///
/// [`None`] pelas razões do módulo **mais uma**: uma superfície localmente PLANA
/// tem `K = 0`, e um plano não tem normal de curvatura. É o chamador que decide
/// o que fazer com isso, e a resposta honesta dele é a normal que já shipa: uma
/// região chata **tem** normal, ela só não tem *curvatura* que a aponte.
#[must_use]
pub fn curvature_normal_dir_at(
    positions: &[[f32; 3]],
    faces: &[Face],
    adj: &Adjacency,
    v: usize,
) -> Option<[f32; 3]> {
    let k = mean_curvature_normal_at(positions, faces, adj, v)?;
    let l2 = len2(k);
    if l2 <= f32::MIN_POSITIVE {
        return None;
    }
    let inv = 1.0 / l2.sqrt();
    Some([k[0] * inv, k[1] * inv, k[2] * inv])
}

/// As normais de curvatura dos vértices listados, **na ordem de `which`** — o
/// chamador espalha. Irmã da [`crate::normals::vertex_normals_of`], pelo mesmo
/// motivo: escrever direto nos índices esparsos seria escrita concorrente sobre
/// o mesmo `Vec`.
pub fn curvature_normals_of(
    positions: &[[f32; 3]],
    faces: &[Face],
    adj: &Adjacency,
    which: &[u32],
    out: &mut Vec<Option<[f32; 3]>>,
) {
    out.clear();
    out.resize(which.len(), None);
    if which.len() < PAR_MIN {
        for (o, &v) in out.iter_mut().zip(which) {
            *o = curvature_normal_dir_at(positions, faces, adj, v as usize);
        }
    } else {
        out.par_iter_mut()
            .zip(which.par_iter())
            .for_each(|(o, &v)| *o = curvature_normal_dir_at(positions, faces, adj, v as usize));
    }
}

#[cfg(test)]
#[path = "cotangent_tests.rs"]
mod cotangent_tests;
