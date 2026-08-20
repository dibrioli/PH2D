//! **O CAMPO DE POSIÇÃO** — o segundo dos dois campos do ADR-0160.
//!
//! O campo de orientação diz **para onde** a grade corre; este diz **onde ela
//! está**. Cada vértice guarda um ponto `o_i` na superfície: a origem da retícula
//! local. Os vértices da malha nova são os pontos dessa retícula —
//! `o_i + s·(a·q_i + b·(n_i×q_i))` para inteiros `a`, `b` —, e é por isso que o
//! campo tem a **mesma simetria de 4 dobras** do de orientação: mover `o_i` por
//! um passo inteiro da retícula descreve **a mesma grade**.
//!
//! ⚠️ **É essa simetria que torna a suavização possível.** Somar `o_i + o_j`
//! direto somaria dois pontos que podem estar a células inteiras de distância, e
//! a média cairia no meio do nada. O que se soma são os representantes
//! **reduzidos ao mesmo ponto de referência**, que é o que a
//! [`position_round_4`] faz.
//!
//! Porte de `compat_position_extrinsic_4` / `position_round_4` / `middle_point`
//! (`instant-meshes`, `src/field.cpp`), BSD-3-Clause.

use ph2d_mesh::Mesh;

use crate::orientation::OrientationField;
use crate::scale::ScaleField;

/// **O campo, um ponto de retícula por vértice.**
#[derive(Clone, Debug, PartialEq)]
pub struct PositionField {
    pos: Vec<[f32; 3]>,
}

impl PositionField {
    /// A origem da retícula do vértice `v`.
    #[must_use]
    pub fn at(&self, v: usize) -> [f32; 3] {
        self.pos[v]
    }

    /// Quantos vértices o campo cobre.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pos.len()
    }

    /// Um campo sem vértices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }

    /// **A ENERGIA do campo** — a soma, sobre as arestas dirigidas, da distância
    /// entre os dois representantes reduzidos, **em unidades de célula**.
    ///
    /// ⚠️ **Em unidades de CÉLULA e não de objeto**, e é o que a torna
    /// comparável entre um modo uniforme e um adaptativo: uma malha de quads
    /// grandes teria energia grande em metros sem estar pior.
    #[must_use]
    pub fn energy(&self, mesh: &Mesh, orient: &OrientationField, scale: &ScaleField) -> f64 {
        let (p, n) = (mesh.positions(), mesh.normals());
        let adj = mesh.adjacency();
        let mut sum = 0.0f64;
        for v in 0..self.pos.len() {
            for &w in adj.vert_verts.neighbours(v) {
                let w = w as usize;
                let (a, b) = compat_position_extrinsic_4(
                    p[v],
                    n[v],
                    orient.dir(v),
                    self.pos[v],
                    scale.at(v),
                    p[w],
                    n[w],
                    orient.dir(w),
                    self.pos[w],
                    scale.at(w),
                );
                let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                let cell = scale.at(v).max(crate::scale::MIN_EDGE);
                sum += f64::from(norm(d) / cell);
            }
        }
        sum
    }
}

/// **RESOLVE o campo de posição** — `iterations` varreduras de suavização.
///
/// Mesma estrutura do [`crate::orientation::solve_orientation`]: Gauss-Seidel,
/// ordem de visita fixa, determinístico.
///
/// ⚠️ **A semente é o PRÓPRIO VÉRTICE.** Um campo de posição semeado longe da
/// superfície teria de caminhar de volta antes de começar a alinhar-se, e o
/// vértice é o ponto da retícula mais honesto que existe antes de qualquer
/// suavização: *a grade passa por onde a malha está*.
#[must_use]
pub fn solve_position(
    mesh: &Mesh,
    orient: &OrientationField,
    scale: &ScaleField,
    iterations: usize,
) -> PositionField {
    let (p, n) = (mesh.positions(), mesh.normals());
    let adj = crate::im_weights::cotangent_adjacency(mesh);
    let dirs: Vec<[f32; 3]> = (0..mesh.vert_count()).map(|v| orient.dir(v)).collect();
    let scales: Vec<f32> = (0..mesh.vert_count()).map(|v| scale.at(v)).collect();
    let mut pos: Vec<[f32; 3]> = p.to_vec();
    smooth_on(&mut pos, p, n, &dirs, &scales, &adj, iterations);
    PositionField { pos }
}

