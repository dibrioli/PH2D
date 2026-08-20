//! **O GRAFO DA MALHA NOVA** — porte **FIEL** de `extract_graph`
//! (`instant-meshes`, `src/extract.cpp`), BSD-3-Clause.
//!
//! ⚠️ **Esta é a peça que a primeira versão desta crate INVENTOU, e a fatura
//! chegou em foto** (Enio, 2026-08-19: *"uma bosta"*). A minha versão adivinhava
//! a grade a partir da geometria — um cone de 45°, uma janela de distância — e a
//! referência não faz nada disso. Ela **lê o passo inteiro entre as duas
//! retículas** e classifica cada aresta da entrada em três casos, e mais nada.
//!
//! Os seis passos da referência, e o que cada um responde:
//!
//! | passo | o que faz |
//! |---|---|
//! | 1 | **classifica** cada aresta: colapsar · ligar · ignorar |
//! | 2 | **colapsa** por erro crescente (`union-find`, unindo as vizinhanças) |
//! | 3 | **compacta** os vértices sobreviventes |
//! | 3a | remove os **espúrios** (os que quase não colapsaram nada) |
//! | 4 | dá a **posição**: média PONDERADA das origens da célula |
//! | 5 | **encolhe triângulos e tira diagonais** — o passo que faz a grade ser quad |
//! | 6 | **ordena** os vizinhos por ângulo, que é o sistema de rotação |
//!
//! ⚠️ **O passo 5 é o que eu não tinha, e é o que o artista vê.** Ele encolhe
//! todo triângulo cuja altura é menor que `0,3` da célula, e depois **remove toda
//! aresta que é diagonal de um quad** (a que tem exatamente dois triângulos
//! pendurados). Sem ele o grafo fica cheio de triângulos e o passeio de faces
//! devolve o que devolveu.

use std::collections::BTreeSet;

use ph2d_mesh::Mesh;

use crate::orientation::{OrientationField, compat_orientation_extrinsic_4};
use crate::position::{PositionField, compat_position_extrinsic_index_4};
use crate::scale::ScaleField;

use crate::extract::{cross, dot, sub};

/// O grafo extraído: vizinhança, posição e normal por nó.
pub(crate) struct ExtractedGraph {
    /// A vizinhança de cada nó, já ordenada por ângulo (passo 6).
    pub adj: Vec<Vec<u32>>,
    /// A origem de cada nó.
    pub o: Vec<[f32; 3]>,
    /// A normal de cada nó.
    pub n: Vec<[f32; 3]>,
    /// Quantos nós a limpeza encolheu, quantas diagonais ela tirou e em quantas
    /// rodadas — o diagnóstico do passo 5, que é o mais intrincado do porte.
    ///
    /// ⚠️ Só as sondas o lêem, e é de propósito: um número que só existe num
    /// `eprintln!` não é comparável entre duas corridas.
    #[cfg_attr(not(test), allow(dead_code))]
    pub cleanup: (usize, usize, usize),
}

/// **UNION-FIND** com compressão de caminho e união por tamanho.
///
/// ⚠️ **A união escolhe o MENOR ÍNDICE como raiz quando os tamanhos empatam**, e
/// é isso que torna a rotulagem byte-reprodutível (A7 / HR-5). A referência usa
/// os bits altos do rank e desempata pela ordem de chegada das threads — o que
/// não é uma opção aqui.
struct DisjointSets {
    parent: Vec<u32>,
    size: Vec<u32>,
}

impl DisjointSets {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n as u32).collect(),
            size: vec![1; n],
        }
    }

    fn find(&mut self, mut x: u32) -> u32 {
        while self.parent[x as usize] != x {
            let g = self.parent[self.parent[x as usize] as usize];
            self.parent[x as usize] = g;
            x = g;
        }
        x
    }

    /// Une duas raízes e devolve a que sobreviveu.
    fn unite(&mut self, a: u32, b: u32) -> u32 {
        let (big, small) = if self.size[a as usize] > self.size[b as usize]
            || (self.size[a as usize] == self.size[b as usize] && a < b)
        {
            (a, b)
        } else {
            (b, a)
        };
        self.parent[small as usize] = big;
        self.size[big as usize] += self.size[small as usize];
        big
    }
}

