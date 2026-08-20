//! **QUE CÉLULAS SÃO VIZINHAS** — as duas leis de ligação, e o corte de assunto
//! entre o [`super`] (*que células existem*) e o [`super::faces`] (*que polígonos
//! elas delimitam*).
//!
//! ⚠️ **O corte foi FORÇADO pelo teto de LOC do workspace (700)** e é o mesmo do
//! irmão das faces: cada metade cresce por um motivo diferente — esta com cada
//! lei de vizinhança, aquela com cada patologia de passeio.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_mesh::Mesh;

use crate::orientation::OrientationField;
use crate::position::PositionField;
use crate::scale::ScaleField;

use super::{Cells, cross, dot, sub};

/// **O PASSO INTEIRO ENTRE DUAS RETÍCULAS**, na moldura de `v`.
///
/// A retícula de `v` é `o_v + s·(a·q + b·(n×q))`. Este é o par `(a, b)` que leva
/// `o_v` ao ponto de retícula mais próximo de `o_w` — ou seja, **quantas células
/// separam os dois campos**.
///
/// ⚠️ **É o MESMO arredondamento que já decide o agrupamento**, lido um degrau
/// adiante: o [`super::cluster_lattice`] pergunta *"o passo é `(0,0)`?"*, e esta
/// pergunta *"então de quanto é?"*. Fazer as duas com a mesma conta é o que
/// impede que a fusão e a vizinhança discordem — duas respostas para a mesma
/// pergunta divergem no dia em que uma delas ganha um consumidor novo.
fn lattice_step(
    o_v: [f32; 3],
    q: [f32; 3],
    n: [f32; 3],
    o_w: [f32; 3],
    scale: f32,
) -> ([i32; 2], f32) {
    let s = scale.max(crate::scale::MIN_EDGE);
    let inv = 1.0 / s;
    let t = cross(n, q);
    let d = sub(o_w, o_v);
    let (u, v) = (dot(q, d) * inv, dot(t, d) * inv);
    let (a, b) = (u.round(), v.round());
    // O RESÍDUO, em células: quão longe o campo está de um passo inteiro. É a
    // única grandeza aqui que diz se a leitura é confiável.
    let residual = (u - a).hypot(v - b);
    ([a as i32, b as i32], residual)
}

/// **O RESÍDUO MÁXIMO que ainda conta como um passo da retícula.**
///
/// ⚠️ **É a metade da célula, e não um gosto:** acima de `0,5` o ponto está mais
/// perto do nó SEGUINTE, e o arredondamento já teria escolhido outro passo. O
/// número aqui é o que separa *"os dois campos concordam"* de *"o campo ainda
/// não convergiu ali"* — e a folga é a mesma que o `round` usa, por construção.
///
/// Medido sobre a esfera amassada da cena `=35` (`edge = 0,18`): com `0,5` a
/// leitura aceita 100 % das arestas e a valência mediana sai 4; a metade
/// (`0,25`) recusa arestas em singularidade legítima e reabre os buracos que o
/// passeio contorna.
const MAX_RESIDUAL: f32 = 0.5;

/// **O GRAFO PELO PASSO DA RETÍCULA** — a lei da referência.
///
/// Para cada aresta da entrada, o passo inteiro entre os dois campos. Passo
/// `(0,0)` já foi consumido pelo agrupamento (as duas pontas são a mesma célula);
/// passo de **norma 1** (`|a| + |b| = 1`) é uma aresta da grade nova. Qualquer
/// outro passo é o campo a saltar mais de uma célula sobre uma aresta da entrada
/// — o que só acontece onde a malha de entrada é mais grossa que o quad pedido, e
/// ali não há aresta de grade a inferir.
///
/// ⚠️ **Sem cone e sem janela de distância.** A lei anterior
/// ([`neighbour_graph`]) adivinhava a grade a partir da geometria: a candidata
/// mais alinhada em cada quadrante, dentro de `[0,5 s, 1,7 s]`. Os dois limiares
/// eram a superfície do erro — uma célula legítima 1,8 células adiante (numa
/// singularidade, onde a grade estica) era recusada, e o passeio de faces
/// contornava o buraco: foi de lá que saíram os ciclos de **44 lados**, e um
/// ciclo de 44 lados vira 42 triângulos **em leque** — o objeto espetado da foto
/// do Enio (2026-08-19).
///
/// ⚠️ **A votação é o que torna a leitura robusta.** Duas células são separadas
/// por muitas arestas da entrada, e cada uma vota; basta uma concordar. Exigir
/// unanimidade seria deixar um vértice mal resolvido cortar uma aresta de grade
/// inteira.
pub(super) fn lattice_graph(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
    c: &Cells,
) -> Vec<BTreeSet<u32>> {
    let n = mesh.vert_count();
    let normals = mesh.normals();
    let adj = mesh.adjacency();
    let k = c.verts.len();

    // Par de células → quantas arestas da entrada votaram nela.
    let mut votes: BTreeMap<(u32, u32), u32> = BTreeMap::new();
    for (v, nv) in normals.iter().enumerate().take(n) {
        for &w in adj.vert_verts.neighbours(v) {
            let w = w as usize;
            if w <= v {
                continue;
            }
            let (ca, cb) = (c.of[v], c.of[w]);
            if ca == cb {
                continue;
            }
            let (step, residual) =
                lattice_step(pos.at(v), orient.dir(v), *nv, pos.at(w), scale.at(v));
            if residual > MAX_RESIDUAL {
                continue;
            }
            if step[0].abs() + step[1].abs() != 1 {
                continue;
            }
            let key = if ca < cb { (ca, cb) } else { (cb, ca) };
            *votes.entry(key).or_insert(0) += 1;
        }
    }

    let mut g = vec![BTreeSet::new(); k];
    for (a, b) in votes.into_keys() {
        g[a as usize].insert(b);
        g[b as usize].insert(a);
    }
    g
}

