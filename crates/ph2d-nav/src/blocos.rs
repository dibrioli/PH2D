//! ⭐⭐ (W10) **A MALHA POR BLOCOS** — a porta da malha que se monta a partir de pedaços RECTANGULARES
//! (os mosaicos da `ph2d-navmesh`), com o trabalho proporcional ao que MUDOU (plano 30 §18).
//!
//! A malha montada é a MESMA, campo a campo, que a de [`NavMesh::from_rings`] sobre os anéis cosidos da
//! montagem inteira (o oráculo dos gates na `ph2d-navmesh`): a mesma numeração de vértices (cada ponto
//! de costura é do 1.º bloco, pela ordem da chave, que o tem), as mesmas junções em T reparadas, os
//! mesmos vizinhos, cantos, ilhas e paredes. Muda QUEM a calcula — três camadas por bloco:
//!
//! - **L1** (só a peça): a grelha de localização do bloco, a caixa, os vértices de cada lado;
//! - **L2** (L1 dele e dos 8 vizinhos): o dono de cada ponto de costura, os anéis cosidos, a validação,
//!   a vizinhança interna, as componentes;
//! - **L3** (L2 dos dois lados de uma costura): a vizinhança através dela.
//!
//! e a montagem final só faz passagens lineares (numerar, traduzir, paredes, ilhas pela união das
//! componentes).
//!
//! # O contrato das peças
//!
//! Cada peça vive no seu rectângulo FECHADO `[lo, hi]`; um vértice está num lado se a coordenada é
//! EXACTAMENTE a do lado; e dois blocos vizinhos partilham a fronteira ao bit (`hi.x` de `(x, y)` é o
//! `lo.x` de `(x + 1, y)`). A `ph2d-navmesh` garante-o: tudo sai da mesma grelha inteira.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::geom::{EPS, V2, side_dist};
use crate::grelha::Grid;
use crate::mesh::{MeshError, NavMesh};

/// A chave de um bloco na grelha regular dos blocos.
pub type Chave = (i64, i64);

/// Os polígonos de UM bloco, com índices LOCAIS (como a [`NavMesh::from_rings`] os recebe), e o
/// rectângulo dele.
#[derive(Clone, Debug, Default)]
pub struct Peca {
    pub lo: V2,
    pub hi: V2,
    pub verts: Vec<V2>,
    pub ring_off: Vec<u32>,
    pub ring: Vec<u32>,
    pub area: Vec<u16>,
}

const ESQ: usize = 0;
const DIR: usize = 1;
const BAIXO: usize = 2;
const CIMA: usize = 3;
/// O vizinho de cada lado (`dx`, `dy`) e o lado dele que fica de frente.
const LADO: [(i64, i64, usize); 4] = [(-1, 0, DIR), (1, 0, ESQ), (0, -1, CIMA), (0, 1, BAIXO)];
/// A direcção de um vizinho, `0..9`; `4` é o próprio bloco.
const PROPRIO: u8 = 4;

fn dir(dx: i64, dy: i64) -> u8 {
    ((dx + 1) * 3 + (dy + 1)) as u8
}

fn vizinho((x, y): Chave, d: u8) -> Chave {
    (x + i64::from(d / 3) - 1, y + i64::from(d % 3) - 1)
}

/// Um vértice visto de um bloco: um vértice LOCAL dele, ou o `k`-ésimo do lado `s` do vizinho `d` — pela
/// POSIÇÃO no lado e não pelo índice local, para que um vizinho refeito com os mesmos pontos de lado não
/// obrigue a recoser este (medido: recoser os 8 vizinhos era `0,27 ms` de cada porta, plano 30 §18).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Vref(u32);

impl Vref {
    fn local(j: u32) -> Self {
        assert!(j < 1 << 28, "um bloco com mais de 2^28 vértices");
        Vref(u32::from(PROPRIO) << 28 | j)
    }
    fn lado(d: u8, s: usize, k: u32) -> Self {
        assert!(k < 1 << 26, "um lado com mais de 2^26 vértices");
        Vref(u32::from(d) << 28 | (s as u32) << 26 | k)
    }
    fn dir(self) -> u8 {
        (self.0 >> 28) as u8
    }
    /// O índice local no bloco dono (`dono` é o L1 do bloco da direcção [`Self::dir`]).
    fn em(self, dono: &L1) -> u32 {
        if self.dir() == PROPRIO {
            self.0 & ((1 << 28) - 1)
        } else {
            dono.lados[((self.0 >> 26) & 3) as usize][(self.0 & ((1 << 26) - 1)) as usize].1
        }
    }
}

