//! **A SUBDIVISÃO** — a resolução deixa de ser fixa.
//!
//! Port de `reference/sculptgl/src/editing/Subdivision.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! Cada triângulo vira **quatro triângulos** e cada quad **quatro quads**: a
//! contagem de faces multiplica por 4 e a de vértices vai a `V + E + Q` (os
//! originais, um por ARESTA, e um por QUAD).
//!
//! # A regra é EXATA, não uma aproximação
//!
//! Os pesos do original são **Loop** para triângulo e **Catmull-Clark** para
//! quad, e isso foi conferido em vez de acreditado — para o vértice interior de
//! valência 4, a fórmula publicada de Catmull-Clark (`(F + 2R + (n−3)V)/n`)
//! expandida nos anéis dá `0,5625·V + 0,09375·Σanel + 0,015625·Σdiagonais`, que
//! são os três literais do `Subdivision.js`. O ponto de aresta de quad
//! (`0,375(v1+v2) + 0,0625·Σ4 diagonais`) é a média `(v1+v2+f1+f2)/4` com os
//! centroides expandidos. Portanto: **não há divergência de modelo a registrar
//! aqui** — só as de FORMA, abaixo.
//!
//! # As três divergências de forma, todas para tirar caso especial
//!
//! 1. **O vértice de aresta é indexado pela ARESTA** (`V + e`), onde o original
//!    aloca sob demanda com um array de tags. O conjunto é o mesmo; a numeração
//!    passa a ser função da malha em vez da ordem de visita das faces. O array
//!    de tags continua existindo, mas só para responder *é a primeira visita?*,
//!    que é a pergunta que o acúmulo do peso de fato faz.
//! 2. **Os vizinhos de BORDA saem do anel de vértices**, filtrados por
//!    valência de aresta 1 — o original percorre o anel de FACES e desembaraça
//!    os cantos por comparação de índice. Mesma resposta, escrita uma vez (é a
//!    divergência nº 2 que a [`crate::Adjacency`] já tinha tomado).
//! 3. **O corpo é GENÉRICO no canal** ([`Lerpable`]): posição, cor e máscara
//!    passam pelo MESMO código. O original repete a aritmética três vezes, lado
//!    a lado, para os três canais — e uma regra escrita três vezes é uma regra
//!    que diverge no dia em que a quarta entrar.
//!
//! ⚠️ **Toda regra desta tabela é AFIM (os pesos somam 1)**, e é isso que
//! mantém a máscara em `[0, 1]` e a cor no gamute sem um clamp que esconderia o
//! erro caso um peso estivesse errado. Há gate afirmando a soma.

use crate::edges::Edges;
use crate::face::Face;
use crate::mesh::Mesh;

/// Um canal por-vértice que a subdivisão sabe misturar.
///
/// ⚠️ Existe para que **posição, cor e máscara passem pelo mesmo corpo**. As
/// três misturam pela mesma tabela de pesos, então três cópias da tabela seriam
/// três lugares onde ela pode divergir — e a que diverge é sempre a que ninguém
/// olha (a máscara, hoje).
pub trait Lerpable: Copy {
    /// O elemento neutro da soma.
    const ZERO: Self;
    /// `self += other * w`.
    fn add_scaled(&mut self, other: Self, w: f32);
}

impl Lerpable for f32 {
    const ZERO: Self = 0.0;
    fn add_scaled(&mut self, other: Self, w: f32) {
        *self += other * w;
    }
}

impl Lerpable for [f32; 3] {
    const ZERO: Self = [0.0; 3];
    fn add_scaled(&mut self, other: Self, w: f32) {
        for k in 0..3 {
            self[k] += other[k] * w;
        }
    }
}

/// A topologia de saída — computada UMA vez e reusada por cada canal.
struct Plan {
    /// `V`, `V + E`, `V + E + Q`.
    verts: usize,
    edge_base: usize,
    /// Por face de entrada, o índice do vértice central. [`NO_FACE_VERTEX`]
    /// num triângulo, que não tem.
    face_vertex: Vec<u32>,
    faces: Vec<Face>,
    out_verts: usize,
}

/// Um triângulo não tem vértice de face.
const NO_FACE_VERTEX: u32 = u32::MAX;

