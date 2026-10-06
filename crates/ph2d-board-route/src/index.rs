//! **Índice espacial das formas** — uma grelha uniforme, mantida por DIFERENÇA (só as formas que
//! mudaram entram e saem): a pergunta «que formas tocam esta região?» custa as células da região,
//! não o documento inteiro (100 mil formas, plano §2).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::ElementId;
use ph2d_vec_connect::Aabb;

/// Lado de uma célula (mundo): uma caixa de nascença (`CLICK_SIZE` 160×100) mais o vão até à
/// vizinha cabe em uma ou duas. É sintonia, não limite: só muda quantas células uma busca visita.
const CELL: f64 = 256.0;
/// Uma caixa que cobre mais células do que isto vai para a lista das GRANDES (candidata em toda
/// busca) — senão uma moldura gigante encheria milhares de células.
const MAX_CELLS: i64 = 64;

#[derive(Debug, Default)]
pub(crate) struct ShapeIndex {
    boxes: BTreeMap<ElementId, Aabb>,
    cells: BTreeMap<(i64, i64), Vec<ElementId>>,
    big: BTreeSet<ElementId>,
}

fn span(b: Aabb) -> (i64, i64, i64, i64) {
    let c = |v: f64| (v / CELL).floor() as i64;
    (c(b.min[0]), c(b.min[1]), c(b.max[0]), c(b.max[1]))
}

fn cells_of(b: Aabb) -> Option<impl Iterator<Item = (i64, i64)>> {
    let (x0, y0, x1, y1) = span(b);
    ((x1 - x0 + 1).saturating_mul(y1 - y0 + 1) <= MAX_CELLS)
        .then(|| (x0..=x1).flat_map(move |cx| (y0..=y1).map(move |cy| (cx, cy))))
}

impl ShapeIndex {
    /// Põe (ou repõe) `id` com a caixa `b`; devolve a caixa antiga.
    pub(crate) fn insert(&mut self, id: ElementId, b: Aabb) -> Option<Aabb> {
        let old = self.remove(id);
        self.boxes.insert(id, b);
        match cells_of(b) {
            Some(cells) => {
                for c in cells {
                    self.cells.entry(c).or_default().push(id);
                }
            }
            None => {
                self.big.insert(id);
            }
        }
        old
    }

    /// Tira `id`; devolve a caixa que tinha.
    pub(crate) fn remove(&mut self, id: ElementId) -> Option<Aabb> {
        let b = self.boxes.remove(&id)?;
        match cells_of(b) {
            Some(cells) => {
                for c in cells {
                    if let Some(v) = self.cells.get_mut(&c) {
                        v.retain(|x| *x != id);
                        if v.is_empty() {
                            self.cells.remove(&c);
                        }
                    }
                }
            }
            None => {
                self.big.remove(&id);
            }
        }
        Some(b)
    }

    /// Os candidatos a tocar `roi` (a mais e repetidos: quem pergunta testa a sobreposição).
    pub(crate) fn near(&self, roi: Aabb, out: &mut Vec<(ElementId, Aabb)>) {
        let (x0, y0, x1, y1) = span(roi);
        let area = (x1 - x0 + 1).saturating_mul(y1 - y0 + 1);
        let mut push = |id: &ElementId| out.push((*id, self.boxes[id]));
        if usize::try_from(area).map_or(true, |a| a >= self.cells.len()) {
            self.cells.values().flatten().for_each(&mut push);
        } else {
            for cx in x0..=x1 {
                for cy in y0..=y1 {
                    if let Some(v) = self.cells.get(&(cx, cy)) {
                        v.iter().for_each(&mut push);
                    }
                }
            }
        }
        self.big.iter().for_each(push);
    }
}
