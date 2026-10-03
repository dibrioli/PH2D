//! ⭐⭐ (W7) **OS ATALHOS** (plano 30 §4, a linha W7) — um teleporte, uma porta de um sentido: duas
//! pontas que a malha não liga, e um custo próprio para as atravessar.
//!
//! # O plano com atalhos é um grafo PEQUENO sobre a procura de sempre
//!
//! Os nós são a partida, o alvo e as pontas de cada atalho (um atalho de dois sentidos são dois);
//! uma aresta entre dois nós é uma PROCURA real ([`Polyanya::find_path_costs`]), e a de um atalho é
//! o custo dele. Dijkstra, com as procuras PREGUIÇOSAS: só se corre a de `u` para `v` quando `u` sai
//! do heap e o limite inferior `g(u) + w_min·|u − v|` ainda pode melhorar `v`. O caminho mais curto
//! do grafo é o mais curto com atalhos: cada troço entre pontas já é o óptimo da malha.
//!
//! ⚠️ Sem atalhos, nada disto corre: é a procura de sempre (uma porta).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{V2, dist, ord_key};
use crate::mesh::NavMesh;
use crate::polyanya::Polyanya;

/// Um ATALHO de `from` a `to`. A ponte dá-lhe o `id` (quem é), e a lei devolve-o no [`Hop`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Link {
    pub id: u32,
    pub from: V2,
    pub to: V2,
    /// Também de `to` para `from`.
    pub two_way: bool,
    /// Teleporte: o corpo SALTA para `to` (custa só `cost`). Senão o agente ANDA a direito de `from`
    /// a `to` (custa o comprimento mais `cost`) — a porta de um sentido.
    pub teleport: bool,
    /// O custo a mais de atravessar, em metros de chão (`0` = só o que ele é).
    pub cost: f64,
}

impl Link {
    /// O custo de o atravessar.
    #[must_use]
    pub fn crossing_cost(&self) -> f64 {
        let andar = if self.teleport {
            0.0
        } else {
            dist(self.from, self.to)
        };
        andar + self.cost.max(0.0)
    }
}

/// Onde um atalho entra no caminho de um agente: o troço `path[at] → path[at + 1]` é ele.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hop {
    pub at: usize,
    pub link: u32,
    pub teleport: bool,
}

/// O que a consulta de um agente sabe além da malha: os custos das áreas (ver [`crate::cost`]) e os
/// atalhos.
#[derive(Clone, Copy, Debug, Default)]
pub struct Query<'a> {
    pub costs: &'a [f64],
    pub links: &'a [Link],
}

/// O caminho de `s` a `t` (os dois na malha) pelo grafo dos atalhos: os pontos, os atalhos no
/// meio, e o custo. `None` se nem com atalhos há caminho.
pub(crate) fn plan_with_links(
    mesh: &NavMesh,
    search: &mut Polyanya,
    q: &Query<'_>,
    s: V2,
    t: V2,
) -> Option<(Vec<V2>, Vec<Hop>, f64)> {
    // Os atalhos dirigidos, com as pontas postas na malha (o ponto mais perto, em qualquer ilha).
    let mut dirigidos: Vec<(V2, V2, Link)> = Vec::new();
    for l in q.links {
        let (Some((a, _)), Some((b, _))) = (
            mesh.nearest_point(l.from, None),
            mesh.nearest_point(l.to, None),
        ) else {
            continue;
        };
        dirigidos.push((a, b, *l));
        if l.two_way {
            dirigidos.push((b, a, *l));
        }
    }
    // Os nós: 0 = s, 1 = t, depois (entrada, saída) de cada atalho dirigido.
    let d = dirigidos.len();
    let n = 2 + 2 * d;
    let ponto = |i: usize| match i {
        0 => s,
        1 => t,
        i if (i - 2).is_multiple_of(2) => dirigidos[(i - 2) / 2].0,
        i => dirigidos[(i - 2) / 2].1,
    };
    let e_entrada = |i: usize| i >= 2 && (i - 2).is_multiple_of(2);
    let wmin = q.costs.iter().copied().fold(1.0, f64::min);
    let mut best = vec![f64::INFINITY; n];
    // De onde se chegou a cada nó, e o troço (os pontos) que lá trouxe — `None` = o atalho.
    let mut prev: Vec<Option<(usize, Option<Vec<V2>>)>> = vec![None; n];
    let mut done = vec![false; n];
    let mut heap = BinaryHeap::new();
    best[0] = 0.0;
    heap.push(Reverse((ord_key(0.0), 0usize)));
    while let Some(Reverse((_, u))) = heap.pop() {
        if done[u] {
            continue;
        }
        done[u] = true;
        if u == 1 {
            break;
        }
        if e_entrada(u) {
            // A entrada de um atalho só leva à saída dele.
            let v = u + 1;
            let c = best[u] + dirigidos[(u - 2) / 2].2.crossing_cost();
            if c < best[v] {
                best[v] = c;
                prev[v] = Some((u, None));
                heap.push(Reverse((ord_key(c), v)));
            }
            continue;
        }
        // Da partida ou de uma saída: pela malha até ao alvo e a cada entrada.
        for v in (1..n).filter(|&v| v == 1 || e_entrada(v)) {
            if done[v] || best[u] + wmin * dist(ponto(u), ponto(v)) >= best[v] {
                continue;
            }
            let Ok(p) = search.find_path_costs(mesh, q.costs, ponto(u), ponto(v)) else {
                continue;
            };
            let c = best[u] + p.cost;
            if c < best[v] {
                best[v] = c;
                prev[v] = Some((u, Some(p.points)));
                heap.push(Reverse((ord_key(c), v)));
            }
        }
    }
    if best[1].is_infinite() {
        return None;
    }
    // Os troços, do alvo para trás.
    let mut pedacos: Vec<(usize, Option<Vec<V2>>)> = Vec::new();
    let mut c = 1;
    while let Some((u, troco)) = prev[c].take() {
        pedacos.push((c, troco));
        c = u;
    }
    pedacos.reverse();
    let mut pts: Vec<V2> = vec![s];
    let mut hops = Vec::new();
    for (v, troco) in pedacos {
        match troco {
            Some(ps) => pts.extend(ps.into_iter().skip(1)),
            None => {
                let l = dirigidos[(v - 3) / 2].2;
                hops.push(Hop {
                    at: pts.len() - 1,
                    link: l.id,
                    teleport: l.teleport,
                });
                pts.push(ponto(v));
            }
        }
    }
    Some((pts, hops, best[1]))
}