/// L1 — o que só depende da peça.
#[derive(Clone, Debug, Default)]
struct L1 {
    /// Os vértices em cada lado: `(coordenada ao longo do lado, índice local CANÓNICO)`, por essa ordem.
    lados: [Vec<(f64, u32)>; 4],
    /// Os vértices de lado REPETIDOS na peça: `(índice, o 1.º com a mesma posição)`, por índice.
    repetidos: Vec<(u32, u32)>,
    grid: Option<Arc<Grid>>,
    caixa: Option<(V2, V2)>,
}

impl L1 {
    fn de(p: &Peca) -> L1 {
        let mut l = L1::default();
        for (i, q) in p.verts.iter().enumerate() {
            let i = i as u32;
            // ⚠️ É o que torna IMPOSSÍVEL a mesma aresta orientada em dois blocos (a sobreposição que a
            // porta inteira confere): num polígono anti-horário e convexo dentro do rectângulo, as
            // arestas do lado direito sobem e as do esquerdo do vizinho descem.
            assert!(
                q[0] >= p.lo[0] && q[0] <= p.hi[0] && q[1] >= p.lo[1] && q[1] <= p.hi[1],
                "o vértice {i} ({q:?}) está fora do rectângulo do bloco"
            );
            if q[0] == p.lo[0] {
                l.lados[ESQ].push((q[1], i));
            }
            if q[0] == p.hi[0] {
                l.lados[DIR].push((q[1], i));
            }
            if q[1] == p.lo[1] {
                l.lados[BAIXO].push((q[0], i));
            }
            if q[1] == p.hi[1] {
                l.lados[CIMA].push((q[0], i));
            }
        }
        for lado in &mut l.lados {
            lado.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            for w in lado.windows(2) {
                if w[0].0 == w[1].0 {
                    let primeiro = l.repetidos.iter().find(|r| r.0 == w[0].1).map_or(w[0].1, |r| r.1);
                    l.repetidos.push((w[1].1, primeiro));
                }
            }
        }
        l.repetidos.sort_unstable();
        l.repetidos.dedup_by_key(|r| r.0);
        for lado in 0..4 {
            for e in 0..l.lados[lado].len() {
                l.lados[lado][e].1 = l.canon(l.lados[lado][e].1);
            }
        }
        let np = p.ring_off.len() - 1;
        if np > 0 && p.ring.iter().all(|&v| (v as usize) < p.verts.len()) {
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for &v in &p.ring {
                let q = p.verts[v as usize];
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
            l.caixa = Some((lo, hi));
            l.grid = Some(Arc::new(Grid::build(&p.verts, &p.ring_off, &p.ring, lo, hi)));
        }
        l
    }

    /// O índice canónico de um vértice local (o 1.º com a mesma posição, num lado).
    fn canon(&self, i: u32) -> u32 {
        self.repetidos
            .binary_search_by_key(&i, |r| r.0)
            .map_or(i, |k| self.repetidos[k].1)
    }

    /// A posição no lado do vértice na coordenada `c`, se há.
    fn no_lado(&self, lado: usize, c: f64) -> Option<u32> {
        let l = &self.lados[lado];
        let k = l.partition_point(|e| e.0 < c);
        (k < l.len() && l[k].0 == c).then_some(k as u32)
    }

    /// Os mesmos pontos neste lado (as coordenadas, pela ordem)?
    fn mesmo_lado(&self, o: &L1, lado: usize) -> bool {
        let (a, b) = (&self.lados[lado], &o.lados[lado]);
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.0 == y.0)
    }
}