impl Plan {
    fn build(mesh: &Mesh, edges: &Edges) -> Self {
        let verts = mesh.vert_count();
        let edge_base = verts;
        let face_base = verts + edges.len();
        let mut face_vertex = vec![NO_FACE_VERTEX; mesh.face_count()];
        let mut next_face_vertex = face_base as u32;
        for (f, face) in mesh.faces().iter().enumerate() {
            if !face.is_tri() {
                face_vertex[f] = next_face_vertex;
                next_face_vertex += 1;
            }
        }
        let mut faces = Vec::with_capacity(mesh.face_count() * 4);
        for (f, face) in mesh.faces().iter().enumerate() {
            let v = face.verts();
            // ⚠️ `expect` e não um índice de reserva: todo canto de face TEM
            // aresta por construção do grafo, e cair num índice inventado
            // produziria geometria errada em silêncio, que é o modo de falha
            // que este módulo inteiro evita.
            let mid = |k: usize| {
                edge_base as u32
                    + edges
                        .face_edge(f, k)
                        .expect("todo canto de face tem aresta no grafo")
            };
            if face.is_tri() {
                let (m1, m2, m3) = (mid(0), mid(1), mid(2));
                // O triângulo do MEIO mais os três das quinas. O winding de cada
                // um segue o da face de entrada — trocar dois deles aqui deixa
                // uma malha que renderiza com buracos e não levanta erro nenhum.
                faces.push(Face::tri(m1, m2, m3));
                faces.push(Face::tri(v[0], m1, m3));
                faces.push(Face::tri(m1, v[1], m2));
                faces.push(Face::tri(m2, v[2], m3));
            } else {
                let (m1, m2, m3, m4) = (mid(0), mid(1), mid(2), mid(3));
                let c = face_vertex[f];
                faces.push(Face::quad(v[0], m1, c, m4));
                faces.push(Face::quad(m1, v[1], m2, c));
                faces.push(Face::quad(c, m2, v[2], m3));
                faces.push(Face::quad(m4, c, m3, v[3]));
            }
        }
        Self {
            verts,
            edge_base,
            out_verts: next_face_vertex as usize,
            face_vertex,
            faces,
        }
    }
}

/// **Subdivide `mesh` uma vez.** Devolve a malha nova; a de entrada fica
/// intacta.
///
/// ⚠️ **Não há teto aqui, de propósito.** Isto é uma função pura sobre uma
/// malha, e quem sabe se cabe na máquina é quem tem a máquina — o custo está na
/// sonda `measure_subdivide`, e o teto (se um for preciso) mora em quem OFERECE
/// o botão, com o número da sonda ao lado.
#[must_use]
pub fn subdivide(mesh: &Mesh) -> Mesh {
    let edges = mesh.edges();
    let plan = Plan::build(mesh, &edges);
    let n = plan.out_verts;

    let mut positions = vec![[0.0f32; 3]; n];
    subdivide_channel(mesh, &edges, &plan, mesh.positions(), &mut positions);

    let mut out = Mesh::from_parts(positions, plan.faces.clone())
        .expect("a subdivisão só nomeia vértices que ela mesma criou");

    if let Some(src) = mesh.colors() {
        let mut dst = vec![[0.0f32; 3]; n];
        subdivide_channel(mesh, &edges, &plan, src, &mut dst);
        out.colors_mut().copy_from_slice(&dst);
    }
    if let Some(src) = mesh.masks() {
        let mut dst = vec![0.0f32; n];
        subdivide_channel(mesh, &edges, &plan, src, &mut dst);
        out.masks_mut().copy_from_slice(&dst);
    }
    out
}

/// O corpo: aplica a tabela de pesos a UM canal.
fn subdivide_channel<T: Lerpable>(
    mesh: &Mesh,
    edges: &Edges,
    plan: &Plan,
    src: &[T],
    out: &mut [T],
) {
    even(mesh, edges, plan, src, out);
    odd(mesh, edges, plan, src, out);
}