/// A classificação de UMA aresta da entrada — os três casos da referência.
enum EdgeKind {
    /// Passo `(0,0)`: os dois vértices descrevem o **mesmo** nó da grade.
    Collapse(f32),
    /// Passo de norma 1: os dois nós são **vizinhos** na grade.
    Link,
    /// Passo maior, ou a **diagonal** `(1,1)` — não descreve aresta nenhuma.
    Ignore,
}

/// **PASSO 1 — a classificação, e é a lei inteira do grafo.**
///
/// ⚠️ **A orientação de `j` é ROTACIONADA para a moldura de `i` ANTES de medir o
/// passo**, e foi exatamente isto que a minha versão não fazia: os índices de
/// cada lado vivem na retícula do seu vértice, e compará-los sem alinhar as duas
/// cruzes é comparar coordenadas de referenciais diferentes.
///
/// ⚠️ **E a DIAGONAL `(1,1)` é recusada de propósito.** Numa grade de quads o
/// vizinho diagonal existe geometricamente e **não é uma aresta** — aceitá-lo
/// enche o grafo de triângulos. É uma linha de código, e é a diferença entre uma
/// grade e uma sopa.
fn classify(
    i: usize,
    j: usize,
    v: &[[f32; 3]],
    nrm: &[[f32; 3]],
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
) -> EdgeKind {
    let (q0, q1) = compat_orientation_extrinsic_4(orient.dir(i), nrm[i], orient.dir(j), nrm[j]);
    let (s0, s1, error) = compat_position_extrinsic_index_4(
        v[i],
        nrm[i],
        q0,
        pos.at(i),
        scale.at(i),
        v[j],
        nrm[j],
        q1,
        pos.at(j),
        scale.at(j),
    );
    let d = [(s0[0] - s1[0]).abs(), (s0[1] - s1[1]).abs()];
    if d[0].max(d[1]) > 1 || (d[0] == 1 && d[1] == 1) {
        return EdgeKind::Ignore;
    }
    if d[0] + d[1] == 0 {
        EdgeKind::Collapse(error)
    } else {
        EdgeKind::Link
    }
}