/// O que está do outro lado de uma aresta cosida.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Lig {
    Parede,
    /// Um polígono do mesmo bloco (`q`, e a aresta `e` dele).
    Dentro { q: u32, e: u32 },
    /// Um polígono do vizinho `d`.
    Fora { d: u8, q: u32, e: u32 },
}

/// L2 — o bloco cosido aos vizinhos.
#[derive(Clone, Debug, Default)]
struct L2 {
    /// Cada SLOT (os vértices locais, depois os de fora cosidos) e o vértice que ele é.
    slots: Vec<Vref>,
    pos: Vec<V2>,
    /// Os anéis cosidos, sobre os slots.
    s_off: Vec<u32>,
    s_ring: Vec<u32>,
    lig: Vec<Lig>,
    /// As arestas cosidas em cada lado: `(polígono, aresta, slot de, slot para)`.
    bordas: [Vec<(u32, u32, u32, u32)>; 4],
    comp: Vec<u32>,
    ncomp: u32,
    erro: Option<MeshError>,
}

#[derive(Clone, Debug, Default)]
struct Bloco {
    peca: Peca,
    l1: L1,
    l2: L2,
}

/// ⭐ **A malha por blocos.** [`Self::poe`] / [`Self::tira`] mudam blocos; [`Self::monta`] devolve a
/// malha, refazendo só o que depende do que mudou.
#[derive(Clone, Debug, Default)]
pub struct MalhaPorBlocos {
    blocos: BTreeMap<Chave, Bloco>,
    sujos: BTreeSet<Chave>,
}

impl MalhaPorBlocos {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Põe (ou troca) o bloco `k`.
    pub fn poe(&mut self, k: Chave, peca: Peca) {
        assert!(
            peca.ring_off.first() == Some(&0)
                && peca.ring_off.last() == Some(&(peca.ring.len() as u32))
                && peca.ring_off.windows(2).all(|w| w[0] <= w[1]),
            "anéis mal formados no bloco {k:?}"
        );
        let l1 = L1::de(&peca);
        // Quem tem de se recoser: este, e os vizinhos que lêem um lado que mudou (o de frente e as duas
        // diagonais desse lado) — todos, se o bloco é novo.
        match self.blocos.get(&k) {
            None => self.sujos.extend((0..9).map(|d| vizinho(k, d))),
            Some(velho) => {
                self.sujos.insert(k);
                for (lado, &(dx, dy, _)) in LADO.iter().enumerate() {
                    if !velho.l1.mesmo_lado(&l1, lado) {
                        let diag = if dx == 0 { [(-1, dy), (1, dy)] } else { [(dx, -1), (dx, 1)] };
                        for (ex, ey) in [(dx, dy), diag[0], diag[1]] {
                            self.sujos.insert((k.0 + ex, k.1 + ey));
                        }
                    }
                }
            }
        }
        self.blocos.insert(
            k,
            Bloco {
                peca,
                l1,
                ..Bloco::default()
            },
        );
    }

    /// Tira o bloco `k` (se há).
    pub fn tira(&mut self, k: Chave) {
        if self.blocos.remove(&k).is_some() {
            self.sujos.extend((0..9).map(|d| vizinho(k, d)));
        }
    }

    /// ⭐ A malha, com os blocos de agora. A recusa é a de [`NavMesh::from_rings`] (o índice do
    /// polígono e os vértices na numeração da malha montada).
    pub fn monta(&mut self) -> Result<NavMesh, MeshError> {
        let mut l2 = std::mem::take(&mut self.sujos);
        l2.retain(|k| self.blocos.contains_key(k));
        for &k in &l2 {
            let novo = self.cose(k);
            if let Some(b) = self.blocos.get_mut(&k) {
                b.l2 = novo;
            }
        }
        let mut l3: BTreeSet<(Chave, usize)> = BTreeSet::new();
        for &k in &l2 {
            for (lado, &(dx, dy, frente)) in LADO.iter().enumerate() {
                l3.insert((k, lado));
                let v = (k.0 + dx, k.1 + dy);
                if self.blocos.contains_key(&v) {
                    l3.insert((v, frente));
                }
            }
        }
        for (k, lado) in l3 {
            self.liga(k, lado);
        }
        junta::junta(self)
    }

