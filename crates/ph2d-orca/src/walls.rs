//! **As paredes** como o ORCA as lê: cadeias FECHADAS de arestas, com o espaço livre à DIREITA de cada
//! uma (a convenção do artigo: um obstáculo é um polígono anti-horário e o agente anda fora dele).
//!
//! Cada entrada `i` é um vértice e a aresta que sai dele para [`Walls::next`]. Um vértice é
//! CONVEXO quando a cadeia vira à esquerda nele (visto do espaço livre, é uma quina que se contorna);
//! num CÔNCAVO (o canto de uma sala) a perna do cone prolonga a parede.

use std::sync::Arc;

use crate::v2::{V2, abs_sq, dist_sq_to_segment, left_of, normalize, sub};

#[derive(Clone, Debug, Default)]
pub struct Walls {
    pub(crate) point: Vec<V2>,
    pub(crate) next: Vec<u32>,
    pub(crate) prev: Vec<u32>,
    pub(crate) dir: Vec<V2>,
    pub(crate) convex: Vec<bool>,
    pub(crate) busca: Busca,
}

/// (W11) **A busca de [`Walls::near`]: uma [`Grade`] por BLOCO**, numa grelha regular de blocos
/// (`xs` × `ys`, os lados de todos os blocos). As paredes de um bloco estão DENTRO do rectângulo dele, e
/// a grelha de cada um tem índices LOCAIS (`base` + local = o da parede): um bloco mudado não toca nas
/// grelhas dos outros ([`crate::ParedesPorBlocos`]). Uma construção inteira é UM bloco, a caixa.
#[derive(Clone, Debug, Default)]
pub(crate) struct Busca {
    pub(crate) xs: Vec<f64>,
    pub(crate) ys: Vec<f64>,
    pub(crate) blocos: Vec<Option<(u32, Arc<Grade>)>>,
}

impl Busca {
    fn um_bloco(g: Grade) -> Self {
        if g.nx == 0 {
            return Self::default();
        }
        Self {
            xs: vec![g.min[0], g.max[0]],
            ys: vec![g.min[1], g.max[1]],
            blocos: vec![Some((0, Arc::new(g)))],
        }
    }
}

/// ⭐ (W9) **A grelha das arestas** — [`Walls::near`] lê só as células ao alcance, e não todas as
/// arestas. Medido (`medir_replaneio` da ponte, `FASES=200`, `100 × 100 m`, `1 000` caixas): o desvio
/// era `97 %` do tique a 200 agentes (`35 ms` de `36`), por varrer as arestas da malha inteira por
/// agente. ⚠️ A resposta é a MESMA, ao bit, da varredura inteira (gate): os mesmos candidatos, a mesma
/// distância, a mesma ordem.
///
/// O lado da célula sai da CONTAGEM (`~1` aresta por célula, `√n` células por eixo sobre a maior
/// dimensão — a regra da grelha da `NavMesh`).
#[derive(Clone, Debug, Default)]
pub(crate) struct Grade {
    min: V2,
    max: V2,
    cell: f64,
    nx: usize,
    ny: usize,
    /// As listas das células, contíguas: a célula `c` é `items[off[c]..off[c + 1]]`.
    off: Vec<u32>,
    items: Vec<u32>,
}

impl Grade {
    /// A grelha das arestas `seg(0..n)` (índices `0..n`).
    pub(crate) fn new(n: usize, seg: impl Fn(usize) -> (V2, V2)) -> Self {
        if n == 0 {
            return Self::default();
        }
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for i in 0..n {
            let (a, b) = seg(i);
            lo = [lo[0].min(a[0]).min(b[0]), lo[1].min(a[1]).min(b[1])];
            hi = [hi[0].max(a[0]).max(b[0]), hi[1].max(a[1]).max(b[1])];
        }
        let ext = (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-9);
        let cell = ext / (n as f64).sqrt().ceil().max(1.0);
        let nx = (((hi[0] - lo[0]) / cell).floor() as usize + 1).max(1);
        let ny = (((hi[1] - lo[1]) / cell).floor() as usize + 1).max(1);
        let mut g = Self {
            min: lo,
            max: hi,
            cell,
            nx,
            ny,
            off: vec![0; nx * ny + 1],
            items: Vec::new(),
        };
        let caixas: Vec<_> = (0..n)
            .map(|i| {
                let (a, b) = seg(i);
                (
                    g.celula([a[0].min(b[0]), a[1].min(b[1])]),
                    g.celula([a[0].max(b[0]), a[1].max(b[1])]),
                )
            })
            .collect();
        for &((ax, ay), (bx, by)) in &caixas {
            for y in ay..=by {
                for x in ax..=bx {
                    g.off[y * nx + x + 1] += 1;
                }
            }
        }
        for c in 0..nx * ny {
            g.off[c + 1] += g.off[c];
        }
        let mut cursor = g.off.clone();
        g.items = vec![0; g.off[nx * ny] as usize];
        for (i, &((ax, ay), (bx, by))) in caixas.iter().enumerate() {
            for y in ay..=by {
                for x in ax..=bx {
                    let c = y * nx + x;
                    g.items[cursor[c] as usize] = i as u32;
                    cursor[c] += 1;
                }
            }
        }
        g
    }

