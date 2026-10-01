//! ⭐ **O ORÁCULO EXACTO** — o caminho mais curto pela FORÇA BRUTA, que nenhum programa precisa de
//! correr (plano 30 §8.3).
//!
//! O caminho mais curto entre obstáculos poligonais é um TEOREMA: ele é uma linha poligonal cujos
//! vértices interiores são vértices dos obstáculos (Lozano-Pérez & Wesley, 1979). ⇒ o grafo cujos
//! nós são a partida, o alvo e todos os vértices de PAREDE da malha, com uma aresta entre cada par
//! que se vê, mais Dijkstra, dá a resposta EXACTA. Custa `O(V² · paredes)` — é um gate, nunca um
//! caminho de produto (⛔ por isso vive atrás da feature `test-support`).
//!
//! # A visibilidade, sem confiar na procura que se está a medir
//!
//! `p` vê `q` quando o segmento `p → q` está dentro da malha (fechada): nenhuma parede o cruza
//! PROPRIAMENTE, e o meio de cada pedaço entre os vértices de parede que ele toca está DENTRO de um
//! polígono (o toque num vértice pode ser rasante por fora ou por dentro de um obstáculo, e só o meio
//! dos pedaços distingue). Usa só a localização da malha — nada da procura.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{EPS, V2, dist, dist_to_segment, lerp, ord_key, proper_cross};
use crate::mesh::NavMesh;

/// `p` vê `q` dentro da malha?
pub fn visible(mesh: &NavMesh, p: V2, q: V2) -> bool {
    let walls = mesh.walls();
    for &(u, w) in walls {
        if proper_cross(p, q, mesh.vert(u), mesh.vert(w)) {
            return false;
        }
    }
    let l = dist(p, q);
    if l <= EPS {
        return mesh.locate(p).is_some();
    }
    let mut ts = vec![0.0, 1.0];
    for &(u, _) in walls {
        let v = mesh.vert(u);
        if dist_to_segment(p, q, v) <= EPS {
            let t = ((v[0] - p[0]) * (q[0] - p[0]) + (v[1] - p[1]) * (q[1] - p[1])) / (l * l);
            ts.push(t.clamp(0.0, 1.0));
        }
    }
    ts.sort_by(f64::total_cmp);
    for win in ts.windows(2) {
        if (win[1] - win[0]) * l <= EPS {
            continue;
        }
        let mid = lerp(p, q, 0.5 * (win[0] + win[1]));
        if mesh.locate(mid).is_none() {
            return false;
        }
    }
    true
}

/// O caminho mais curto EXACTO de `s` a `t` e o comprimento, ou `None` se não há.
pub fn shortest(mesh: &NavMesh, s: V2, t: V2) -> Option<(Vec<V2>, f64)> {
    mesh.locate(s)?;
    mesh.locate(t)?;
    let mut nodes: Vec<V2> = vec![s, t];
    let mut seen = std::collections::BTreeSet::new();
    for &(u, w) in mesh.walls() {
        for v in [u, w] {
            if seen.insert(v) {
                nodes.push(mesh.vert(v));
            }
        }
    }
    let n = nodes.len();
    let mut best = vec![f64::INFINITY; n];
    let mut prev = vec![usize::MAX; n];
    let mut done = vec![false; n];
    let mut heap = BinaryHeap::new();
    best[0] = 0.0;
    heap.push(Reverse((ord_key(0.0), 0usize)));
    while let Some(Reverse((_, i))) = heap.pop() {
        if done[i] {
            continue;
        }
        done[i] = true;
        if i == 1 {
            break;
        }
        for j in 0..n {
            if done[j] || j == i {
                continue;
            }
            let d = best[i] + dist(nodes[i], nodes[j]);
            if d < best[j] - 1e-12 && visible(mesh, nodes[i], nodes[j]) {
                best[j] = d;
                prev[j] = i;
                heap.push(Reverse((ord_key(d), j)));
            }
        }
    }
    if best[1].is_infinite() {
        return None;
    }
    let mut pts = vec![nodes[1]];
    let mut c = 1;
    while prev[c] != usize::MAX {
        c = prev[c];
        pts.push(nodes[c]);
    }
    pts.reverse();
    Some((pts, best[1]))
}