    /// L2 do bloco `k`, lido dos vizinhos como estão.
    fn cose(&self, k: Chave) -> L2 {
        let b = &self.blocos[&k];
        let (p, l1) = (&b.peca, &b.l1);
        let nl = p.verts.len();
        let mut l = L2::default();
        // O dono de cada vértice local: o 1.º bloco, pela ordem da chave, que tem o ponto.
        for (i, q) in p.verts.iter().enumerate() {
            let c = l1.canon(i as u32);
            let mut r = Vref::local(c);
            if c == i as u32 {
                let (esq, baixo, cima) = (q[0] == p.lo[0], q[1] == p.lo[1], q[1] == p.hi[1]);
                let candidatos = [
                    (esq && baixo, -1, -1, DIR, q[1]),
                    (esq, -1, 0, DIR, q[1]),
                    (esq && cima, -1, 1, DIR, q[1]),
                    (baixo, 0, -1, CIMA, q[0]),
                ];
                for (sim, dx, dy, lado, coord) in candidatos {
                    if !sim {
                        continue;
                    }
                    let d = dir(dx, dy);
                    if let Some(j) = self
                        .blocos
                        .get(&vizinho(k, d))
                        .and_then(|n| n.l1.no_lado(lado, coord))
                    {
                        r = Vref::lado(d, lado, j);
                        break;
                    }
                }
            }
            l.slots.push(r);
            l.pos.push(*q);
        }
        // Os anéis cosidos: em cada aresta sobre um lado, os pontos do lado (deste bloco e do vizinho
        // de frente) estritamente entre as pontas, de uma para a outra.
        let mut de_fora: BTreeMap<(usize, u64), u32> = BTreeMap::new();
        l.s_off.push(0);
        let np = p.ring_off.len() - 1;
        for pi in 0..np {
            let anel = &p.ring[p.ring_off[pi] as usize..p.ring_off[pi + 1] as usize];
            if let Some(&v) = anel.iter().find(|&&v| v as usize >= nl) {
                l.erro.get_or_insert(MeshError::BadIndex {
                    poly: pi,
                    index: v,
                });
                l.s_off.push(l.s_ring.len() as u32);
                continue;
            }
            let n = anel.len();
            for i in 0..n {
                let (a, b) = (anel[i], anel[(i + 1) % n]);
                let (pa, pb) = (p.verts[a as usize], p.verts[b as usize]);
                l.s_ring.push(l1.canon(a));
                let lado = if pa[0] == pb[0] && (pa[0] == p.lo[0] || pa[0] == p.hi[0]) {
                    Some((if pa[0] == p.lo[0] { ESQ } else { DIR }, pa[1], pb[1]))
                } else if pa[1] == pb[1] && (pa[1] == p.lo[1] || pa[1] == p.hi[1]) {
                    Some((if pa[1] == p.lo[1] { BAIXO } else { CIMA }, pa[0], pb[0]))
                } else {
                    None
                };
                let Some((lado, ca, cb)) = lado else { continue };
                let (dx, dy, frente) = LADO[lado];
                let d = dir(dx, dy);
                let n_frente = self.blocos.get(&(k.0 + dx, k.1 + dy));
                let (lo, hi) = (ca.min(cb), ca.max(cb));
                let meus = &l1.lados[lado];
                let deles: &[(f64, u32)] = n_frente.map_or(&[], |n| &n.l1.lados[frente]);
                let fatia = |v: &[(f64, u32)]| {
                    let a = v.partition_point(|e| e.0 <= lo);
                    let b = v.partition_point(|e| e.0 < hi);
                    (a, b.max(a))
                };
                let (ma, mb) = fatia(meus);
                let (da, db) = fatia(deles);
                // A união por coordenada (a do bloco ganha), crescente.
                let mut pontos: Vec<(f64, Option<u32>, u32)> = Vec::new();
                let (mut x, mut y) = (ma, da);
                while x < mb || y < db {
                    let cm = (x < mb).then(|| meus[x].0);
                    let cd = (y < db).then(|| deles[y].0);
                    match (cm, cd) {
                        (Some(m), Some(e)) if m == e => {
                            pontos.push((m, Some(meus[x].1), 0));
                            x += 1;
                            y += 1;
                        }
                        (Some(m), Some(e)) if m < e => {
                            pontos.push((m, Some(meus[x].1), 0));
                            x += 1;
                        }
                        (Some(m), None) => {
                            pontos.push((m, Some(meus[x].1), 0));
                            x += 1;
                        }
                        (_, Some(e)) => {
                            pontos.push((e, None, y as u32));
                            y += 1;
                        }
                        (None, None) => unreachable!(),
                    }
                    // Repetidos do mesmo lado: a coordenada já entrou.
                    while x < mb && pontos.last().is_some_and(|u| u.0 == meus[x].0) {
                        x += 1;
                    }
                    while y < db && pontos.last().is_some_and(|u| u.0 == deles[y].0) {
                        y += 1;
                    }
                }
                if ca > cb {
                    pontos.reverse();
                }
                for (c, meu, j) in pontos {
                    let s = match meu {
                        Some(i) => i,
                        None => {
                            let n = n_frente.expect("um ponto de fora vem do vizinho");
                            *de_fora.entry((lado, c.to_bits())).or_insert_with(|| {
                                l.slots.push(Vref::lado(d, frente, j));
                                l.pos.push(n.peca.verts[n.l1.lados[frente][j as usize].1 as usize]);
                                (l.slots.len() - 1) as u32
                            })
                        }
                    };
                    l.s_ring.push(s);
                }
            }
            l.s_off.push(l.s_ring.len() as u32);
        }
        if l.erro.is_none() {
            l.erro = valida(&l);
        }
        if l.erro.is_none() {
            liga_dentro(&mut l, p);
        }
        l
    }

