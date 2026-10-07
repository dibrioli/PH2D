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

use crate::cota::{Cota, inferior};
use crate::geom::{V2, dist, ord_key};
use crate::mesh::NavMesh;
use crate::polyanya::fatias::Custos;
use crate::polyanya::{Path, Polyanya};

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

/// O caminho com atalhos: os pontos, os atalhos no meio, e o custo.
pub(crate) type ComAtalhos = Option<(Vec<V2>, Vec<Hop>, f64)>;

/// ⭐ (W15) O Dijkstra dos atalhos em FATIAS: o estado do grafo e a procura da malha a meio.
#[derive(Debug)]
pub(crate) struct Atalhos {
    /// Os atalhos dirigidos, com as pontas postas na malha (o ponto mais perto, em qualquer ilha).
    dirigidos: Vec<(V2, V2, Link)>,
    s: V2,
    t: V2,
    /// (W19) O menor custo e a distância de cada nó às áreas baratas: a cota de um troço ([`crate::cota`]).
    wmin: f64,
    d: Vec<f64>,
    best: Vec<f64>,
    /// De onde se chegou a cada nó, e o troço (os pontos) que lá trouxe — `None` = o atalho.
    prev: Vec<Option<(usize, Option<Vec<V2>>)>>,
    done: Vec<bool>,
    heap: BinaryHeap<Reverse<(u64, usize)>>,
    /// O nó que saiu do heap e o próximo vizinho dele a tentar.
    u: Option<(usize, usize)>,
    /// A procura de `u` para este vizinho, a meio.
    troco: Option<(usize, Custos)>,
}

impl Atalhos {
    pub(crate) fn new(mesh: &NavMesh, q: &Query<'_>, s: V2, t: V2) -> Self {
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
        let n = 2 + 2 * dirigidos.len();
        let mut best = vec![f64::INFINITY; n];
        let mut heap = BinaryHeap::new();
        best[0] = 0.0;
        heap.push(Reverse((ord_key(0.0), 0usize)));
        let cota = Cota::nova(mesh, q.costs, false);
        let mut d = vec![cota.distancia(s), cota.distancia(t)];
        for (a, b, _) in &dirigidos {
            d.extend([cota.distancia(*a), cota.distancia(*b)]);
        }
        Self {
            dirigidos,
            s,
            t,
            wmin: cota.w(),
            d,
            best,
            prev: vec![None; n],
            done: vec![false; n],
            heap,
            u: None,
            troco: None,
        }
    }

    fn ponto(&self, i: usize) -> V2 {
        match i {
            0 => self.s,
            1 => self.t,
            i if e_entrada(i) => self.dirigidos[(i - 2) / 2].0,
            i => self.dirigidos[(i - 2) / 2].1,
        }
    }

    /// O troço de `u` a `v` acabou.
    fn relaxa(&mut self, u: usize, v: usize, p: Path) {
        let c = self.best[u] + p.cost;
        if c < self.best[v] {
            self.best[v] = c;
            self.prev[v] = Some((u, Some(p.points)));
            self.heap.push(Reverse((ord_key(c), v)));
        }
    }

    /// Continua até à resposta, ou até o trabalho de `search` passar `ate`.
    pub(crate) fn run(
        &mut self,
        mesh: &NavMesh,
        search: &mut Polyanya,
        q: &Query<'_>,
        ate: u64,
    ) -> Option<ComAtalhos> {
        let n = self.best.len();
        loop {
            if let Some((v, c)) = &mut self.troco {
                let v = *v;
                let r = c.run(search, mesh, q.costs, ate)?;
                self.troco = None;
                if let (Ok(p), Some((u, _))) = (r, self.u) {
                    self.relaxa(u, v, p);
                }
            }
            // Da partida ou de uma saída: pela malha até ao alvo e a cada entrada.
            if let Some((u, mut v)) = self.u {
                while v < n {
                    let atual = v;
                    v += 1;
                    if !(atual == 1 || e_entrada(atual))
                        || self.done[atual]
                        || self.best[u]
                            + inferior(
                                self.wmin,
                                dist(self.ponto(u), self.ponto(atual)),
                                self.d[u] + self.d[atual],
                            )
                            >= self.best[atual]
                    {
                        continue;
                    }
                    self.u = Some((u, v));
                    match Custos::begin(search, mesh, q.costs, self.ponto(u), self.ponto(atual)) {
                        Ok(c) => {
                            self.troco = Some((atual, c));
                            break;
                        }
                        Err(Ok(p)) => self.relaxa(u, atual, p),
                        Err(Err(_)) => {}
                    }
                }
                if self.troco.is_some() {
                    continue;
                }
                self.u = None;
            }
            let Some(Reverse((_, u))) = self.heap.pop() else {
                return Some(self.caminho());
            };
            if self.done[u] {
                continue;
            }
            self.done[u] = true;
            if u == 1 {
                return Some(self.caminho());
            }
            if e_entrada(u) {
                // A entrada de um atalho só leva à saída dele.
                let v = u + 1;
                let c = self.best[u] + self.dirigidos[(u - 2) / 2].2.crossing_cost();
                if c < self.best[v] {
                    self.best[v] = c;
                    self.prev[v] = Some((u, None));
                    self.heap.push(Reverse((ord_key(c), v)));
                }
                continue;
            }
            self.u = Some((u, 1));
        }
    }

    /// O caminho do grafo, do alvo para trás.
    fn caminho(&mut self) -> ComAtalhos {
        if self.best[1].is_infinite() {
            return None;
        }
        let mut pedacos: Vec<(usize, Option<Vec<V2>>)> = Vec::new();
        let mut c = 1;
        while let Some((u, troco)) = self.prev[c].take() {
            pedacos.push((c, troco));
            c = u;
        }
        pedacos.reverse();
        let mut pts: Vec<V2> = vec![self.s];
        let mut hops = Vec::new();
        for (v, troco) in pedacos {
            match troco {
                Some(ps) => pts.extend(ps.into_iter().skip(1)),
                None => {
                    let l = self.dirigidos[(v - 3) / 2].2;
                    hops.push(Hop {
                        at: pts.len() - 1,
                        link: l.id,
                        teleport: l.teleport,
                    });
                    pts.push(self.ponto(v));
                }
            }
        }
        Some((pts, hops, self.best[1]))
    }
}

/// O nó `i` é a ENTRADA de um atalho dirigido.
fn e_entrada(i: usize) -> bool {
    i >= 2 && (i - 2).is_multiple_of(2)
}
