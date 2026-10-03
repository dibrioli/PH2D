//! ⭐⭐⭐ **POLYANYA** — o caminho MAIS CURTO POSSÍVEL, em qualquer ângulo, sobre a malha andável e sem
//! pré-processamento (Cui, Harabor & Grastien, *Compromise-free Pathfinding on a Navigation Mesh*,
//! IJCAI 2017). A lei sai do ARTIGO; ⛔ nenhum fonte foi lido (§0.9 do roteador).
//!
//! # Porque não o A\* + funil que a indústria usa
//!
//! O A\* escolhe um CORREDOR de polígonos pelo centro deles e o funil estica a corda **dentro** desse
//! corredor ⇒ o caminho é óptimo só no corredor que o A\* escolheu (o defeito que a pesquisa §3 do
//! doc 29 mediu no Godot). Aqui a fronteira da procura é feita de **intervalos** — pedaços de aresta
//! vistos de uma **raiz** — e o heurístico é exacto quando não há obstáculo ⇒ o primeiro alvo que sai
//! do heap é o óptimo. A régua é o ORÁCULO EXACTO ([`crate::oracle`]), que nenhum programa precisa de
//! correr: o caminho mais curto entre obstáculos poligonais passa pelos vértices deles (teorema).
//!
//! # O nó
//!
//! ```text
//!           a ●────────L━━━━━━━R───● b       a → b: a aresta de entrada, no sentido anti-horário de P
//!                       ╲     ╱             [L, R]: o intervalo VISTO da raiz
//!                        ╲   ╱              P: o polígono para onde o nó EXPANDE
//!                          ρ                ρ: a raiz (o início, ou um CANTO onde o caminho virou)
//! ```
//!
//! Visto de `ρ`, `a` fica à esquerda e `b` à direita (o interior de `P` está à esquerda de `a → b`).
//!
//! # As duas famílias de sucessor
//!
//! - **Observável** — a mesma raiz: o cone `ρ→R … ρ→L` corta as outras arestas de `P`, e cada pedaço
//!   com vizinho vira um nó. O pedaço de uma aresta é a solução de DUAS desigualdades LINEARES no
//!   parâmetro dela (*à esquerda do raio direito* e *à direita do raio esquerdo*) — sem ramos de
//!   «onde é que o raio sai», que é onde as implementações partem.
//! - **A volta num canto** — quando o extremo do intervalo É o vértice da aresta (`R == b`, `L == a`)
//!   e ele toca uma parede, o caminho pode virar ali: nasce uma raiz nova no vértice, e os nós dela
//!   cobrem só a SOMBRA — o pedaço de `P` do lado de fora do raio que passa pelo canto.
//!
//! ⚠️ **Divergência DECLARADA do artigo, e é deliberada:** na sombra, a aresta de `P` que TOCA o canto
//! daria um nó COLINEAR com a raiz nova (o caso especial que o artigo trata à parte e onde as
//! implementações partem). Aqui ela não vira nó: o polígono do outro lado tem o canto como vértice,
//! logo VÊ-SE inteiro dele, e a volta anda o LEQUE do canto (polígono a polígono, pelas arestas que
//! o tocam) até à parede, gerando as arestas que não o tocam. ⇒ **nenhum nó é colinear com a raiz,
//! por construção** (a raiz é vértice de um polígono convexo, logo está do lado de dentro das arestas
//! que não a tocam — a `ph2d-navmesh` torna-os ESTRITAMENTE convexos em aritmética inteira exacta).
//!
//! ⛔ **Medido e recusado: «gerar TUDO o que se vê do canto».** É a forma mais simples de não ter nós
//! colineares, e foi a primeira redacção desta crate: dá os mesmos caminhos (o oráculo concordou), e
//! a 1 000 obstáculos cada procura custava `19 ms` — cada canto re-inundava o que a raiz anterior já
//! via. A sombra é o que torna a procura barata, não um pormenor do artigo.
//!
//! A poda por raiz ([`Polyanya::root_g`]) corta uma volta só quando ela é ESTRITAMENTE pior: duas
//! voltas no mesmo canto com o mesmo `g` são sombras de polígonos diferentes do leque, e cortar a
//! segunda perdia-a.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use crate::geom::{EPS, V2, dist, dist_to_segment, lerp, ord_key, orient, same, side_dist, sub};
use crate::mesh::NavMesh;

#[path = "polyanya_custo.rs"]
mod custo;
#[path = "polyanya_dominancia.rs"]
mod dominancia;

