//! **O passo guardado pelos TILES que ele tocou** (B17.5) — filho de [`super`] (`#[path]`, lê os campos
//! privados da janela).
//!
//! A janela declarada é a UNIÃO-retângulo do que se escreveu: um traço diagonal cobre a tela inteira e o
//! [`super::StoredPlane::from_window`] guardava o `Whole` (66,4 MB por passo no Digital a 3840×2160,
//! 265,4 MB no Impasto). Os tiles de [`TILE`]² que o traço toca são poucos; o variant `Tiles` guarda os
//! dois lados só deles. Superconjunto do escrito pela mesma lei da janela (`crate::undo::window`): cada
//! tile é a caixa de um `mark`, recortada à janela.

use std::collections::BTreeSet;

use super::{PlaneWindow, StoredPlane};
use crate::compositor::Region;
use crate::undo::journal::TILE;

/// Os tiles de [`TILE`]² px marcados desde o último commit, e a união-retângulo de que eles falam.
///
/// ⚠️ Só valem para uma janela IGUAL a `rect`: um `mark` que passe ao lado (o `WriteWindow` copiado,
/// marcado e reposto) muda a união sem passar por aqui, e aí [`Self::for_hint`] cala-se — o commit
/// guarda a janela, como antes.
#[derive(Clone, Debug, Default)]
pub(crate) struct TileMarks {
    rect: Option<Region>,
    marks: u32,
    tiles: BTreeSet<(u32, u32)>,
}

impl TileMarks {
    pub(crate) fn mark(&mut self, r: Region) {
        self.marks = self.marks.wrapping_add(1);
        if r.w == 0 || r.h == 0 {
            return;
        }
        let t = TILE as u32;
        for ty in r.y / t..=(r.y + r.h - 1) / t {
            for tx in r.x / t..=(r.x + r.w - 1) / t {
                self.tiles.insert((ty, tx));
            }
        }
        self.rect = Some(self.rect.map_or(r, |a| {
            let (x0, y0) = (a.x.min(r.x), a.y.min(r.y));
            let x1 = (a.x + a.w).max(r.x + r.w);
            let y1 = (a.y + a.h).max(r.y + r.h);
            Region {
                x: x0,
                y: y0,
                w: x1 - x0,
                h: y1 - y0,
            }
        }));
    }

    pub(crate) fn clear(&mut self) {
        self.rect = None;
        self.marks = 0;
        self.tiles.clear();
    }

    /// Os tiles, em PIXELS, quando eles viram os mesmos `marks` que fizeram a janela `hint`.
    pub(crate) fn for_hint(&self, hint: Option<Region>, marks: u32) -> Option<Vec<Region>> {
        let h = hint?;
        if self.rect != Some(h) || self.marks != marks {
            return None;
        }
        let t = TILE as u32;
        Some(
            self.tiles
                .iter()
                .map(|&(ty, tx)| Region {
                    x: tx * t,
                    y: ty * t,
                    w: t,
                    h: t,
                })
                .collect(),
        )
    }
}

impl PlaneWindow {
    /// As caixas `px` (em pixels) em ELEMENTOS deste plano, recortadas a esta janela.
    pub(crate) fn rects_within(&self, px: &[Region], canvas_w: u32) -> Option<Vec<Self>> {
        let mut out = Vec::with_capacity(px.len());
        for &r in px {
            if let Some(w) = Self::from_region(r, canvas_w, self.stride)
                .and_then(|w| w.fit_to(self.plane_len))
                .and_then(|w| w.intersect(*self))
            {
                out.push(w);
            }
        }
        Some(out)
    }
}

/// Os elementos que as caixas carregam (por lado).
fn elems(rects: &[PlaneWindow]) -> usize {
    rects.iter().map(PlaneWindow::elems).sum()
}

/// As caixas compensam contra a janela única? (Elas guardam os mesmos dois lados, só de menos texels.)
pub(super) fn worth(rects: &[PlaneWindow], win: &PlaneWindow) -> bool {
    rects.len() > 1 && elems(rects) < win.elems()
}

impl<T: Copy + Default + PartialEq + Send + Sync> StoredPlane<T> {
    /// Os dois lados das `rects`, na ordem delas (o `extract` e o `blit` percorrem-nas na MESMA).
    pub(super) fn tiles_from(
        win: PlaneWindow,
        rects: Vec<PlaneWindow>,
        before: impl Fn(&PlaneWindow, &mut Vec<T>),
        after: &[T],
    ) -> Self {
        let n = elems(&rects);
        let mut b = Vec::with_capacity(n);
        let mut a = Vec::with_capacity(n);
        for r in &rects {
            before(r, &mut b);
            a.extend_from_slice(&r.extract(after));
        }
        Self::Tiles {
            win,
            rects: rects.into_boxed_slice(),
            before: b.into_boxed_slice(),
            after: a.into_boxed_slice(),
        }
    }

    /// Escreve um dos lados das caixas sobre um plano completo.
    pub(super) fn blit_tiles(rects: &[PlaneWindow], side: &[T], dst: &mut [T]) {
        let mut at = 0;
        for r in rects {
            let n = r.elems();
            r.blit(&side[at..at + n], dst);
            at += n;
        }
    }
}