    /// A célula de `p`, presa à grelha (um ponto fora cai na da borda).
    fn celula(&self, p: V2) -> (usize, usize) {
        let f = |v: f64, m: f64, n: usize| {
            let k = ((v - m) / self.cell).floor();
            if k.is_nan() {
                0
            } else {
                k.clamp(0.0, (n - 1) as f64) as usize
            }
        };
        (f(p[0], self.min[0], self.nx), f(p[1], self.min[1], self.ny))
    }
}

impl Walls {
    /// Polígonos-OBSTÁCULO, cada um anti-horário (área com sinal positiva): o agente anda fora deles.
    /// Um polígono de dois pontos é um segmento solto. ⚠️ Um polígono horário prende o agente DENTRO
    /// dele — medido no Godot, que tem a mesma convenção (a 1.ª corrida do oráculo do desvio).
    #[must_use]
    pub fn from_polygons(polys: &[Vec<V2>]) -> Self {
        let mut w = Self::default();
        for poly in polys {
            let n = poly.len();
            if n < 2 {
                continue;
            }
            let base = w.point.len() as u32;
            for (k, &p) in poly.iter().enumerate() {
                w.point.push(p);
                w.next.push(base + ((k + 1) % n) as u32);
                w.prev.push(base + ((k + n - 1) % n) as u32);
            }
        }
        w.finish();
        w
    }

    /// ⭐ **As paredes de uma MALHA ANDÁVEL**: `walls` são as arestas `(de, para)` da fronteira com o
    /// espaço ANDÁVEL à esquerda (o sentido dos polígonos da malha). Invertidas, ficam com ele à
    /// direita — a convenção do ORCA.
    ///
    /// ⚠️ Num vértice onde a fronteira se TOCA a si própria (duas quinas encostadas) há duas
    /// continuações; fica a de menor índice — a ordem é a da malha, igual nos três sistemas.
    #[must_use]
    pub fn from_walkable_walls(verts: &[V2], walls: &[(u32, u32)]) -> Self {
        // A aresta invertida `para → de` é a entrada `k`, no ponto `para`.
        let n = walls.len();
        let mut w = Self::default();
        // ⚠️ Indexados pelo VÉRTICE (a 1.ª entrada que lá começa / acaba; `u32::MAX` = nenhuma) — eram
        // dois `BTreeMap` e custavam `3 ms` a cada mudança de uma malha de `11 000` polígonos (W6).
        let mut starting_at = vec![u32::MAX; verts.len()];
        let mut ending_at = vec![u32::MAX; verts.len()];
        for (k, &(de, para)) in walls.iter().enumerate() {
            w.point.push(verts[para as usize]);
            if starting_at[para as usize] == u32::MAX {
                starting_at[para as usize] = k as u32;
            }
            if ending_at[de as usize] == u32::MAX {
                ending_at[de as usize] = k as u32;
            }
        }
        for &(de, para) in walls {
            // A entrada `k` vai de `para` a `de`; a seguinte começa em `de`, a anterior acaba em `para`.
            let k = w.next.len() as u32;
            let ou = |v: u32| if v == u32::MAX { k } else { v };
            w.next.push(ou(starting_at[de as usize]));
            w.prev.push(ou(ending_at[para as usize]));
        }
        debug_assert_eq!(w.next.len(), n);
        w.finish();
        w
    }

