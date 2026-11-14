//! **O topo SEGUIDO do journal** (ADR-0205, a absorção do escorrido do Wet) — filho de [`super`] (`#[path]`: lê
//! os campos privados de [`PlaneWindow`]). O topo guarda `antes0 → R` (R = o cursor), o journal do vidro guarda
//! `R → vivo` (o escorrido depois do commit); a entrada re-partida é `antes0 → vivo`. Em vez de segurar o plano
//! do reset e varrer o plano inteiro três vezes (clone + detector + re-split: medido +34 ms no pen-down e +41 ms
//! no Ctrl+Z a 4096², `watercolor_vidro_custo_sonda`), compõe-se nas caixas da grade de [`TILE`] que um dos
//! dois toca: o `before` de cada caixa é o do topo onde ele fala, senão o do journal, senão o vivo.

use super::{PlaneWindow, RtLayerId, StoredEntry, StoredMap, StoredPlane};
use crate::undo::journal::{TILE, TileJournal};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

impl<T: Copy + Default + PartialEq + Send + Sync> StoredPlane<T> {
    /// `self` = `antes0 → R`, `j` = `R → live`: o delta `antes0 → live`, só nas caixas dos dois, e se o journal
    /// mudou algum byte (`R ≠ live`). `None` = uma forma que a composição não descreve (quem chama corre o
    /// caminho que segura o plano).
    pub(crate) fn composto_com_o_journal(
        &self,
        live: &[T],
        j: &TileJournal<T>,
    ) -> Option<(Self, bool)> {
        let len = live.len();
        if j.whole().is_some() {
            return None;
        }
        let (topo, antes0): (Vec<PlaneWindow>, &[T]) = match self {
            Self::Unchanged => (Vec::new(), &[]),
            Self::Patch { win, before, .. } => (vec![*win], before),
            Self::Tiles { rects, before, .. } => (rects.to_vec(), before),
            Self::Whole { .. } | Self::Nova { .. } => return None,
        };
        let jr = j.captured();
        let Some(stride) = topo.iter().chain(&jr).map(|w| w.stride).next() else {
            return Some((Self::Unchanged, false));
        };
        if topo
            .iter()
            .chain(&jr)
            .any(|w| w.stride != stride || w.plane_len != len)
            || !super::fits(len, stride)
        {
            return None;
        }
        let rows = len / stride;
        let tiles_x = stride.div_ceil(TILE);
        let mut grade = BTreeSet::new();
        for w in topo.iter().chain(&jr).filter(|w| w.elems() > 0) {
            for ty in w.row / TILE..=(w.row + w.rows - 1) / TILE {
                for tx in w.col / TILE..=(w.col + w.cols - 1) / TILE {
                    grade.insert(ty * tiles_x + tx);
                }
            }
        }
        let mut off = Vec::with_capacity(topo.len());
        let mut at = 0;
        for w in &topo {
            off.push(at);
            at += w.elems();
        }
        if at != antes0.len() {
            return None;
        }
        let caixas: Vec<PlaneWindow> = grade
            .into_iter()
            .filter_map(|t| {
                let (row, col) = ((t / tiles_x) * TILE, (t % tiles_x) * TILE);
                PlaneWindow::tiles(
                    row,
                    (row + TILE).min(rows) - row,
                    col,
                    (col + TILE).min(stride) - col,
                    stride,
                    len,
                )
            })
            .collect();
        // `(o before da caixa, o journal mudou-a, ela difere do vivo)` — uma leitura do journal e do vivo por caixa.
        let lado = |r: &PlaneWindow| -> (Box<[T]>, bool, bool) {
            let mut v = r.extract_journal(j, live).into_vec();
            let pingou = !r.holds(&v, live);
            let mut tocou = false;
            for (w, o) in topo.iter().zip(&off) {
                let Some(i) = w.intersect(*r) else { continue };
                tocou = true;
                for k in 0..i.rows {
                    let s = o + (i.row - w.row + k) * w.cols + (i.col - w.col);
                    let d = (i.row - r.row + k) * r.cols + (i.col - r.col);
                    v[d..d + i.cols].copy_from_slice(&antes0[s..s + i.cols]);
                }
            }
            let difere = if tocou { !r.holds(&v, live) } else { pingou };
            (v.into_boxed_slice(), pingou, difere)
        };
        use rayon::prelude::*;
        let todas: Vec<(PlaneWindow, Box<[T]>, bool, bool)> = caixas
            .into_par_iter()
            .map(|r| {
                let (b, p, d) = lado(&r);
                (r, b, p, d)
            })
            .collect();
        let pingou = todas.iter().any(|x| x.2);
        #[cfg(test)]
        SONDA_CAIXAS.with(|c| {
            c.set((
                todas.len(),
                topo.len(),
                jr.len(),
                todas.iter().filter(|x| x.3).count(),
            ))
        });
        let lados: Vec<(PlaneWindow, Box<[T]>)> = todas
            .into_iter()
            .filter_map(|(r, b, _, d)| d.then_some((r, b)))
            .collect();
        if lados.is_empty() {
            return Some((Self::Unchanged, pingou));
        }
        let n: usize = lados.iter().map(|(r, _)| r.elems()).sum();
        if 2 * n >= len {
            return None;
        }
        let (r0, r1, c0, c1) = lados.iter().fold(
            (usize::MAX, 0, usize::MAX, 0),
            |(r0, r1, c0, c1), (r, _)| {
                (
                    r0.min(r.row),
                    r1.max(r.row + r.rows),
                    c0.min(r.col),
                    c1.max(r.col + r.cols),
                )
            },
        );
        let win = PlaneWindow::tiles(r0, r1 - r0, c0, c1 - c0, stride, len)?;
        // Os dois lados concatenados na ordem das caixas, cada caixa no seu pedaço, em paralelo (~57 MB a
        // 4096² com o escorrido num quarto da tela: em série era a parte mais lenta da composição).
        let (mut b, mut a) = (vec![T::default(); n], vec![T::default(); n]);
        let mut pedacos = Vec::with_capacity(lados.len());
        let (mut rb, mut ra) = (&mut b[..], &mut a[..]);
        for (r, x) in &lados {
            let (pb, resto_b) = std::mem::take(&mut rb).split_at_mut(r.elems());
            let (pa, resto_a) = std::mem::take(&mut ra).split_at_mut(r.elems());
            (rb, ra) = (resto_b, resto_a);
            pedacos.push((r, x, pb, pa));
        }
        pedacos.into_par_iter().for_each(|(r, x, pb, pa)| {
            pb.copy_from_slice(x);
            for k in 0..r.rows {
                let s = (r.row + k) * r.stride + r.col;
                pa[k * r.cols..(k + 1) * r.cols].copy_from_slice(&live[s..s + r.cols]);
            }
        });
        let rects: Vec<PlaneWindow> = lados.into_iter().map(|(r, _)| r).collect();
        Some((
            Self::Tiles {
                win,
                rects: rects.into_boxed_slice(),
                before: b.into_boxed_slice(),
                after: a.into_boxed_slice(),
            },
            pingou,
        ))
    }
}