/// **EXTRAI O GRAFO** — os seis passos da referência, em ordem.
pub(crate) fn extract_graph(
    mesh: &Mesh,
    orient: &OrientationField,
    pos: &PositionField,
    scale: &ScaleField,
) -> ExtractedGraph {
    let n = mesh.vert_count();
    let v = mesh.positions();
    let nrm = mesh.normals();
    let adjacency = mesh.adjacency();

    // ── PASSO 1: classificar ────────────────────────────────────────────────
    let mut dset = DisjointSets::new(n);
    let mut adj_new: Vec<Vec<u32>> = vec![Vec::new(); n];
    let mut collapse_edges: Vec<(f32, u32, u32)> = Vec::new();
    for i in 0..n {
        for &jj in adjacency.vert_verts.neighbours(i) {
            let j = jj as usize;
            if j < i {
                continue;
            }
            match classify(i, j, v, nrm, orient, pos, scale) {
                EdgeKind::Ignore => {}
                EdgeKind::Collapse(error) => collapse_edges.push((error, i as u32, jj)),
                EdgeKind::Link => {
                    adj_new[i].push(jj);
                    adj_new[j].push(i as u32);
                }
            }
        }
    }

    // ── PASSO 2: colapsar, por erro CRESCENTE ───────────────────────────────
    // ⚠️ **A ordem é o algoritmo, não uma otimização:** colapsar primeiro os
    // pares que mais concordam deixa os duvidosos para o fim, quando já há
    // vizinhança formada a recusá-los (o teste de conflito abaixo).
    collapse_edges.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut collapses = vec![0u32; n];
    for &(_, ei, ej) in &collapse_edges {
        let (a, b) = (dset.find(ei), dset.find(ej));
        if a == b {
            continue;
        }
        // ⚠️ **O teste de CONFLITO:** se `a` já tem uma vizinha que mora em `b`,
        // uni-los criaria um laço — o nó ficaria vizinho de si mesmo, e o passeio
        // de faces não tem resposta para isso.
        if adj_new[a as usize].iter().any(|&k| dset.find(k) == b) {
            continue;
        }
        let mut merged: BTreeSet<u32> = BTreeSet::new();
        for &k in &adj_new[a as usize] {
            merged.insert(dset.find(k));
        }
        for &k in &adj_new[b as usize] {
            merged.insert(dset.find(k));
        }
        let target = dset.unite(a, b);
        adj_new[a as usize] = Vec::new();
        adj_new[b as usize] = Vec::new();
        let c = collapses[a as usize] + collapses[b as usize] + 1;
        collapses[target as usize] = c;
        adj_new[target as usize] = merged.into_iter().collect();
    }

    // ── PASSO 3: compactar ──────────────────────────────────────────────────
    let mut vertex_map = vec![u32::MAX; n];
    let mut count = 0usize;
    let mut packed_adj: Vec<Vec<u32>> = Vec::new();
    let mut packed_collapses: Vec<u32> = Vec::new();
    for i in 0..n {
        if adj_new[i].is_empty() {
            continue;
        }
        vertex_map[i] = count as u32;
        packed_adj.push(core::mem::take(&mut adj_new[i]));
        packed_collapses.push(collapses[i]);
        count += 1;
    }
    let avg_collapses = if count == 0 {
        0.0
    } else {
        packed_collapses.iter().map(|c| f64::from(*c)).sum::<f64>() / count as f64
    };
    for list in &mut packed_adj {
        let mut temp: BTreeSet<u32> = BTreeSet::new();
        for &k in list.iter() {
            let r = dset.find(k);
            if vertex_map[r as usize] != u32::MAX {
                temp.insert(vertex_map[r as usize]);
            }
        }
        *list = temp.into_iter().collect();
    }

    // ── PASSO 3a: remover os ESPÚRIOS ───────────────────────────────────────
    // Um nó que quase não colapsou nada é um vértice da entrada que sobreviveu
    // sozinho — ele não descreve um nó de grade, descreve um acidente.
    for i in 0..count {
        if f64::from(packed_collapses[i]) > avg_collapses / 10.0 {
            continue;
        }
        let gone = core::mem::take(&mut packed_adj[i]);
        for k in gone {
            packed_adj[k as usize].retain(|&x| x != i as u32);
        }
    }

    // ── PASSO 4: a POSIÇÃO, por média PONDERADA ─────────────────────────────
    // ⚠️ **O peso `exp(−9·|O−V|²/s²)` não é enfeite:** um vértice cuja origem de
    // retícula ficou longe dele próprio é um vértice em que o campo não se
    // decidiu, e deixá-lo puxar a média com o mesmo voto arrasta o nó para fora
    // da superfície. A minha versão fazia média simples.
    let mut o_new = vec![[0.0f32; 3]; count];
    let mut n_new = vec![[0.0f32; 3]; count];
    let mut weight_sum = vec![0.0f32; count];
    for i in 0..n {
        let r = dset.find(i as u32);
        let j = vertex_map[r as usize];
        if j == u32::MAX {
            continue;
        }
        let j = j as usize;
        let inv_scale = 1.0 / scale.at(i).max(crate::scale::MIN_EDGE);
        let d = sub(pos.at(i), v[i]);
        let weight = (-dot(d, d) * inv_scale * inv_scale * 9.0).exp();
        for (k, (o, nn)) in o_new[j].iter_mut().zip(n_new[j].iter_mut()).enumerate() {
            *o += pos.at(i)[k] * weight;
            *nn += nrm[i][k] * weight;
        }
        weight_sum[j] += weight;
    }
    for i in 0..count {
        if weight_sum[i] > 0.0 {
            let inv = 1.0 / weight_sum[i];
            for c in &mut o_new[i] {
                *c *= inv;
            }
        }
        n_new[i] = normalize_or(n_new[i], [0.0, 0.0, 1.0]);
    }

    // ── PASSO 5: encolher triângulos e TIRAR AS DIAGONAIS ───────────────────
    let global_scale = mean_scale(scale, n);
    let cleanup_stats = cleanup(&mut packed_adj, &mut o_new, &mut n_new, global_scale);

    // ── PASSO 6: ordenar por ÂNGULO ─────────────────────────────────────────
    // ⚠️ **DECRESCENTE**, e o sentido importa: é ele que decide para que lado o
    // passeio de faces vira, e portanto de que lado ficam as normais.
    for i in 0..count {
        let (s, t) = coordinate_system(n_new[i]);
        let p = o_new[i];
        let angle = |x: u32| {
            let d = sub(o_new[x as usize], p);
            dot(d, t).atan2(dot(d, s))
        };
        packed_adj[i].sort_by(|a, b| angle(*b).total_cmp(&angle(*a)));
    }

    ExtractedGraph {
        adj: packed_adj,
        o: o_new,
        n: n_new,
        cleanup: cleanup_stats,
    }
}

