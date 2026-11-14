//! **O passo guardado pelos TILES que ele tocou** (B17.5) — filho de [`super`] (`#[path]`, lê os campos
//! privados da janela).
//!
//! A janela declarada é a UNIÃO-retângulo do que se escreveu: um traço diagonal cobre a tela inteira e o
//! [`super::StoredPlane::from_window`] guardava o `Whole` (66,4 MB por passo no Digital a 3840×2160,
//! 265,4 MB no Impasto). Os tiles de [`TILE`]² que o traço toca são poucos; o variant `Tiles` guarda os
//! dois lados só deles. Superconjunto do escrito pela mesma lei da janela (`crate::undo::window`): cada
//! tile é a caixa de um `mark`, recortada à janela.

use std::collections::BTreeSet;
use std::sync::Arc;

use super::{PlaneWindow, StoredEntry, StoredMap, StoredPlane};

#[cfg(test)]
thread_local! {
    /// A ablação do B17.5: `true` = [`StoredPlane::side_tomando`] copia sempre (o oráculo, o mundo de antes).
    pub(crate) static COPIA_SEMPRE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
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

impl PlaneWindow {
    /// Os dois planos coincidem nesta caixa?
    pub(super) fn same_in<T: PartialEq>(&self, a: &[T], b: &[T]) -> bool {
        (0..self.rows).all(|r| {
            let s = (self.row + r) * self.stride + self.col;
            a[s..s + self.cols] == b[s..s + self.cols]
        })
    }

    /// O plano tem, nesta caixa, exactamente os elementos de `patch` (na ordem do `extract`)?
    pub(super) fn holds<T: PartialEq>(&self, patch: &[T], plane: &[T]) -> bool {
        (0..self.rows).all(|r| {
            let s = (self.row + r) * self.stride + self.col;
            patch[r * self.cols..(r + 1) * self.cols] == plane[s..s + self.cols]
        })
    }
}

impl<T: Copy + Default + PartialEq + Send + Sync> StoredPlane<T> {
    /// **Só as caixas que MUDARAM**, com os dois lados, na ordem de `rects` (o `extract` e o `blit`
    /// percorrem-nas na MESMA). A marca diz onde se PODE ter escrito; a caixa igual nos dois lados não se
    /// guarda — o commit do Impasto declara a caixa do traço inteiro (a diagonal = a tela) e só os tiles
    /// da pegada mudam (B17.5).
    ///
    /// `None` = as caixas que mudaram não pesam menos que a janela `win` nem que metade do plano
    /// (`plane_len`): o chamador guarda a janela, como antes. A comparação e a extração correm por caixa
    /// no rayon (leitura pura, caixas disjuntas, `collect` na ordem: o mesmo bit em qualquer nº de fios).
    pub(super) fn changed_tiles(
        win: PlaneWindow,
        rects: Vec<PlaneWindow>,
        plane_len: usize,
        differs: impl Fn(&PlaneWindow) -> bool + Sync,
        before: impl Fn(&PlaneWindow) -> Box<[T]> + Sync,
        after: &[T],
    ) -> Option<Self> {
        use rayon::prelude::*;
        let mudou: Vec<bool> = rects.par_iter().map(&differs).collect();
        let kept: Vec<PlaneWindow> = rects
            .into_iter()
            .zip(mudou)
            .filter_map(|(r, m)| m.then_some(r))
            .collect();
        if kept.is_empty() {
            return Some(Self::Unchanged);
        }
        let n = elems(&kept);
        if n >= win.elems() || 2 * n >= plane_len {
            return None;
        }
        let lados: Vec<_> = kept
            .par_iter()
            .map(|r| (before(r), r.extract(after)))
            .collect();
        let mut b = Vec::with_capacity(n);
        let mut a = Vec::with_capacity(n);
        for (x, y) in &lados {
            b.extend_from_slice(x);
            a.extend_from_slice(y);
        }
        Some(Self::Tiles {
            win,
            rects: kept.into_boxed_slice(),
            before: b.into_boxed_slice(),
            after: a.into_boxed_slice(),
        })
    }

    /// `side` pode partir de um plano de `len` elementos? (A única recusa de `Patch`/`Tiles`.)
    pub(crate) fn cabe(&self, len: usize) -> bool {
        match self {
            Self::Patch { win, .. } | Self::Tiles { win, .. } => len == win.plane_len,
            _ => true,
        }
    }

    /// [`Self::side`] TOMANDO a base (B17.5): com um dono só — nem `Weak`, cuja testemunha lê «mesmo `Arc` ⇒ mesmos
    /// bytes» — os tiles vão para o plano NO LUGAR; com outro dono, a cópia de sempre. O resultado é o mesmo bit.
    pub(crate) fn side_tomando(
        &self,
        mut base: Arc<Vec<T>>,
        want_before: bool,
    ) -> Option<Arc<Vec<T>>> {
        if !self.cabe(base.len()) {
            return None;
        }
        #[cfg(test)]
        let unico = !COPIA_SEMPRE.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let unico = true;
        let Some(v) = Arc::get_mut(&mut base).filter(|_| unico) else {
            return self.side(&base, want_before);
        };
        match self {
            Self::Patch { win, before, after } => {
                win.blit(if want_before { before } else { after }, v);
            }
            Self::Tiles {
                rects,
                before,
                after,
                ..
            } => Self::blit_tiles(rects, if want_before { before } else { after }, v),
            _ => return self.side(&base, want_before),
        }
        Some(base)
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

type Mapa<T> = std::collections::BTreeMap<crate::layers::LayerId, Arc<Vec<T>>>;

impl<T: Copy + Default + PartialEq + Send + Sync> StoredMap<T> {
    /// `side` parte de `base`? (Cada `Both` acha a chave, do tamanho do delta: as únicas recusas.)
    pub(crate) fn cabe(&self, base: &Mapa<T>) -> bool {
        self.entries.iter().all(|(k, e)| match e {
            StoredEntry::Both(p) => base.get(k).is_some_and(|c| p.cabe(c.len())),
            _ => true,
        })
    }

    /// [`Self::side`] TOMANDO a base (B17.5, o relevo do Impasto): cada plano com um dono só recebe os tiles no
    /// lugar. Pré-condição: [`Self::cabe`] — então não recusa, e nada é escrito antes de uma recusa possível.
    pub(crate) fn side_tomando(&self, mut base: Mapa<T>, want_before: bool) -> Mapa<T> {
        let mut out = Mapa::new();
        for (k, e) in &self.entries {
            let v = match e {
                StoredEntry::Both(p) => {
                    let c = base.remove(k).expect("cabe: a chave existe");
                    Some(p.side_tomando(c, want_before).expect("cabe: o tamanho"))
                }
                borda => Self::borda(borda, want_before),
            };
            if let Some(v) = v {
                out.insert(*k, v);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// **A marca larga guarda só os tiles que MUDARAM, e os dois lados voltam ao bit** (B17.5): o commit
    /// do Impasto marca a caixa do traço inteiro (a diagonal = a tela) e a pegada é um tile.
    #[test]
    fn a_marca_larga_guarda_so_os_tiles_que_mudaram_e_os_dois_lados_voltam_ao_bit() {
        let (w, h) = (512u32, 384u32);
        let t = TILE as u32;
        let before = Arc::new(vec![7u8; (w * h) as usize]);
        let mut a = (*before).clone();
        a[(t + 5) as usize * w as usize + 2 * t as usize + 9] = 200; // só o tile (1, 2)
        let after = Arc::new(a);
        let tudo = Region { x: 0, y: 0, w, h };
        let mut marks = TileMarks::default();
        marks.mark(tudo);
        let px = marks
            .for_hint(Some(tudo), 1)
            .expect("os marks viram a janela");
        let hint = PlaneWindow::from_region(tudo, w, w as usize);
        let (mut b, mut d) = (Arc::clone(&before), Arc::clone(&after));
        let out = StoredPlane::split_tiled(&mut b, &mut d, w as usize, hint, Some((&px, w)));
        match &out {
            StoredPlane::Tiles { rects, .. } => assert_eq!(rects.len(), 1, "só o tile que mudou"),
            o => panic!("esperava Tiles, veio {}", o.variant()),
        }
        assert_eq!(out.side(&after, true).as_deref(), Some(&*before));
        assert_eq!(out.side(&before, false).as_deref(), Some(&*after));
        let (mut b, mut d) = (Arc::clone(&before), Arc::new((*before).clone()));
        let igual = StoredPlane::split_tiled(&mut b, &mut d, w as usize, hint, Some((&px, w)));
        assert_eq!(igual.variant(), "-", "nada mudou: zero bytes");
    }

    /// **Com um dono só, o Ctrl+Z escreve os tiles NO LUGAR e dá o bit da cópia** (B17.5). Oráculo: `side` sobre o
    /// mesmo plano (a cópia de antes). Controlo: com outro dono forte ou um `Weak` vivo, copia e não toca a base.
    #[test]
    fn o_side_tomando_escreve_no_lugar_com_um_dono_e_da_o_bit_da_copia() {
        let (w, h) = (512u32, 384u32);
        let t = TILE as u32;
        let before = Arc::new(vec![7u8; (w * h) as usize]);
        let mut a = (*before).clone();
        a[(t + 5) as usize * w as usize + 2 * t as usize + 9] = 200;
        a[(2 * t + 1) as usize * w as usize + 3] = 9;
        let after = Arc::new(a);
        let tudo = Region { x: 0, y: 0, w, h };
        let mut marks = TileMarks::default();
        marks.mark(tudo);
        let px = marks.for_hint(Some(tudo), 1).expect("janela");
        let hint = PlaneWindow::from_region(tudo, w, w as usize);
        let (mut b, mut d) = (Arc::clone(&before), Arc::clone(&after));
        let out = StoredPlane::split_tiled(&mut b, &mut d, w as usize, hint, Some((&px, w)));
        assert_eq!(out.variant(), "tiles");
        for (base, want, lado) in [(&after, true, &before), (&before, false, &after)] {
            let oraculo = out.side(base, want).expect("oráculo");
            assert_eq!(*oraculo, **lado);
            // Um dono só: no lugar (o mesmo ponteiro), o mesmo bit.
            let unico = Arc::new((**base).clone());
            let ptr = Arc::as_ptr(&unico);
            let r = out.side_tomando(unico, want).expect("cabe");
            assert_eq!(*r, *oraculo, "no lugar ≠ cópia (want_before={want})");
            assert_eq!(
                Arc::as_ptr(&r),
                ptr,
                "um dono só tem de escrever no lugar, sem copiar"
            );
            // Outro dono forte: copia, a base fica intacta.
            let partilhado = Arc::new((**base).clone());
            let outro = Arc::clone(&partilhado);
            let r = out.side_tomando(partilhado, want).expect("cabe");
            assert_eq!(*r, *oraculo);
            assert!(
                !Arc::ptr_eq(&r, &outro) && *outro == **base,
                "o outro dono viu a escrita"
            );
            // Um `Weak` vivo (a testemunha da elisão): copia, a testemunha continua a ler os bytes de antes.
            let fraco_base = Arc::new((**base).clone());
            let fraco = Arc::downgrade(&fraco_base);
            let r = out.side_tomando(fraco_base, want).expect("cabe");
            assert_eq!(*r, *oraculo);
            assert!(
                fraco.upgrade().is_none_or(|p| !Arc::ptr_eq(&p, &r)),
                "testemunha enganada"
            );
        }
        assert!(
            out.side_tomando(Arc::new(vec![0u8; 3]), true).is_none(),
            "tamanho errado recusa"
        );
    }
}