/// Um caminho: os pontos por onde ele passa (o primeiro é a partida, o último o alvo, os do meio são
/// CANTOS da malha — e, com áreas de custo, os pontos onde ele atravessa uma fronteira), o
/// comprimento dele e o CUSTO (`∑ custo da área × comprimento`; sem áreas, igual ao comprimento).
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub points: Vec<V2>,
    pub length: f64,
    pub cost: f64,
}

/// Porque [`Polyanya::find_path`] não devolveu um caminho. Cada um tem cura diferente, e por isso são
/// três e não um `None` (o agente DIZ qual — queixa Q12 da pesquisa).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoPath {
    /// A partida está fora da malha (o chamador projecta-a antes, e diz que projectou).
    StartOff,
    /// O alvo está fora da malha.
    TargetOff,
    /// Os dois estão na malha, em ilhas diferentes.
    Unreachable,
}

/// O que a procura contou (para as tabelas de custo da W2 e para os gates de «não recalcula»).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub searches: u64,
    pub generated: u64,
    pub expanded: u64,
    pub turns: u64,
    pub pruned_turns: u64,
    /// (W7) Raízes nascidas onde um intervalo atravessou uma fronteira de custo.
    pub refractions: u64,
    /// (W7) Raízes de fronteira que não nasceram por estarem dominadas.
    pub pruned_refractions: u64,
    /// (W9) Nós que outra frente na mesma aresta dominava inteiros (não expandiram).
    pub dominated: u64,
    /// (W9) Nós cortados nas pontas por outra frente.
    pub trimmed: u64,
}

#[derive(Clone, Copy, Debug)]
struct Root {
    p: V2,
    g: f64,
    prev: u32,
    /// O custo da região do troço que CHEGA a esta raiz (vindo de `prev`).
    w_in: f64,
    /// (W7) Uma raiz de fronteira: o pedaço `[L, R]` da aresta onde ela pode deslizar no polimento
    /// (o que a raiz anterior VÊ pela região de `w_in`).
    range: Option<(V2, V2)>,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Interval {
        poly: u32,
        entry: u32,
        left: V2,
        right: V2,
    },
    Final {
        via: Option<V2>,
    },
    /// (W7) Uma raiz de fronteira PROMETIDA — materializa-se quando sai do heap (`polyanya_custo.rs`).
    Pending {
        idx: u32,
    },
}

/// De que lado do intervalo o caminho vira.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Right,
    Left,
}

#[derive(Clone, Copy, Debug)]
struct Node {
    root: u32,
    kind: Kind,
    /// O custo da região onde estão os troços da raiz até ao intervalo (`1` sem áreas).
    w: f64,
}

const NONE: u32 = u32::MAX;

/// A procura, com os buffers reaproveitados entre consultas (uma por agente basta — e é o que a ponte
/// faz). Nenhum estado sobrevive entre duas chamadas de [`Polyanya::find_path`].
#[derive(Debug, Default)]
pub struct Polyanya {
    open: BinaryHeap<Reverse<(u64, u64, u32)>>,
    nodes: Vec<Node>,
    roots: Vec<Root>,
    /// A melhor distância com que o caminho já VIROU em cada vértice — a poda por raiz do artigo.
    root_g: Vec<f64>,
    touched: Vec<u32>,
    target_polys: Vec<u32>,
    seq: u64,
    /// (W7) A tabela de custos desta consulta, o menor deles (o heurístico escala por ele) e as
    /// podas das raízes de fronteira — ver `polyanya_custo.rs`.
    costs: Vec<f64>,
    wmin: f64,
    steiner_g: std::collections::BTreeMap<(u32, u32, u32, u32), f64>,
    ponta_g: std::collections::BTreeMap<(u32, u32, u32), Vec<(f64, f64)>>,
    fan_g: Vec<f64>,
    pendentes: Vec<custo::Pendente>,
    steiner_override: Option<f64>,
    /// (W9) A dominância entre frentes (só na procura ponderada) — `polyanya_dominancia.rs`.
    dominancia: bool,
    sem_dominancia: bool,
    frentes: dominancia::Frentes,
    pub stats: Stats,
}