    fn finish(&mut self) {
        let n = self.point.len();
        self.dir = (0..n)
            .map(|i| normalize(sub(self.point[self.next[i] as usize], self.point[i])))
            .collect();
        self.convex = (0..n)
            .map(|i| {
                let p = self.prev[i] as usize;
                let q = self.next[i] as usize;
                // Um segmento solto (dois pontos) tem as duas pontas convexas.
                p == q || left_of(self.point[p], self.point[i], self.point[q]) >= 0.0
            })
            .collect();
        let (point, next) = (&self.point, &self.next);
        self.busca = Busca::um_bloco(Grade::new(n, |i| (point[i], point[next[i] as usize])));
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.point.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.point.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn point(&self, i: usize) -> V2 {
        self.point[i]
    }

    #[inline]
    #[must_use]
    pub fn next(&self, i: usize) -> usize {
        self.next[i] as usize
    }

    #[inline]
    #[must_use]
    pub fn prev(&self, i: usize) -> usize {
        self.prev[i] as usize
    }

    /// A direcção unitária da aresta que sai de `i`.
    #[inline]
    #[must_use]
    pub fn dir(&self, i: usize) -> V2 {
        self.dir[i]
    }

    #[inline]
    #[must_use]
    pub fn convex(&self, i: usize) -> bool {
        self.convex[i]
    }

    /// **As paredes que contam para um agente em `pos`**: as que olham para ele (ele está do lado
    /// LIVRE) e estão a menos de `range` — pela ordem da distância, e do índice num empate. A ordem
    /// importa: uma parede cujo cone já está coberto pelas anteriores não dá semi-plano.
    pub fn near(&self, pos: V2, range: f64, out: &mut Vec<u32>) {
        out.clear();
        if self.point.is_empty() || range.is_nan() || range <= 0.0 {
            return;
        }
        let range_sq = range * range;
        let mut found: Vec<(f64, u32)> = Vec::new();
        let (lo, hi) = (
            [pos[0] - range, pos[1] - range],
            [pos[0] + range, pos[1] + range],
        );
        let b = &self.busca;
        let (nx, ny) = (b.xs.len().saturating_sub(1), b.ys.len().saturating_sub(1));
        if nx == 0 || ny == 0 {
            return;
        }
        // Os blocos cujo rectângulo (fechado) toca a caixa do alcance.
        let (ix0, ix1) = (
            b.xs[1..].partition_point(|&x| x < lo[0]),
            b.xs[..nx].partition_point(|&x| x <= hi[0]),
        );
        let (iy0, iy1) = (
            b.ys[1..].partition_point(|&y| y < lo[1]),
            b.ys[..ny].partition_point(|&y| y <= hi[1]),
        );
        for by in iy0..iy1 {
            for bx in ix0..ix1 {
                let Some((base, g)) = &b.blocos[by * nx + bx] else {
                    continue;
                };
                let (ax, ay) = g.celula(lo);
                let (cx, cy) = g.celula(hi);
                for y in ay..=cy {
                    for x in ax..=cx {
                        let c = y * g.nx + x;
                        for &i in &g.items[g.off[c] as usize..g.off[c + 1] as usize] {
                            let i = base + i;
                            if let Some(d) = self.candidata(i as usize, pos, range_sq) {
                                found.push((d, i));
                            }
                        }
                    }
                }
            }
        }
        found.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        // Uma aresta comprida está em várias células: o mesmo `(d, i)` fica adjacente na ordem.
        found.dedup_by_key(|x| x.1);
        out.extend(found.into_iter().map(|(_, i)| i));
    }

    /// A aresta `i` olha para `pos` e está a menos de `√range_sq`? Devolve a distância ao quadrado.
    #[inline]
    fn candidata(&self, i: usize, pos: V2, range_sq: f64) -> Option<f64> {
        let a = self.point[i];
        let b = self.point[self.next[i] as usize];
        if abs_sq(sub(b, a)) <= 0.0 || left_of(a, b, pos) >= 0.0 {
            return None;
        }
        let d = dist_sq_to_segment(a, b, pos);
        (d < range_sq).then_some(d)
    }

    /// A varredura INTEIRA — o oráculo do gate da grelha.
    #[cfg(test)]
    pub(crate) fn near_todas(&self, pos: V2, range: f64, out: &mut Vec<u32>) {
        out.clear();
        let range_sq = range * range;
        let mut found: Vec<(f64, u32)> = (0..self.point.len())
            .filter_map(|i| self.candidata(i, pos, range_sq).map(|d| (d, i as u32)))
            .collect();
        found.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
        out.extend(found.into_iter().map(|(_, i)| i));
    }
}