/// **PASSO 5 — a limpeza**, porte fiel do bloco `remove_unnecessary_edges`.
///
/// Duas metades que se repetem até nada mudar:
///
/// 1. **encolher**: todo caminho `i→j→k` cujo triângulo tenha ALTURA menor que
///    `0,3` da célula é degenerado — ou funde dois nós, ou move `i` para o meio
///    de `j` e `k`;
/// 2. **tirar as diagonais**: toda aresta `(i,j)` que tenha **exatamente dois**
///    vizinhos comuns é a diagonal de um quad, e sai.
///
/// ⚠️ **A segunda metade é a que transforma triângulos em quads**, e era ela que
/// faltava. Não há emparelhamento de triângulos nenhum na referência: os
/// triângulos simplesmente não chegam a existir.
fn cleanup(
    adj: &mut [Vec<u32>],
    o: &mut [[f32; 3]],
    n: &mut [[f32; 3]],
    scale: f32,
) -> (usize, usize, usize) {
    let thresh = 0.3 * scale;
    let (mut snapped, mut removed) = (0usize, 0usize);
    let mut changed = true;
    let mut rounds = 0usize;
    while changed && rounds < MAX_CLEANUP_ROUNDS {
        rounds += 1;
        changed = false;

        // (1) encolher os triângulos degenerados.
        let mut inner = true;
        let mut inner_rounds = 0usize;
        while inner && inner_rounds < MAX_CLEANUP_ROUNDS {
            inner_rounds += 1;
            inner = false;
            let mut candidates: Vec<(f32, u32, u32, u32)> = Vec::new();
            for i in 0..adj.len() {
                let p_i = o[i];
                for &j in &adj[i] {
                    let p_j = o[j as usize];
                    for &k in &adj[j as usize] {
                        if k == i as u32 {
                            continue;
                        }
                        let p_k = o[k as usize];
                        let (a, b, c) = (dist(p_j, p_k), dist(p_i, p_j), dist(p_i, p_k));
                        if a > b.max(c) {
                            let s = 0.5 * (a + b + c);
                            let area2 = (s * (s - a) * (s - b) * (s - c)).max(0.0);
                            let height = 2.0 * area2.sqrt() / a.max(1.0e-20);
                            if height < thresh {
                                candidates.push((height, i as u32, j, k));
                            }
                        }
                    }
                }
            }
            candidates.sort_by(|x, y| {
                x.0.total_cmp(&y.0)
                    .then(x.1.cmp(&y.1))
                    .then(x.2.cmp(&y.2))
                    .then(x.3.cmp(&y.3))
            });
            for (height, i, j, k) in candidates {
                let (i, j, k) = (i as usize, j as usize, k as usize);
                let edge1 = adj[i].contains(&(j as u32));
                let edge2 = adj[j].contains(&(k as u32));
                let edge3 = adj[k].contains(&(i as u32));
                if !edge1 || !edge2 {
                    continue;
                }
                let (p_i, p_j, p_k) = (o[i], o[j], o[k]);
                let (a, b, c) = (dist(p_j, p_k), dist(p_i, p_j), dist(p_i, p_k));
                let s = 0.5 * (a + b + c);
                let area2 = (s * (s - a) * (s - b) * (s - c)).max(0.0);
                let h = 2.0 * area2.sqrt() / a.max(1.0e-20);
                // ⚠️ **A re-medição não é paranoia:** a lista foi construída antes
                // de qualquer fusão, e uma fusão anterior pode ter movido estes
                // três nós. A referência compara com o valor gravado e desiste se
                // mudou.
                if (h - height).abs() > f32::EPSILON * h.abs().max(1.0) {
                    continue;
                }
                if b < thresh || c < thresh {
                    // Fundir `i` com o vizinho perto.
                    let merge = if b < thresh { j } else { k };
                    for t in 0..3 {
                        o[i][t] = (o[i][t] + o[merge][t]) * 0.5;
                        n[i][t] = (n[i][t] + n[merge][t]) * 0.5;
                    }
                    let mut updated: BTreeSet<u32> = BTreeSet::new();
                    let gone = core::mem::take(&mut adj[merge]);
                    for x in gone {
                        if x as usize == i {
                            continue;
                        }
                        updated.insert(x);
                        for e in &mut adj[x as usize] {
                            if *e == merge as u32 {
                                *e = i as u32;
                            }
                        }
                    }
                    for &x in &adj[i] {
                        updated.insert(x);
                    }
                    updated.remove(&(i as u32));
                    updated.remove(&(merge as u32));
                    adj[i] = updated.into_iter().collect();
                } else {
                    // Mover `i` para o meio da aresta longa, e trocar a ligação.
                    o[i] = [
                        (p_j[0] + p_k[0]) * 0.5,
                        (p_j[1] + p_k[1]) * 0.5,
                        (p_j[2] + p_k[2]) * 0.5,
                    ];
                    n[i] = normalize_or(
                        [n[j][0] + n[k][0], n[j][1] + n[k][1], n[j][2] + n[k][2]],
                        n[i],
                    );
                    adj[j].retain(|&x| x != k as u32);
                    adj[k].retain(|&x| x != j as u32);
                    if !edge3 {
                        adj[i].push(k as u32);
                        adj[k].push(i as u32);
                    }
                }
                changed = true;
                inner = true;
                snapped += 1;
            }
        }

        // (2) ⭐ TIRAR AS DIAGONAIS.
        let mut candidates: Vec<(f32, u32, u32)> = Vec::new();
        for i in 0..adj.len() {
            let p_i = o[i];
            for &j in &adj[i] {
                let p_j = o[j as usize];
                let mut tris = 0usize;
                let mut length = 0.0f32;
                for &k in &adj[i] {
                    if k == j {
                        continue;
                    }
                    if !adj[j as usize].contains(&k) {
                        continue;
                    }
                    tris += 1;
                    length += dist(o[k as usize], p_i) + dist(o[k as usize], p_j);
                }
                if tris == 2 {
                    // ⚠️ **O escore é a DIAGONAL ESPERADA de um quad**
                    // (`lado · √2`): entre duas arestas candidatas a sair, sai
                    // primeiro a que mais se parece com uma diagonal.
                    let exp_diag = length / 4.0 * core::f32::consts::SQRT_2;
                    let diag = dist(p_i, p_j);
                    let score = ((diag - exp_diag) / diag.min(exp_diag).max(1.0e-20)).abs();
                    candidates.push((score, i as u32, j));
                }
            }
        }
        candidates.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)).then(x.2.cmp(&y.2)));
        for (_, i, j) in candidates {
            let (i, j) = (i as usize, j as usize);
            if !adj[i].contains(&(j as u32)) {
                continue;
            }
            let tris = adj[i].iter().filter(|k| adj[j].contains(k)).count();
            if tris == 2 {
                adj[i].retain(|&x| x != j as u32);
                adj[j].retain(|&x| x != i as u32);
                changed = true;
                removed += 1;
            }
        }
    }
    (snapped, removed, rounds)
}