/// Os vértices ORIGINAIS, suavizados — as quatro regras do original.
fn even<T: Lerpable>(mesh: &Mesh, edges: &Edges, plan: &Plan, src: &[T], out: &mut [T]) {
    let adj = mesh.adjacency();
    for v in 0..plan.verts {
        let ring = adj.vert_verts.neighbours(v);
        let mut acc = T::ZERO;

        if adj.is_border(v) {
            // ⚠️ **Um vértice de borda ouve SÓ a borda** — a mesma lei que a
            // W6.0 pôs no laplaciano, e pelo mesmo motivo: incluir o anel de
            // dentro suga a boca para o miolo e a peça encolhe pelas pontas.
            let border: Vec<u32> = ring
                .iter()
                .copied()
                .filter(|&w| {
                    edges
                        .id_of(adj, v as u32, w)
                        .is_some_and(|e| edges.valence(e) == 1)
                })
                .collect();
            if border.len() < 2 {
                // Não-manifold: sem dois vizinhos de borda não há corda a
                // seguir, e o original também desiste aqui.
                out[v] = src[v];
                continue;
            }
            let beta = 0.25 / border.len() as f32;
            acc.add_scaled(src[v], 0.75);
            for &w in &border {
                acc.add_scaled(src[w as usize], beta);
            }
            out[v] = acc;
            continue;
        }

        let count = ring.len() as f32;
        let mut ring_sum = T::ZERO;
        for &w in ring {
            ring_sum.add_scaled(src[w as usize], 1.0);
        }

        // A diagonal oposta em cada quad incidente — o termo que separa
        // Catmull-Clark de Loop.
        let mut opp_sum = T::ZERO;
        let mut quads = 0f32;
        let faces = adj.vert_faces.neighbours(v);
        for &f in faces {
            let face = mesh.faces()[f as usize];
            if face.is_tri() {
                continue;
            }
            quads += 1.0;
            let q = face.verts();
            let i = q.iter().position(|&x| x as usize == v).unwrap_or(0);
            opp_sum.add_scaled(src[q[(i + 2) % 4] as usize], 1.0);
        }

        let (alpha, beta, gamma) = if quads == 0.0 {
            // Loop, com os pesos de Warren para a valência 3.
            let (a, b) = if (count - 6.0).abs() < f32::EPSILON {
                (0.625, 0.0625)
            } else if (count - 3.0).abs() < f32::EPSILON {
                (0.4375, 0.1875)
            } else {
                (0.625, 0.375 / count)
            };
            (a, b, 0.0)
        } else if quads == faces.len() as f32 {
            // Catmull-Clark.
            if (count - 4.0).abs() < f32::EPSILON {
                (0.5625, 0.09375, 0.015625)
            } else {
                let b = 1.5 / (count * count);
                let g = 0.25 / (count * count);
                (1.0 - (b + g) * count, b, g)
            }
        } else {
            // Fronteira tri/quad: a média ponderada que interpola as duas.
            let a = 1.0 / (1.0 + count * 0.5 + quads * 0.25);
            (a, a * 0.5, a * 0.25)
        };

        acc.add_scaled(src[v], alpha);
        acc.add_scaled(ring_sum, beta);
        if gamma != 0.0 {
            acc.add_scaled(opp_sum, gamma);
        }
        out[v] = acc;
    }
}

/// Os vértices NOVOS: um por aresta, um por quad.
fn odd<T: Lerpable>(mesh: &Mesh, edges: &Edges, plan: &Plan, src: &[T], out: &mut [T]) {
    // ⚠️ *Primeira visita?* — a pergunta que o acúmulo de peso faz. O original
    // usa este mesmo array também para ALOCAR o índice; aqui o índice é a
    // aresta, então sobra só a pergunta.
    let mut seen = vec![false; edges.len()];
    for (f, face) in mesh.faces().iter().enumerate() {
        let v = face.verts();
        let n = v.len();
        for k in 0..n {
            let Some(e) = edges.face_edge(f, k) else {
                continue;
            };
            let slot = plan.edge_base + e as usize;
            let (a, b) = (src[v[k] as usize], src[v[(k + 1) % n] as usize]);

            if edges.valence(e) != 2 {
                // Borda (1) ou não-manifold (≥ 3): o ponto MÉDIO, e uma vez só.
                // Suavizar um vértice não-manifold é uma pergunta sem resposta
                // definida, e o original também recusa.
                if !seen[e as usize] {
                    seen[e as usize] = true;
                    let mut acc = T::ZERO;
                    acc.add_scaled(a, 0.5);
                    acc.add_scaled(b, 0.5);
                    out[slot] = acc;
                }
                continue;
            }

            // Interior manifold: as DUAS faces contribuem, e as duas regras
            // (tri e quad) foram desenhadas para somar 1 quando compostas —
            // 0,375·2 do par + 0,125 de cada oposto de triângulo ou 0,0625 de
            // cada oposta de quad. Uma aresta entre um tri e um quad recebe uma
            // metade de cada, e fecha.
            if !seen[e as usize] {
                seen[e as usize] = true;
                let mut acc = T::ZERO;
                acc.add_scaled(a, 0.375);
                acc.add_scaled(b, 0.375);
                out[slot] = acc;
            }
            let mut acc = out[slot];
            if n == 3 {
                acc.add_scaled(src[v[(k + 2) % 3] as usize], 0.125);
            } else {
                acc.add_scaled(src[v[(k + 2) % 4] as usize], 0.0625);
                acc.add_scaled(src[v[(k + 3) % 4] as usize], 0.0625);
            }
            out[slot] = acc;
        }

        // O vértice de face: o centroide do quad.
        let c = plan.face_vertex[f];
        if c != NO_FACE_VERTEX {
            let mut acc = T::ZERO;
            for &iv in v {
                acc.add_scaled(src[iv as usize], 0.25);
            }
            out[c as usize] = acc;
        }
    }
}

#[cfg(test)]
#[path = "subdivide_tests.rs"]
mod tests;
