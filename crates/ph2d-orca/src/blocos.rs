//! ⭐ (W11) **As paredes de uma malha por BLOCOS** (plano 30 §19): as MESMAS [`Walls`], ao bit, que
//! [`Walls::from_walkable_walls`] daria às paredes de todos os blocos postas em fila pela ordem da chave —
//! refazendo só o que depende dos blocos que mudaram.
//!
//! Um bloco é uma célula de uma grelha REGULAR de chaves inteiras (os vizinhos são as chaves `±1`), com
//! as suas paredes `(de, para)` DENTRO do rectângulo dele. Um vértice é o seu ponto, ao bit: um ponto de
//! DENTRO do rectângulo só aparece neste bloco; um da BORDA pode estar nos 8 vizinhos. Por bloco:
//!
//! | camada | depende de | o quê |
//! |---|---|---|
//! | L1 | só as paredes do bloco | o seguinte e o anterior pelos pontos de dentro, os pontos da borda, a grelha |
//! | L2 | L1 dele e dos 8 vizinhos | o seguinte e o anterior pelos pontos da borda, `dir`, `convex` |
//!
//! e [`ParedesPorBlocos::monta`] só concatena (os índices de um bloco são a base dele + o local).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::v2::{V2, left_of, normalize, sub};
use crate::walls::{Busca, Grade, Walls};

/// A chave de um bloco na grelha regular dos blocos.
pub type Chave = (i64, i64);

/// Um ponto ao bit — a identidade de um vértice.
type Ponto = u128;

fn bits(p: V2) -> Ponto {
    (u128::from(p[0].to_bits()) << 64) | u128::from(p[1].to_bits())
}

/// A direcção `d` de um vizinho, `0..9` (`4` = o próprio): pela ordem de `d` vão as chaves por ordem.
const PROPRIO: u8 = 4;

fn vizinho((x, y): Chave, d: u8) -> Chave {
    (x + i64::from(d / 3) - 1, y + i64::from(d % 3) - 1)
}

/// Uma entrada referida: deste bloco, ou do vizinho `d`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ref {
    Local(u32),
    Fora(u8, u32),
}

/// L1: tudo o que os pontos de DENTRO decidem. As entradas `pendentes` (o `de` ou o `para` na borda) têm
/// aqui um seguinte/anterior provisório; a L2 decide-as.
#[derive(Clone, Debug, Default)]
struct L1 {
    next: Vec<u32>,
    prev: Vec<u32>,
    dir: Vec<V2>,
    convex: Vec<bool>,
    pendentes: Vec<u32>,
    /// Os pontos da BORDA, ordenados: a 1.ª entrada que lá começa e a 1.ª que lá acaba (`u32::MAX` =
    /// nenhuma).
    borda: Vec<(Ponto, u32, u32)>,
    grade: Option<Arc<Grade>>,
}

/// L2: as entradas pendentes, decididas com os vizinhos.
#[derive(Clone, Debug)]
struct Pendente {
    j: u32,
    next: Ref,
    prev: Ref,
    dir: V2,
    convex: bool,
}

type L2 = Vec<Pendente>;

#[derive(Clone, Debug, Default)]
struct Bloco {
    lo: V2,
    hi: V2,
    /// As paredes `(de, para)` da malha; a entrada `k` está em `para` e vai a `de`.
    ends: Vec<(V2, V2)>,
    l1: L1,
    l2: L2,
}

/// ⭐ **As paredes por blocos.** [`Self::poe`] / [`Self::tira`] mudam blocos; [`Self::monta`] devolve
/// as [`Walls`].
#[derive(Clone, Debug, Default)]
pub struct ParedesPorBlocos {
    blocos: BTreeMap<Chave, Bloco>,
    sujos: BTreeSet<Chave>,
    buf: Vec<(V2, V2)>,
    contas: Contas,
}

/// O que a última [`ParedesPorBlocos::monta`] refez (desde a anterior).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Contas {
    /// Blocos postos com paredes DIFERENTES (L1 refeita).
    pub l1: usize,
    /// Blocos cosidos de novo (L2).
    pub l2: usize,
}