/// **O CAMPO, a partir de origens já resolvidas** — a porta da hierarquia.
#[must_use]
pub fn field_from(pos: Vec<[f32; 3]>) -> PositionField {
    PositionField { pos }
}

/// **O NÚCLEO da suavização do campo de posição, sobre slices.**
///
/// ⚠️ Irmão do [`crate::orientation::smooth_on`] e pelo mesmo motivo: a
/// hierarquia corre a MESMA lei sobre níveis que não são malhas.
#[allow(clippy::too_many_arguments)]
pub fn smooth_on(
    pos: &mut [[f32; 3]],
    p: &[[f32; 3]],
    n: &[[f32; 3]],
    dirs: &[[f32; 3]],
    scales: &[f32],
    adjacency: &[Vec<crate::im_weights::Link>],
    iterations: usize,
) {
    let count = pos.len();
    for _ in 0..iterations {
        for v in 0..count {
            let (pv, nv, qv, sv) = (p[v], n[v], dirs[v], scales[v]);
            // ⚠️ **`weight_sum` começa em ZERO** — a mesma correção da
            // orientação, e pela mesma razão.
            let mut sum = pos[v];
            let mut weight_sum = 0.0f32;
            for link in &adjacency[v] {
                let w = link.weight;
                if w == 0.0 {
                    continue;
                }
                let j = link.id as usize;
                let (a, b) = compat_position_extrinsic_4(
                    pv, nv, qv, sum, sv, p[j], n[j], dirs[j], pos[j], scales[j],
                );
                sum = [
                    a[0].mul_add(weight_sum, b[0] * w),
                    a[1].mul_add(weight_sum, b[1] * w),
                    a[2].mul_add(weight_sum, b[2] * w),
                ];
                weight_sum += w;
                if weight_sum.abs() > 1.0e-20 {
                    let inv = 1.0 / weight_sum;
                    sum = [sum[0] * inv, sum[1] * inv, sum[2] * inv];
                }
                // ⚠️ **De volta ao plano tangente DO VÉRTICE, a cada vizinho.**
                let d = dot([sum[0] - pv[0], sum[1] - pv[1], sum[2] - pv[2]], nv);
                sum = [
                    d.mul_add(-nv[0], sum[0]),
                    d.mul_add(-nv[1], sum[1]),
                    d.mul_add(-nv[2], sum[2]),
                ];
            }
            // ⚠️ **E o resultado volta À RETÍCULA.** Sem este arredondamento o
            // campo derivaria continuamente e a extração não teria células a que
            // agarrar-se.
            if weight_sum > 0.0 {
                pos[v] = position_round_4(sum, qv, nv, pv, sv);
            }
        }
    }
}