/// **O TETO DE RODADAS da limpeza** — uma guarda de TERMINAÇÃO, não de gosto.
///
/// ⚠️ A referência roda `do { } while (changed)` sem teto, e confia na
/// convergência. Aqui o teto existe porque o passe corre **no laço do artista**:
/// uma malha patológica que oscilasse entre duas configurações travaria o app, e
/// um remesh imperfeito é melhor que uma janela morta. Medido nas fixturas: a
/// limpeza converge em **menos de 10** rodadas.
const MAX_CLEANUP_ROUNDS: usize = 64;

fn mean_scale(scale: &ScaleField, n: usize) -> f32 {
    if n == 0 {
        return crate::scale::MIN_EDGE;
    }
    let sum: f64 = (0..n).map(|v| f64::from(scale.at(v))).sum();
    (sum / n as f64) as f32
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = sub(a, b);
    dot(d, d).sqrt()
}

fn normalize_or(a: [f32; 3], fallback: [f32; 3]) -> [f32; 3] {
    let len = dot(a, a).sqrt();
    if len > 1.0e-20 {
        [a[0] / len, a[1] / len, a[2] / len]
    } else {
        fallback
    }
}

/// O par tangente de uma normal — porte de `coordinate_system` (`common.h`).
pub(crate) fn coordinate_system(a: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let c = if a[0].abs() > a[1].abs() {
        let inv_len = 1.0 / a[0].mul_add(a[0], a[2] * a[2]).sqrt();
        [a[2] * inv_len, 0.0, -a[0] * inv_len]
    } else {
        let inv_len = 1.0 / a[1].mul_add(a[1], a[2] * a[2]).sqrt();
        [0.0, a[2] * inv_len, -a[1] * inv_len]
    };
    (cross(c, a), c)
}

#[cfg(test)]
#[path = "im_graph_tests.rs"]
mod tests;