    /// L3: a vizinhança das arestas do lado `lado` do bloco `k` com o vizinho de frente.
    fn liga(&mut self, k: Chave, lado: usize) {
        let (dx, dy, frente) = LADO[lado];
        let d = dir(dx, dy);
        let v = (k.0 + dx, k.1 + dy);
        let Some(b) = self.blocos.get(&k) else { return };
        let abs = |chave: Chave, l: &L2, s: u32| {
            let r = l.slots[s as usize];
            let dono = vizinho(chave, r.dir());
            (dono, r.em(&self.blocos[&dono].l1))
        };
        let mut novas: Vec<(usize, Lig)> = Vec::new();
        if let Some(n) = self.blocos.get(&v) {
            let mut deles: Vec<(((Chave, u32), (Chave, u32)), u32, u32)> = n.l2.bordas[frente]
                .iter()
                .map(|&(q, e, u, w)| ((abs(v, &n.l2, u), abs(v, &n.l2, w)), q, e))
                .collect();
            deles.sort_unstable();
            // A aresta `(u, w)` deste casa com a `(w, u)` do vizinho. (A mesma orientação dos dois lados
            // é impossível — ver `L1::de`.)
            for &(q, e, u, w) in &b.l2.bordas[lado] {
                let chave = (abs(k, &b.l2, w), abs(k, &b.l2, u));
                if let Ok(i) = deles.binary_search_by(|x| x.0.cmp(&chave)) {
                    let pos = (b.l2.s_off[q as usize] + e) as usize;
                    novas.push((pos, Lig::Fora { d, q: deles[i].1, e: deles[i].2 }));
                }
            }
        }
        let b = self.blocos.get_mut(&k).expect("o bloco existe");
        for &(q, e, _, _) in &b.l2.bordas[lado] {
            let pos = (b.l2.s_off[q as usize] + e) as usize;
            if matches!(b.l2.lig[pos], Lig::Fora { .. }) {
                b.l2.lig[pos] = Lig::Parede;
            }
        }
        for (pos, lig) in novas {
            b.l2.lig[pos] = lig;
        }
    }
}