impl Polyanya {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self, mesh: &NavMesh) {
        self.open.clear();
        self.nodes.clear();
        self.roots.clear();
        if self.root_g.len() != mesh.verts().len() {
            self.root_g = vec![f64::INFINITY; mesh.verts().len()];
            self.fan_g = vec![f64::INFINITY; mesh.verts().len()];
            self.touched.clear();
        } else {
            for &v in &self.touched {
                self.root_g[v as usize] = f64::INFINITY;
                self.fan_g[v as usize] = f64::INFINITY;
            }
            self.touched.clear();
        }
        self.steiner_g.clear();
        self.ponta_g.clear();
        self.pendentes.clear();
        self.frentes.clear(mesh);
        self.seq = 0;
    }

    /// O caminho mais curto de `s` a `t`, os dois DENTRO da malha (fechada).
    pub fn find_path(&mut self, mesh: &NavMesh, s: V2, t: V2) -> Result<Path, NoPath> {
        self.find_path_costs(mesh, &[], s, t)
    }

    /// A procura (uniforme quando `costs` não distingue nada).
    fn search(&mut self, mesh: &NavMesh, costs: &[f64], s: V2, t: V2) -> Result<Path, NoPath> {
        self.reset(mesh);
        self.costs.clear();
        self.costs.extend_from_slice(costs);
        self.wmin = costs.iter().copied().fold(1.0, f64::min);
        let mut ps = Vec::new();
        mesh.locate_all(s, &mut ps);
        if ps.is_empty() {
            return Err(NoPath::StartOff);
        }
        mesh.locate_all(t, &mut self.target_polys);
        if self.target_polys.is_empty() {
            return Err(NoPath::TargetOff);
        }
        // No mesmo polígono: a direito (convexo), ao custo do mais barato que tem os dois. ⚠️ (W7) Só
        // é a resposta se nenhuma área custa MENOS: dentro da lama, sair e contornar pode ser mais
        // barato (medido) — então a recta é uma candidata no heap, como qualquer outra.
        let directo = ps
            .iter()
            .filter(|p| self.target_polys.binary_search(p).is_ok())
            .map(|&p| self.cost(mesh, p))
            .reduce(f64::min);
        if let Some(c) = directo
            && c <= self.wmin
        {
            return Ok(Path {
                points: vec![s, t],
                length: dist(s, t),
                cost: c * dist(s, t),
            });
        }
        let same_island = directo.is_some()
            || ps.iter().any(|&a| {
                self.target_polys
                    .iter()
                    .any(|&b| mesh.island(a) == mesh.island(b))
            });
        if !same_island {
            return Err(NoPath::Unreachable);
        }

        self.roots.push(Root {
            p: s,
            g: 0.0,
            prev: NONE,
            w_in: 0.0,
            range: None,
        });
        if let Some(c) = directo {
            self.push(
                c * dist(s, t),
                Node {
                    root: 0,
                    kind: Kind::Final { via: None },
                    w: c,
                },
            );
        }
        for &p in &ps {
            self.push_from_point(mesh, 0, p, None, t);
        }
        while let Some(Reverse((_, _, ni))) = self.open.pop() {
            let node = self.nodes[ni as usize];
            match node.kind {
                Kind::Final { via } => {
                    return Ok(self.reconstruct(mesh, node.root, via, node.w, t));
                }
                Kind::Interval {
                    poly,
                    entry,
                    left,
                    right,
                } => {
                    self.stats.expanded += 1;
                    self.expand(mesh, node.root, poly, entry, left, right, node.w, t);
                }
                Kind::Pending { idx } => self.materialize(mesh, idx, t),
            }
        }
        Err(NoPath::Unreachable)
    }

    fn push(&mut self, f: f64, node: Node) {
        let idx = self.nodes.len() as u32;
        self.nodes.push(node);
        self.seq += 1;
        self.stats.generated += 1;
        self.open.push(Reverse((ord_key(f), self.seq, idx)));
    }

    /// Os nós de tudo o que se vê de um PONTO dentro (ou no bordo) do polígono `p`: cada aresta com
    /// vizinho que não contém o ponto, inteira. É o nó inicial, e (W7) a raiz de uma fronteira.
    fn push_from_point(
        &mut self,
        mesh: &NavMesh,
        root: u32,
        p: u32,
        at_vertex: Option<u32>,
        t: V2,
    ) {
        let rp = self.roots[root as usize].p;
        let g = self.roots[root as usize].g;
        let c = self.cost(mesh, p);
        // O alvo num polígono que tem a raiz: vê-se a direito (convexo). A partida (raiz `0`) já
        // saiu pelo atalho do mesmo polígono.
        if root != 0 && self.target_polys.binary_search(&p).is_ok() {
            self.push(
                g + c * dist(rp, t),
                Node {
                    root,
                    kind: Kind::Final { via: None },
                    w: c,
                },
            );
        }
        let poly = mesh.poly(p);
        let n = poly.len();
        for i in 0..n {
            let Some(q) = poly.nbrs[i] else { continue };
            let u = poly.verts[i];
            let w = poly.verts[(i + 1) % n];
            if at_vertex.is_some_and(|v| v == u || v == w) {
                continue;
            }
            let (pu, pw) = (mesh.vert(u), mesh.vert(w));
            // A partida (que não é vértice) em cima de uma aresta: o polígono do outro lado também a
            // contém e gera os dele. ⚠️ Para uma raiz-vértice a pergunta é TOPOLÓGICA (acima): um
            // canto a menos de `EPS` de uma aresta que não é dele vê o outro lado — saltá-lo perdia-o.
            if at_vertex.is_none() && dist_to_segment(pu, pw, rp) <= EPS {
                continue;
            }
            // No vizinho a aresta é `w → u` (anti-horário dele): visto de cá, `w` à esquerda.
            self.push_interval(root, q, poly.twin[i], pw, pu, c, t);
        }
    }

    /// ⚠️ O heurístico escala pelo MENOR custo da tabela (admissível); sem áreas é `× 1`, ao bit.
    #[allow(clippy::too_many_arguments)]
    fn push_interval(
        &mut self,
        root: u32,
        poly: u32,
        entry: u32,
        left: V2,
        right: V2,
        w: f64,
        t: V2,
    ) {
        let r = self.roots[root as usize];
        let f = r.g + self.wmin * heuristic(r.p, left, right, t);
        self.push(
            f,
            Node {
                root,
                kind: Kind::Interval {
                    poly,
                    entry,
                    left,
                    right,
                },
                w,
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn expand(
        &mut self,
        mesh: &NavMesh,
        root: u32,
        p: u32,
        entry: u32,
        left: V2,
        right: V2,
        cw: f64,
        t: V2,
    ) {
        // (W7) O custo muda nesta aresta: o troço recto não continua — o caminho refracta.
        if self.cost(mesh, p) != cw {
            self.refract(mesh, root, p, entry, left, right, cw, t);
            return;
        }
        let (left, right) = if self.dominancia {
            let r = self.roots[root as usize];
            match self.domina(mesh, p, entry, r.p, r.g, cw, left, right) {
                Some(lr) => lr,
                None => return,
            }
        } else {
            (left, right)
        };
        let poly = mesh.poly(p);
        let n = poly.len();
        let k = entry as usize;
        let a = poly.verts[k];
        let b = poly.verts[(k + 1) % n];
        let r = self.roots[root as usize];
        let rho = r.p;

        if self.target_polys.binary_search(&p).is_ok() {
            let (h, via) = final_cost(rho, left, right, t);
            self.push(
                r.g + cw * h,
                Node {
                    root,
                    kind: Kind::Final { via },
                    w: cw,
                },
            );
        }

        // A volta nos cantos — só quando o extremo do intervalo É o vértice da aresta.
        if same(right, mesh.vert(b)) && mesh.is_corner(b) {
            self.turn(mesh, root, p, k, Side::Right, cw, t);
        }
        if same(left, mesh.vert(a)) && mesh.is_corner(a) {
            self.turn(mesh, root, p, k, Side::Left, cw, t);
        }

        // Os observáveis: o cone ρ→R … ρ→L sobre as outras arestas de P.
        let poly = mesh.poly(p);
        for step in 1..n {
            let e = (k + step) % n;
            let Some(q) = poly.nbrs[e] else { continue };
            let u = poly.verts[e];
            let w = poly.verts[(e + 1) % n];
            let (pu, pw) = (mesh.vert(u), mesh.vert(w));
            let mut lo = 0.0;
            let mut hi = 1.0;
            // À esquerda (ou em cima) do raio direito ρ → R …
            clip(
                &mut lo,
                &mut hi,
                side_dist(rho, right, pu),
                side_dist(rho, right, pw),
            );
            // … e à direita (ou em cima) do raio esquerdo ρ → L.
            clip(
                &mut lo,
                &mut hi,
                -side_dist(rho, left, pu),
                -side_dist(rho, left, pw),
            );
            self.push_portion(root, q, poly.twin[e], pu, pw, lo, hi, cw, t);
        }
    }

    /// O caminho vira no canto da aresta de entrada de `P` (o `b` à direita, o `a` à esquerda): uma
    /// raiz nova, e os nós da SOMBRA — o pedaço de `P` do lado de fora do raio que passa pelo canto,
    /// mais o leque do canto do mesmo lado, até à parede.
    #[allow(clippy::too_many_arguments)]
    fn turn(&mut self, mesh: &NavMesh, root: u32, p: u32, k: usize, side: Side, cw: f64, t: V2) {
        let poly = mesh.poly(p);
        let n = poly.len();
        let v = match side {
            Side::Right => poly.verts[(k + 1) % n],
            Side::Left => poly.verts[k],
        };
        let r = self.roots[root as usize];
        let pv = mesh.vert(v);
        let gv = r.g + cw * dist(r.p, pv);
        if self.root_g[v as usize] < gv - EPS {
            self.stats.pruned_turns += 1;
            return;
        }
        if self.root_g[v as usize].is_infinite() {
            self.touched.push(v);
        }
        self.root_g[v as usize] = self.root_g[v as usize].min(gv);
        self.stats.turns += 1;
        let ri = self.roots.len() as u32;
        self.roots.push(Root {
            p: pv,
            g: gv,
            prev: root,
            w_in: cw,
            range: None,
        });
        let rho = r.p;

        // A sombra em P: do lado de FORA do raio ρ → v (à direita do direito, à esquerda do esquerdo).
        let poly = mesh.poly(p);
        for step in 1..n {
            let e = (k + step) % n;
            let u = poly.verts[e];
            let w = poly.verts[(e + 1) % n];
            let (pu, pw) = (mesh.vert(u), mesh.vert(w));
            let (su, sw) = (side_dist(rho, pv, pu), side_dist(rho, pv, pw));
            let incident = u == v || w == v;
            if incident {
                // A aresta que toca o canto: na sombra só se a ponta de lá está estritamente fora.
                let far = if u == v { sw } else { su };
                let in_shadow = match side {
                    Side::Right => far < -EPS,
                    Side::Left => far > EPS,
                };
                if in_shadow && poly.nbrs[e].is_some() {
                    self.walk_fan(mesh, ri, p, e, v, side, t);
                }
                continue;
            }
            let Some(q) = poly.nbrs[e] else { continue };
            let mut lo = 0.0;
            let mut hi = 1.0;
            match side {
                Side::Right => clip(&mut lo, &mut hi, -su, -sw),
                Side::Left => clip(&mut lo, &mut hi, su, sw),
            }
            self.push_portion(ri, q, poly.twin[e], pu, pw, lo, hi, cw, t);
        }
    }

    /// Anda o leque do canto `v` a partir de `P`, atravessando a aresta `e` de `P` (que toca `v`), e
    /// continua do mesmo lado — no sentido horário para uma volta à direita, anti-horário à esquerda —
    /// até uma parede. Cada polígono do caminho tem `v` como vértice ⇒ vê-se inteiro de `v`.
    #[allow(clippy::too_many_arguments)]
    fn walk_fan(&mut self, mesh: &NavMesh, ri: u32, p: u32, e: usize, v: u32, side: Side, t: V2) {
        let mut cur_poly = p;
        let mut cur_edge = e;
        let cp = self.cost(mesh, p);
        // O leque de um vértice tem tantos polígonos quantos o tocam: é o tecto do laço (uma malha
        // bem-formada pára antes, na parede; o tecto só impede um laço numa malha partida).
        for _ in 0..mesh.polys_at_vertex(v).len() {
            let poly = mesh.poly(cur_poly);
            let Some(q) = poly.nbrs[cur_edge] else { return };
            let tw = poly.twin[cur_edge] as usize;
            // ⚠️ (W7) Uma fronteira de CUSTO é a parede do leque: entrar na área cara a partir do
            // canto é a refracção (a raiz-vértice emite o leque todo, uma vez). Medido sem isto: num
            // canto sem parede o leque dava a volta inteira e cada volta re-inundava a vizinhança, de
            // canto em canto (6,8 milhões de voltas numa cena de 200 polígonos).
            let cq = self.cost(mesh, q);
            if cq != cp {
                return;
            }
            if self.target_polys.binary_search(&q).is_ok() {
                let r = self.roots[ri as usize];
                self.push(
                    r.g + cq * dist(r.p, t),
                    Node {
                        root: ri,
                        kind: Kind::Final { via: None },
                        w: cq,
                    },
                );
            }
            let qp = mesh.poly(q);
            let m = qp.len();
            // Em Q a aresta atravessada é `tw`; a outra que toca `v` é a seguinte (à direita: v é o
            // fim de `tw`) ou a anterior (à esquerda: v é o início de `tw`).
            let next = match side {
                Side::Right => (tw + 1) % m,
                Side::Left => (tw + m - 1) % m,
            };
            debug_assert!(qp.verts[next] == v || qp.verts[(next + 1) % m] == v);
            for i in 0..m {
                if i == tw || i == next {
                    continue;
                }
                let Some(nq) = qp.nbrs[i] else { continue };
                let u = qp.verts[i];
                let w = qp.verts[(i + 1) % m];
                // No vizinho a aresta é `w → u`: visto de `v`, `w` à esquerda.
                self.push_interval(ri, nq, qp.twin[i], mesh.vert(w), mesh.vert(u), cq, t);
            }
            cur_poly = q;
            cur_edge = next;
        }
    }

    /// O nó do pedaço `[lo, hi]` da aresta `pu → pw` (no sentido do polígono que a expande), com as
    /// pontas encostadas ao vértice quando estão a menos de [`EPS`] dele.
    #[allow(clippy::too_many_arguments)]
    fn push_portion(
        &mut self,
        root: u32,
        q: u32,
        twin: u32,
        pu: V2,
        pw: V2,
        lo: f64,
        hi: f64,
        w: f64,
        t: V2,
    ) {
        if hi <= lo {
            return;
        }
        let elen = dist(pu, pw);
        if (hi - lo) * elen <= EPS {
            return;
        }
        let near_u = if lo * elen <= EPS {
            pu
        } else {
            lerp(pu, pw, lo)
        };
        let near_w = if (1.0 - hi) * elen <= EPS {
            pw
        } else {
            lerp(pu, pw, hi)
        };
        // No vizinho a aresta é `w → u`: o lado de `w` é a esquerda.
        self.push_interval(root, q, twin, near_w, near_u, w, t);
    }
}

/// Mantém de `[lo, hi]` só os `t` com `f(t) = f0 + t·(f1 − f0) ≥ 0` (à tolerância [`EPS`]).
pub(crate) fn clip(lo: &mut f64, hi: &mut f64, f0: f64, f1: f64) {
    if f0 >= -EPS && f1 >= -EPS {
        return;
    }
    if f0 < -EPS && f1 < -EPS {
        *hi = *lo - 1.0;
        return;
    }
    let tc = f0 / (f0 - f1);
    if f1 > f0 {
        *lo = lo.max(tc);
    } else {
        *hi = hi.min(tc);
    }
}

/// `t` está no cone `ρ → R … ρ → L` (fechado)?
fn in_cone(rho: V2, left: V2, right: V2, t: V2) -> bool {
    side_dist(rho, right, t) >= -EPS && side_dist(rho, left, t) <= EPS
}

/// O heurístico do artigo: a distância mais curta de `ρ` a `t` que passa pelo intervalo. Se `t` está
/// do mesmo lado da recta do intervalo que `ρ`, reflecte-se (o caminho tem de atravessar).
fn heuristic(rho: V2, left: V2, right: V2, t: V2) -> f64 {
    let sr = orient(left, right, rho);
    let st = orient(left, right, t);
    let t2 = if (sr > 0.0 && st > 0.0) || (sr < 0.0 && st < 0.0) {
        reflect(t, left, right)
    } else {
        t
    };
    final_cost(rho, left, right, t2).0
}

/// O custo EXACTO de `ρ` a `t` através do intervalo, quando `t` está do outro lado (ou na recta), e
/// o extremo por onde o caminho dobra se `t` está fora do cone.
fn final_cost(rho: V2, left: V2, right: V2, t: V2) -> (f64, Option<V2>) {
    if in_cone(rho, left, right, t) {
        return (dist(rho, t), None);
    }
    let via_l = dist(rho, left) + dist(left, t);
    let via_r = dist(rho, right) + dist(right, t);
    if via_l <= via_r {
        (via_l, Some(left))
    } else {
        (via_r, Some(right))
    }
}

fn reflect(p: V2, a: V2, b: V2) -> V2 {
    let d = sub(b, a);
    let l2 = d[0] * d[0] + d[1] * d[1];
    if l2 <= EPS * EPS {
        return p;
    }
    let ap = sub(p, a);
    let s = (ap[0] * d[0] + ap[1] * d[1]) / l2;
    let foot = [a[0] + d[0] * s, a[1] + d[1] * s];
    [2.0 * foot[0] - p[0], 2.0 * foot[1] - p[1]]
}