/// O ponto está na borda do rectângulo `[lo, hi]` (ao bit: os pontos da costura são os lados).
fn na_borda(lo: V2, hi: V2) -> impl Fn(V2) -> bool {
    move |p| p[0] == lo[0] || p[0] == hi[0] || p[1] == lo[1] || p[1] == hi[1]
}

fn mesmo_ponto(a: V2, b: V2) -> bool {
    bits(a) == bits(b)
}

impl ParedesPorBlocos {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Põe (ou troca) o bloco `k`, de rectângulo `[lo, hi]`, com as paredes `(de, para)` da malha
    /// andável (a convenção de [`Walls::from_walkable_walls`]). Iguais às de antes, ao bit = nada muda.
    ///
    /// # Panics
    /// Uma parede fora do rectângulo: é o que faz de um ponto de dentro um vértice só deste bloco.
    pub fn poe(&mut self, k: Chave, lo: V2, hi: V2, paredes: impl IntoIterator<Item = (V2, V2)>) {
        self.buf.clear();
        self.buf.extend(paredes);
        if let Some(b) = self.blocos.get(&k)
            && mesmo_ponto(b.lo, lo)
            && mesmo_ponto(b.hi, hi)
            && b.ends.len() == self.buf.len()
            && b.ends
                .iter()
                .zip(&self.buf)
                .all(|(x, y)| mesmo_ponto(x.0, y.0) && mesmo_ponto(x.1, y.1))
        {
            return;
        }
        let dentro = |p: V2| p[0] >= lo[0] && p[0] <= hi[0] && p[1] >= lo[1] && p[1] <= hi[1];
        for (i, &(de, para)) in self.buf.iter().enumerate() {
            assert!(
                dentro(de) && dentro(para),
                "a parede {i} ({de:?} → {para:?}) está fora do rectângulo do bloco {k:?}"
            );
        }
        let ends = std::mem::take(&mut self.buf);
        let l1 = L1::de(lo, hi, &ends);
        self.contas.l1 += 1;
        self.sujos.extend((0..9).map(|d| vizinho(k, d)));
        self.blocos.insert(
            k,
            Bloco {
                lo,
                hi,
                ends,
                l1,
                l2: Vec::new(),
            },
        );
    }

    /// Tira o bloco `k` (se há).
    pub fn tira(&mut self, k: Chave) {
        if self.blocos.remove(&k).is_some() {
            self.sujos.extend((0..9).map(|d| vizinho(k, d)));
        }
    }

    /// Tira os blocos cuja chave não `fica`.
    pub fn retem(&mut self, fica: impl Fn(Chave) -> bool) {
        let fora: Vec<Chave> = self.blocos.keys().copied().filter(|&k| !fica(k)).collect();
        for k in fora {
            self.tira(k);
        }
    }

    /// ⭐ As paredes, com os blocos de agora.
    ///
    /// # Panics
    /// Blocos que não formam uma grelha (um rectângulo que não é uma célula dos lados de todos).
    pub fn monta(&mut self) -> Walls {
        self.contas.l2 = 0;
        for k in std::mem::take(&mut self.sujos) {
            if self.blocos.contains_key(&k) {
                self.contas.l2 += 1;
                let l2 = self.cose(k);
                if let Some(b) = self.blocos.get_mut(&k) {
                    b.l2 = l2;
                }
            }
        }
        let mut base: BTreeMap<Chave, u32> = BTreeMap::new();
        let mut n = 0u32;
        for (&k, b) in &self.blocos {
            base.insert(k, n);
            n += b.ends.len() as u32;
        }
        let mut w = Walls::default();
        for v in [&mut w.next, &mut w.prev] {
            v.reserve(n as usize);
        }
        w.point.reserve(n as usize);
        w.dir.reserve(n as usize);
        w.convex.reserve(n as usize);
        for (&k, b) in &self.blocos {
            let b0 = base[&k];
            let global = |r: Ref| match r {
                Ref::Local(j) => b0 + j,
                Ref::Fora(d, j) => base[&vizinho(k, d)] + j,
            };
            w.point.extend(b.ends.iter().map(|e| e.1));
            w.next.extend(b.l1.next.iter().map(|&j| b0 + j));
            w.prev.extend(b.l1.prev.iter().map(|&j| b0 + j));
            w.dir.extend_from_slice(&b.l1.dir);
            w.convex.extend_from_slice(&b.l1.convex);
            for p in &b.l2 {
                let i = (b0 + p.j) as usize;
                w.next[i] = global(p.next);
                w.prev[i] = global(p.prev);
                w.dir[i] = p.dir;
                w.convex[i] = p.convex;
            }
        }
        w.busca = self.busca(&base);
        w
    }

