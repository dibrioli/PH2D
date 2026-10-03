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

/// ⭐ **O ORÁCULO PONDERADO** (W7) — o caminho mais BARATO quando cada área tem um custo (`costs`, ver
/// [`crate::cost`]).
///
/// O caminho óptimo entre regiões pesadas (Mitchell & Papadimitriou 1991) é RECTO dentro de cada área,
/// DOBRA só nos cantos das paredes e REFRACTA nas fronteiras entre áreas. ⇒ os nós são a partida, o
/// alvo, os cantos da malha e pontos de Steiner SÓ nas arestas entre áreas diferentes (de `spacing`
/// em `spacing` metros, as pontas incluídas); entre cada par de nós cujo segmento fica na malha há
/// uma aresta com o custo EXACTO dele ([`crate::cost::segment_cost`]). Dijkstra.
///
/// ⚠️ Não é exacto: converge para o óptimo por cima quando `spacing` desce (o único erro é onde o
/// caminho atravessa uma fronteira). A régua é a CONVERGÊNCIA medida, nunca um `spacing` escolhido.
/// ⛔ A 1.ª redacção punha Steiner em TODAS as arestas (Lanthier et al. 1997) com troços só dentro de
/// cada polígono: convergia LINEARMENTE (`+9,8 %` a 8 pontos por aresta, `+5,3 %` a 16) — a recta
/// dentro de uma área uniforme virava ziguezague pelos pontos das arestas interiores.
pub struct WeightedOracle<'a> {
    mesh: &'a NavMesh,
    costs: Vec<f64>,
    nodes: Vec<V2>,
    adj: Vec<Vec<(u32, f64)>>,
}

impl<'a> WeightedOracle<'a> {
    /// O grafo entre os cantos e os pontos das fronteiras (caro: `O(nós²)` caminhadas — uma vez por
    /// malha e tabela; as consultas só ligam a partida e o alvo).
    pub fn new(mesh: &'a NavMesh, costs: &[f64], spacing: f64) -> Self {
        use crate::cost::segment_cost;
        let mut seen = std::collections::BTreeSet::new();
        let mut nodes: Vec<V2> = Vec::new();
        for v in 0..mesh.verts().len() as u32 {
            if mesh.is_corner(v) {
                seen.insert(v);
                nodes.push(mesh.vert(v));
            }
        }
        let polys = mesh.polys();
        for (pi, p) in polys.iter().enumerate() {
            let n = p.len();
            for i in 0..n {
                let Some(q) = p.nbrs[i] else { continue };
                if (q as usize) < pi || mesh.area_id(q) == mesh.area_id(pi as u32) {
                    continue;
                }
                let (u, w) = (p.verts[i], p.verts[(i + 1) % n]);
                for v in [u, w] {
                    if seen.insert(v) {
                        nodes.push(mesh.vert(v));
                    }
                }
                let (a, b) = (mesh.vert(u), mesh.vert(w));
                let k = (dist(a, b) / spacing).ceil().max(1.0) as usize;
                nodes.extend((1..k).map(|j| lerp(a, b, j as f64 / k as f64)));
            }
        }
        let n = nodes.len();
        let mut adj = vec![Vec::new(); n];
        for i in 0..n {
            for j in (i + 1)..n {
                if let Some(c) = segment_cost(mesh, costs, nodes[i], nodes[j]) {
                    adj[i].push((j as u32, c));
                    adj[j].push((i as u32, c));
                }
            }
        }
        Self {
            mesh,
            costs: costs.to_vec(),
            nodes,
            adj,
        }
    }

    /// Quantos nós o grafo tem (a sonda imprime-o ao lado da convergência).
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// O caminho mais barato de `s` a `t` (os dois na malha) e o custo, ou `None`.
    pub fn shortest(&self, s: V2, t: V2) -> Option<(Vec<V2>, f64)> {
        use crate::cost::segment_cost;
        let (m, costs) = (self.mesh, &self.costs[..]);
        m.locate(s)?;
        m.locate(t)?;
        let n = self.nodes.len();
        // Os nós n e n+1 são a partida e o alvo.
        let de_s: Vec<Option<f64>> = self
            .nodes
            .iter()
            .map(|&q| segment_cost(m, costs, s, q))
            .collect();
        let ate_t: Vec<Option<f64>> = self
            .nodes
            .iter()
            .map(|&q| segment_cost(m, costs, q, t))
            .collect();
        let directo = segment_cost(m, costs, s, t);
        let mut best = vec![f64::INFINITY; n + 2];
        let mut prev = vec![usize::MAX; n + 2];
        let mut done = vec![false; n + 2];
        let mut heap = BinaryHeap::new();
        best[n] = 0.0;
        heap.push(Reverse((ord_key(0.0), n)));
        while let Some(Reverse((_, i))) = heap.pop() {
            if done[i] {
                continue;
            }
            done[i] = true;
            if i == n + 1 {
                break;
            }
            let mut relax = |j: usize, c: f64, heap: &mut BinaryHeap<_>| {
                let d = best[i] + c;
                if d < best[j] {
                    best[j] = d;
                    prev[j] = i;
                    heap.push(Reverse((ord_key(d), j)));
                }
            };
            if i == n {
                for (j, c) in de_s.iter().enumerate() {
                    if let Some(c) = *c {
                        relax(j, c, &mut heap);
                    }
                }
                if let Some(c) = directo {
                    relax(n + 1, c, &mut heap);
                }
                continue;
            }
            for &(j, c) in &self.adj[i] {
                relax(j as usize, c, &mut heap);
            }
            if let Some(c) = ate_t[i] {
                relax(n + 1, c, &mut heap);
            }
        }
        if best[n + 1].is_infinite() {
            return None;
        }
        let ponto = |k: usize| match k {
            k if k == n => s,
            k if k == n + 1 => t,
            k => self.nodes[k],
        };
        let mut pts = vec![t];
        let mut c = n + 1;
        while prev[c] != usize::MAX {
            c = prev[c];
            pts.push(ponto(c));
        }
        pts.reverse();
        Some((pts, best[n + 1]))
    }
}