/// **AS ARESTAS — as da RETÍCULA, não as da entrada.** *(a lei do CONE, mantida
/// como controle — ver [`super::Linking::Cone`])*
///
/// Cada célula liga-se à melhor candidata em **cada uma das quatro direções** da
/// cruz local (`±q`, `±(n×q)`), entre as células que a malha de entrada torna
/// vizinhas.
///
/// ⚠️ **A primeira versão ligava toda aresta da entrada que atravessasse
/// células, e o resultado foi medido: 7,2 % de quads.** É aritmética, não azar —
/// a entrada é uma triangulação, cada célula herdava ~6 vizinhas, e o passeio de
/// faces devolvia **triângulos**. Uma grade de quads tem quatro vizinhas por nó,
/// e quem as escolhe é a **cruz**.
///
/// ⚠️ **Os dois limiares são derivados, não escolhidos** — e é **por serem
/// derivados de uma premissa geométrica que eles falham**: o cone de 45° reparte
/// o plano em quatro quadrantes, e a janela `[0,5 s, 1,7 s]` é uma célula com
/// folga. Ambos assumem que a grade nova é *localmente métrica* — e ela não é:
/// numa singularidade a grade estica, a vizinha legítima cai fora da janela, e a
/// célula fica com um buraco. Medido na esfera amassada: 117 células de 392 com
/// **cinco ou mais** vizinhas e 32 com menos de quatro.
///
/// ⚠️ **O grafo sai SIMÉTRICO** (as duas pontas inserem), porque uma aresta que
/// só um lado reivindica quebra o sistema de rotação.
pub(super) fn neighbour_graph(mesh: &Mesh, c: &Cells, scale: &ScaleField) -> Vec<BTreeSet<u32>> {
    let (verts, normals, dirs, of) = (&c.verts, &c.normals, &c.dirs, &c.of);
    let k = verts.len();

    // As candidatas: as células que a entrada torna vizinhas.
    let mut near = vec![BTreeSet::new(); k];
    let adj = mesh.adjacency();
    for v in 0..mesh.vert_count() {
        for &w in adj.vert_verts.neighbours(v) {
            let (a, b) = (of[v], of[w as usize]);
            if a != b {
                near[a as usize].insert(b);
                near[b as usize].insert(a);
            }
        }
    }

    // A escala de cada célula — a do primeiro vértice que caiu nela.
    let mut cell_scale = vec![0.0f32; k];
    let mut seen = vec![false; k];
    for (v, &c) in of.iter().enumerate() {
        if !seen[c as usize] {
            cell_scale[c as usize] = scale.at(v);
            seen[c as usize] = true;
        }
    }

    // ⚠️ **Primeiro cada célula ESCOLHE, e só depois as escolhas se confrontam.**
    // A versão anterior inseria a aresta nos dois lados assim que UM deles a
    // queria — e a valência estourava: medido na esfera, **390 células com 5 ou
    // mais vizinhas** (até 11), sobre uma grade cujo nó tem quatro.
    let mut choice: Vec<Vec<u32>> = vec![Vec::new(); k];
    for c in 0..k {
        let (o, n, q) = (verts[c], normals[c], dirs[c]);
        let t = cross(n, q);
        let s = cell_scale[c].max(crate::scale::MIN_EDGE);
        let axes = [q, t, [-q[0], -q[1], -q[2]], [-t[0], -t[1], -t[2]]];
        for axis in axes {
            let mut best: Option<(f32, u32)> = None;
            for &cand in &near[c] {
                let d = sub(verts[cand as usize], o);
                let len = dot(d, d).sqrt();
                if len < 0.5 * s || len > 1.7 * s {
                    continue;
                }
                let cosang = dot(d, axis) / len;
                if cosang < core::f32::consts::FRAC_1_SQRT_2 {
                    continue;
                }
                // A melhor é a mais ALINHADA, e o desempate é o índice — nunca a
                // ordem de visita.
                let score = cosang;
                if best.is_none_or(|(b, bi)| score > b || (score == b && cand < bi)) {
                    best = Some((score, cand));
                }
            }
            if let Some((_, w)) = best {
                choice[c].push(w);
            }
        }
    }

    // ⚠️ **A aresta vale se UM dos lados a escolheu — a MUTUALIDADE foi MEDIDA e
    // REJEITADA.** Exigir que as duas pontas se escolhessem limita a valência a
    // quatro por construção, o que parecia a cura do histograma. Medido: ela
    // **remove** arestas de mais, o passeio de faces passa a atravessar os
    // buracos, e os ciclos chegam a **31 lados** — a fração honesta de quads cai
    // de **53,3 % para 35,0 %**.
    let mut g = vec![BTreeSet::new(); k];
    for c in 0..k {
        for &w in &choice[c] {
            g[c].insert(w);
            g[w as usize].insert(c as u32);
        }
    }
    g
}