    /// O que a última [`Self::monta`] refez; a seguinte recomeça a contar.
    pub fn contas(&mut self) -> Contas {
        let c = self.contas;
        self.contas.l1 = 0;
        c
    }

    /// A grelha regular dos blocos, com a grelha de cada um.
    fn busca(&self, base: &BTreeMap<Chave, u32>) -> Busca {
        let mut xs: Vec<f64> = self
            .blocos
            .values()
            .flat_map(|b| [b.lo[0], b.hi[0]])
            .collect();
        let mut ys: Vec<f64> = self
            .blocos
            .values()
            .flat_map(|b| [b.lo[1], b.hi[1]])
            .collect();
        for v in [&mut xs, &mut ys] {
            v.sort_by(f64::total_cmp);
            v.dedup();
        }
        let nx = xs.len().saturating_sub(1);
        let mut blocos = vec![None; nx * ys.len().saturating_sub(1)];
        for (k, b) in &self.blocos {
            let ix = xs.partition_point(|&f| f < b.lo[0]);
            let iy = ys.partition_point(|&f| f < b.lo[1]);
            assert!(
                xs.get(ix + 1) == Some(&b.hi[0]) && ys.get(iy + 1) == Some(&b.hi[1]),
                "o bloco {k:?} não é uma célula da grelha dos blocos"
            );
            if let Some(g) = &b.l1.grade {
                blocos[iy * nx + ix] = Some((base[k], Arc::clone(g)));
            }
        }
        Busca { xs, ys, blocos }
    }

    /// L2 do bloco `k`, lido dos vizinhos como estão.
    fn cose(&self, k: Chave) -> L2 {
        let b = &self.blocos[&k];
        let viz: [Option<&Bloco>; 9] =
            std::array::from_fn(|d| self.blocos.get(&vizinho(k, d as u8)));
        // A 1.ª entrada, pela ordem das chaves, que começa (`comeca`) ou acaba num ponto da borda.
        // Só os blocos cujo rectângulo tem o ponto: `-1`/`+1` num eixo só quando ele está nesse lado.
        let lados =
            |v: f64, lo: f64, hi: f64| (if v == lo { 0 } else { 1 }, if v == hi { 2 } else { 1 });
        let acha = |q: V2, comeca: bool| {
            let p = bits(q);
            let ((x0, x1), (y0, y1)) =
                (lados(q[0], b.lo[0], b.hi[0]), lados(q[1], b.lo[1], b.hi[1]));
            (x0..=x1)
                .flat_map(|x| (y0..=y1).map(move |y| (x * 3 + y) as u8))
                .find_map(|d| {
                    let c = viz[d as usize]?;
                    let i = c.l1.borda.binary_search_by(|x| x.0.cmp(&p)).ok()?;
                    let (_, s, e) = c.l1.borda[i];
                    let j = if comeca { s } else { e };
                    (j != u32::MAX).then_some(if d == PROPRIO {
                        Ref::Local(j)
                    } else {
                        Ref::Fora(d, j)
                    })
                })
        };
        let ponto = |r: Ref| match r {
            Ref::Local(j) => b.ends[j as usize].1,
            Ref::Fora(d, j) => {
                viz[d as usize].expect("o vizinho onde a borda achou").ends[j as usize].1
            }
        };
        let na_borda = na_borda(b.lo, b.hi);
        b.l1.pendentes
            .iter()
            .map(|&j| {
                let (de, para) = b.ends[j as usize];
                let eu = Ref::Local(j);
                let nx = if na_borda(de) {
                    acha(de, true).unwrap_or(eu)
                } else {
                    Ref::Local(b.l1.next[j as usize])
                };
                let pv = if na_borda(para) {
                    acha(para, false).unwrap_or(eu)
                } else {
                    Ref::Local(b.l1.prev[j as usize])
                };
                let (pn, pp) = (ponto(nx), ponto(pv));
                Pendente {
                    j,
                    next: nx,
                    prev: pv,
                    dir: normalize(sub(pn, para)),
                    convex: pv == nx || left_of(pp, para, pn) >= 0.0,
                }
            })
            .collect()
    }
}