/// **O PONTO DA RETÍCULA de `o` mais próximo de `p`.**
///
/// A retícula é `o + s·(a·q + b·(n×q))`, `a, b` inteiros; esta função devolve o
/// ponto dela mais perto de `p`. É a redução que torna dois campos comparáveis.
#[must_use]
pub fn position_round_4(
    o: [f32; 3],
    q: [f32; 3],
    n: [f32; 3],
    p: [f32; 3],
    scale: f32,
) -> [f32; 3] {
    let s = scale.max(crate::scale::MIN_EDGE);
    let inv = 1.0 / s;
    let t = cross(n, q);
    let d = [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
    let a = (dot(q, d) * inv).round() * s;
    let b = (dot(t, d) * inv).round() * s;
    [
        a.mul_add(q[0], b.mul_add(t[0], o[0])),
        a.mul_add(q[1], b.mul_add(t[1], o[1])),
        a.mul_add(q[2], b.mul_add(t[2], o[2])),
    ]
}

/// **O ÍNDICE INTEIRO da célula de `o` que contém `p`** — o `floor`, não o
/// `round`.
///
/// Porte de `position_floor_index_4` (`instant-meshes`, `src/field.cpp`),
/// BSD-3-Clause.
///
/// ⚠️ **`floor` e não `round`, e a diferença é o que torna a enumeração
/// possível:** o `floor` nomeia a CÉLULA em que o ponto caiu, e as quatro quinas
/// dela são `+0/+1` em cada eixo. Com `round` ter-se-ia o nó mais próximo — um
/// ponto, não uma célula — e não haveria o que enumerar.
#[must_use]
pub fn position_floor_index_4(
    o: [f32; 3],
    q: [f32; 3],
    n: [f32; 3],
    p: [f32; 3],
    scale: f32,
) -> [i32; 2] {
    let s = scale.max(crate::scale::MIN_EDGE);
    let inv = 1.0 / s;
    let t = cross(n, q);
    let d = [p[0] - o[0], p[1] - o[1], p[2] - o[2]];
    [
        (dot(q, d) * inv).floor() as i32,
        (dot(t, d) * inv).floor() as i32,
    ]
}

/// O nó `(a, b)` da retícula de `o`.
fn lattice_node(o: [f32; 3], q: [f32; 3], t: [f32; 3], a: i32, b: i32, s: f32) -> [f32; 3] {
    let (fa, fb) = (a as f32 * s, b as f32 * s);
    [
        fa.mul_add(q[0], fb.mul_add(t[0], o[0])),
        fa.mul_add(q[1], fb.mul_add(t[1], o[1])),
        fa.mul_add(q[2], fb.mul_add(t[2], o[2])),
    ]
}

/// **O PAR DE QUINAS MAIS PRÓXIMAS ENTRE SI**, e os índices inteiros delas.
///
/// Porte de `compat_position_extrinsic_index_4` (`instant-meshes`,
/// `src/field.cpp`), BSD-3-Clause. Devolve `(índice de 0, índice de 1, custo)`.
///
/// ⚠️ **É a peça que a primeira versão desta crate NÃO tinha, e a ausência custou
/// a wave inteira.** Aquela arredondava CADA lado ao ponto médio,
/// independentemente — então cada campo ficava junto do seu próprio vértice, as
/// duas retículas nunca se procuravam, e o campo saía **suave** em vez de
/// escalonado. Sem degraus não há platôs; sem platôs a extração não tem células,
/// e o quociente pela retícula funde o modelo inteiro (medido: 1 célula numa
/// esfera de 3 072 vértices).
///
/// Aqui as **quatro quinas** da célula de cada lado são enumeradas e escolhe-se o
/// **PAR** que minimiza a distância — 16 combinações. É isto que **puxa uma
/// retícula até à outra**: a quina escolhida pode estar a um passo inteiro de
/// distância do nó mais próximo do próprio vértice, e é esse passo que vira o
/// degrau.
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn compat_position_extrinsic_index_4(
    p0: [f32; 3],
    n0: [f32; 3],
    q0: [f32; 3],
    o0: [f32; 3],
    s0: f32,
    p1: [f32; 3],
    n1: [f32; 3],
    q1: [f32; 3],
    o1: [f32; 3],
    s1: f32,
) -> ([i32; 2], [i32; 2], f32) {
    let (t0, t1) = (cross(n0, q0), cross(n1, q1));
    let middle = middle_point(p0, n0, p1, n1);
    let a = position_floor_index_4(o0, q0, n0, middle, s0);
    let b = position_floor_index_4(o1, q1, n1, middle, s1);

    let (mut best, mut bi, mut bj) = (f32::INFINITY, 0i32, 0i32);
    for i in 0..4i32 {
        let p = lattice_node(o0, q0, t0, (i & 1) + a[0], ((i & 2) >> 1) + a[1], s0);
        for j in 0..4i32 {
            let r = lattice_node(o1, q1, t1, (j & 1) + b[0], ((j & 2) >> 1) + b[1], s1);
            let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
            let cost = dot(d, d);
            // ⚠️ `<` estrito: com o empate a primeira vence, e é isso que mantém
            // a função determinística sobre uma malha simétrica.
            if cost < best {
                best = cost;
                bi = i;
                bj = j;
            }
        }
    }
    (
        [(bi & 1) + a[0], ((bi & 2) >> 1) + a[1]],
        [(bj & 1) + b[0], ((bj & 2) >> 1) + b[1]],
        best,
    )
}

/// **OS DOIS REPRESENTANTES de dois campos de posição** — as quinas que o
/// [`compat_position_extrinsic_index_4`] escolheu, em 3D.
///
/// Porte de `compat_position_extrinsic_4` (`instant-meshes`, `src/field.cpp`).
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn compat_position_extrinsic_4(
    p0: [f32; 3],
    n0: [f32; 3],
    q0: [f32; 3],
    o0: [f32; 3],
    s0: f32,
    p1: [f32; 3],
    n1: [f32; 3],
    q1: [f32; 3],
    o1: [f32; 3],
    s1: f32,
) -> ([f32; 3], [f32; 3]) {
    let (ia, ib, _) = compat_position_extrinsic_index_4(p0, n0, q0, o0, s0, p1, n1, q1, o1, s1);
    (
        lattice_node(o0, q0, cross(n0, q0), ia[0], ia[1], s0),
        lattice_node(o1, q1, cross(n1, q1), ib[0], ib[1], s1),
    )
}

/// **O PONTO ENTRE DOIS VÉRTICES QUE RESPEITA OS DOIS PLANOS TANGENTES.**
///
/// Minimiza `‖x − p0‖² + ‖x − p1‖²` sob `n0·x = n0·p0` e `n1·x = n1·p1` — os
/// dois multiplicadores de Lagrange, resolvidos em forma fechada.
///
/// ⚠️ **O `+1e-4` no denominador é uma guarda de RECURSO**, não um fator de
/// gosto: com as duas normais paralelas (`n0·n1 = ±1`) o sistema é singular — as
/// duas restrições são a mesma —, e o termo evita o infinito sem mover a
/// resposta em nenhum caso não-degenerado. É o valor da referência.
#[must_use]
pub fn middle_point(p0: [f32; 3], n0: [f32; 3], p1: [f32; 3], n1: [f32; 3]) -> [f32; 3] {
    let (n0p0, n0p1) = (dot(n0, p0), dot(n0, p1));
    let (n1p0, n1p1) = (dot(n1, p0), dot(n1, p1));
    let n0n1 = dot(n0, n1);
    let denom = 1.0 / n0n1.mul_add(-n0n1, 1.0 + 1.0e-4);
    let lambda0 = 2.0 * n0n1.mul_add(-(n1p0 - n1p1), n0p1 - n0p0) * denom;
    let lambda1 = 2.0 * n0n1.mul_add(-(n0p1 - n0p0), n1p0 - n1p1) * denom;
    [
        0.25f32.mul_add(
            -lambda0.mul_add(n0[0], lambda1 * n1[0]),
            0.5 * (p0[0] + p1[0]),
        ),
        0.25f32.mul_add(
            -lambda0.mul_add(n0[1], lambda1 * n1[1]),
            0.5 * (p0[1] + p1[1]),
        ),
        0.25f32.mul_add(
            -lambda0.mul_add(n0[2], lambda1 * n1[2]),
            0.5 * (p0[2] + p1[2]),
        ),
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

fn norm(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

#[cfg(test)]
#[path = "position_tests.rs"]
mod tests;