impl<T: Copy + Default + PartialEq + Send + Sync> StoredMap<T> {
    /// O mapa `antes0 → live` com o journal a falar pela camada `layer` (as outras: as do topo, pela lei do
    /// journal de uma camada só), e se o journal mudou algum byte. `None` = uma chave fora de `Both`, ou as
    /// chaves do topo ≠ as do vivo.
    pub(crate) fn composto_com_o_journal(
        &self,
        live: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
        layer: RtLayerId,
        j: &TileJournal<T>,
    ) -> Option<(Self, bool)> {
        if !self.aparece.is_empty() || !live.keys().eq(self.entries.keys()) {
            return None;
        }
        let mut entries = BTreeMap::new();
        let mut pingou = false;
        for (k, e) in &self.entries {
            let StoredEntry::Both(p) = e else {
                return None;
            };
            let p = if *k == layer {
                let (p, mudou) = p.composto_com_o_journal(live.get(k)?, j)?;
                pingou = mudou;
                p
            } else {
                p.clone()
            };
            entries.insert(*k, StoredEntry::Both(p));
        }
        entries.contains_key(&layer).then_some((
            Self {
                entries,
                aparece: BTreeMap::new(),
            },
            pingou,
        ))
    }
}

#[cfg(test)]
thread_local! {
    /// SONDA: (caixas lidas, caixas do topo, tiles do journal, caixas que diferem) da última composição.
    pub(crate) static SONDA_CAIXAS: std::cell::Cell<(usize, usize, usize, usize)> = const { std::cell::Cell::new((0, 0, 0, 0)) };
}