/// Confere cada polígono cosido como a [`NavMesh::from_rings`]: `≥ 3` vértices, anti-horário, convexo.
fn valida(l: &L2) -> Option<MeshError> {
    for (pi, w) in l.s_off.windows(2).enumerate() {
        let anel = &l.s_ring[w[0] as usize..w[1] as usize];
        let n = anel.len();
        if n < 3 {
            return Some(MeshError::Degenerate { poly: pi });
        }
        let mut area2 = 0.0;
        for i in 0..n {
            let a = l.pos[anel[i] as usize];
            let b = l.pos[anel[(i + 1) % n] as usize];
            area2 += a[0] * b[1] - b[0] * a[1];
        }
        if area2 <= 0.0 {
            return Some(MeshError::NotCcw { poly: pi, area2 });
        }
        for i in 0..n {
            let a = l.pos[anel[(i + n - 1) % n] as usize];
            let b = l.pos[anel[i] as usize];
            let c = l.pos[anel[(i + 1) % n] as usize];
            if side_dist(a, b, c) < -EPS {
                return Some(MeshError::NotConvex { poly: pi, at: i });
            }
        }
    }
    None
}

/// A vizinhança DENTRO do bloco, a sobreposição, as arestas de cada lado e as componentes.
fn liga_dentro(l: &mut L2, p: &Peca) {
    let ns = l.slots.len();
    let mut eo_off = vec![0u32; ns + 1];
    for &v in &l.s_ring {
        eo_off[v as usize + 1] += 1;
    }
    for i in 0..ns {
        eo_off[i + 1] += eo_off[i];
    }
    let mut cursor = eo_off.clone();
    let mut saidas = vec![(0u32, 0u32, 0u32); eo_off[ns] as usize];
    for (pi, w) in l.s_off.windows(2).enumerate() {
        let anel = &l.s_ring[w[0] as usize..w[1] as usize];
        let n = anel.len();
        for i in 0..n {
            let u = anel[i] as usize;
            saidas[cursor[u] as usize] = (anel[(i + 1) % n], pi as u32, i as u32);
            cursor[u] += 1;
        }
    }
    let de = |u: u32| &saidas[eo_off[u as usize] as usize..eo_off[u as usize + 1] as usize];
    l.lig = vec![Lig::Parede; l.s_ring.len()];
    for (pi, w) in l.s_off.windows(2).enumerate() {
        let o = w[0] as usize;
        let anel = &l.s_ring[o..w[1] as usize];
        let n = anel.len();
        for i in 0..n {
            let (u, v) = (anel[i], anel[(i + 1) % n]);
            if de(u).iter().any(|&(d, q, _)| d == v && (q as usize) < pi) {
                l.erro.get_or_insert(MeshError::NonManifold { a: u, b: v });
            }
            if let Some(&(_, q, j)) = de(v).iter().find(|&&(d, q, _)| d == u && q as usize != pi) {
                l.lig[o + i] = Lig::Dentro { q, e: j };
            }
            let (pu, pv) = (l.pos[u as usize], l.pos[v as usize]);
            let lado = if pu[0] == pv[0] && pu[0] == p.lo[0] {
                Some(ESQ)
            } else if pu[0] == pv[0] && pu[0] == p.hi[0] {
                Some(DIR)
            } else if pu[1] == pv[1] && pu[1] == p.lo[1] {
                Some(BAIXO)
            } else if pu[1] == pv[1] && pu[1] == p.hi[1] {
                Some(CIMA)
            } else {
                None
            };
            if let Some(lado) = lado {
                l.bordas[lado].push((pi as u32, i as u32, u, v));
            }
        }
    }
    // As componentes pela vizinhança interna, numeradas pela ordem do 1.º polígono.
    let np = l.s_off.len() - 1;
    l.comp = vec![u32::MAX; np];
    let mut pilha = Vec::new();
    for s in 0..np {
        if l.comp[s] != u32::MAX {
            continue;
        }
        l.comp[s] = l.ncomp;
        pilha.push(s);
        while let Some(q) = pilha.pop() {
            for lig in &l.lig[l.s_off[q] as usize..l.s_off[q + 1] as usize] {
                if let Lig::Dentro { q: r, .. } = *lig
                    && l.comp[r as usize] == u32::MAX
                {
                    l.comp[r as usize] = l.ncomp;
                    pilha.push(r as usize);
                }
            }
        }
        l.ncomp += 1;
    }
}

#[path = "blocos_junta.rs"]
mod junta;

#[cfg(test)]
#[path = "blocos_tests.rs"]
mod tests;