#[cfg(test)]
#[path = "blocos_tests.rs"]
mod tests;

impl L1 {
    fn de(lo: V2, hi: V2, ends: &[(V2, V2)]) -> Self {
        let na_borda = na_borda(lo, hi);
        let n = ends.len();
        // As entradas pelo ponto onde começam (`para`) e onde acabam (`de`); a 1.ª de cada ponto é a de
        // menor índice.
        let ordena = |f: fn(&(V2, V2)) -> V2| {
            let mut v: Vec<(Ponto, u32)> = ends
                .iter()
                .enumerate()
                .map(|(k, e)| (bits(f(e)), k as u32))
                .collect();
            v.sort_unstable();
            v
        };
        let (por_para, por_de) = (ordena(|e| e.1), ordena(|e| e.0));
        let primeiras = |v: &[(Ponto, u32)]| {
            let mut d = v.to_vec();
            d.dedup_by_key(|x| x.0);
            d
        };
        let (comeca, acaba) = (primeiras(&por_para), primeiras(&por_de));
        // O seguinte de `k` é a 1.ª que começa no `de` dele; o anterior, a 1.ª que acaba no `para` —
        // duas junções lineares de listas ordenadas.
        let junta = |quem: &[(Ponto, u32)], alvo: &[(Ponto, u32)]| {
            let mut out = vec![0u32; n];
            let mut i = 0;
            for &(p, k) in quem {
                while i < alvo.len() && alvo[i].0 < p {
                    i += 1;
                }
                out[k as usize] = if i < alvo.len() && alvo[i].0 == p {
                    alvo[i].1
                } else {
                    k
                };
            }
            out
        };
        let mut l = Self {
            next: junta(&por_de, &comeca),
            prev: junta(&por_para, &acaba),
            dir: Vec::with_capacity(n),
            convex: Vec::with_capacity(n),
            ..Self::default()
        };
        for (k, &(de, para)) in ends.iter().enumerate() {
            if na_borda(de) || na_borda(para) {
                // Decide-a a L2 (o seguinte/anterior daqui é provisório).
                l.pendentes.push(k as u32);
                l.dir.push([0.0; 2]);
                l.convex.push(false);
            } else {
                let (nx, pv) = (l.next[k], l.prev[k]);
                let (pn, pp) = (ends[nx as usize].1, ends[pv as usize].1);
                l.dir.push(normalize(sub(pn, para)));
                l.convex.push(pv == nx || left_of(pp, para, pn) >= 0.0);
            }
        }
        // Os pontos da borda: a junção das duas listas ordenadas.
        let pb = |p: Ponto| na_borda([f64::from_bits((p >> 64) as u64), f64::from_bits(p as u64)]);
        let (mut c, mut a) = (
            comeca.iter().copied().filter(|x| pb(x.0)).peekable(),
            acaba.iter().copied().filter(|x| pb(x.0)).peekable(),
        );
        loop {
            let e = match (c.peek().copied(), a.peek().copied()) {
                (None, None) => break,
                (Some(x), Some(y)) if x.0 == y.0 => {
                    c.next();
                    a.next();
                    (x.0, x.1, y.1)
                }
                (Some(x), Some(y)) if x.0 < y.0 => {
                    c.next();
                    (x.0, x.1, u32::MAX)
                }
                (Some(x), None) => {
                    c.next();
                    (x.0, x.1, u32::MAX)
                }
                (_, Some(y)) => {
                    a.next();
                    (y.0, u32::MAX, y.1)
                }
            };
            l.borda.push(e);
        }
        l.grade = (!ends.is_empty())
            .then(|| Arc::new(Grade::new(ends.len(), |i| (ends[i].1, ends[i].0))));
        l
    }
}
