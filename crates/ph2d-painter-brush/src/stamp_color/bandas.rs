//! O esqueleto de bandas ÚNICO das três rotas em lote ([`super::accumulate_batch`],
//! [`super::accumulate_rgba_batch`]): partir os mapas nas MESMAS faixas de linhas disjuntas, correr cada faixa
//! numa thread sobre os dabs na ordem serial e juntar o retângulo tocado (B52.R26). Cada píxel pertence a UMA
//! faixa ⇒ o mesmo bit para qualquer número de faixas (HR-5).

use super::ColorStampMask;
use super::accumulate_batch::FusedDab;
use crate::dab::DirtyRect;

/// The bounding union of two touched rects.
pub(super) fn union_rect(a: DirtyRect, b: DirtyRect) -> DirtyRect {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.w).max(b.x + b.w);
    let y1 = (a.y + a.h).max(b.y + b.h);
    DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    }
}

/// A caixa recortada de um dab do lote: o índice do dab e `[x0, x1) × [y0, y1)`.
pub(super) trait Caixa: Sync {
    fn di(&self) -> usize;
    fn caixa(&self) -> [i64; 4];
}

/// As linhas `[gy0, gy1)` que o lote cobre e a soma das áreas das caixas (a pergunta do `band_count`).
pub(super) struct Lote {
    pub gy0: usize,
    pub gy1: usize,
    pub area: usize,
}

impl Lote {
    pub(super) fn de<B: Caixa>(boxes: &[B]) -> Option<Lote> {
        let gy0 = boxes.iter().map(|b| b.caixa()[1]).min()? as usize;
        let gy1 = boxes.iter().map(|b| b.caixa()[3]).max()? as usize;
        let area = boxes
            .iter()
            .map(|b| {
                let [x0, y0, x1, y1] = b.caixa();
                ((x1 - x0) * (y1 - y0)) as usize
            })
            .sum();
        Some(Lote { gy0, gy1, area })
    }

    /// Quantas faixas ([`crate::dab::band_count`], a porta única).
    pub(super) fn faixas(&self) -> usize {
        crate::dab::band_count(
            self.area,
            self.gy1 - self.gy0,
            crate::dab::PARALLEL_MIN_AREA,
        )
    }
}

/// A união das caixas dos dabs que `tocado(di)`.
pub(super) fn uniao<B: Caixa>(boxes: &[B], tocado: impl Fn(usize) -> bool) -> Option<DirtyRect> {
    let mut union: Option<DirtyRect> = None;
    for b in boxes {
        if tocado(b.di()) {
            let [x0, y0, x1, y1] = b.caixa();
            let r = DirtyRect {
                x: x0 as u32,
                y: y0 as u32,
                w: (x1 - x0) as u32,
                h: (y1 - y0) as u32,
            };
            union = Some(union.map_or(r, |u| union_rect(u, r)));
        }
    }
    union
}

/// Parte cada mapa (`bpp` bytes por píxel) nas MESMAS `threads` faixas de linhas de `lote`, corre
/// `faixa(mapas_da_faixa, band_y0, band_y1, tocado)` numa thread por faixa e devolve a união das caixas
/// tocadas em alguma faixa. O pânico de uma faixa sobe com a carga ORIGINAL (`resume_unwind`).
#[allow(clippy::too_many_arguments)]
pub(super) fn bandas_com_toque<B: Caixa>(
    maps: &mut [Vec<u8>],
    width: u32,
    bpp: usize,
    boxes: &[B],
    ndabs: usize,
    lote: &Lote,
    threads: usize,
    faixa: impl Fn(&mut [&mut [u8]], i64, i64, &mut [bool]) + Sync,
) -> Option<DirtyRect> {
    let (gy0, gy1) = (lote.gy0, lote.gy1);
    let rows = gy1 - gy0;
    let stride = width as usize * bpp;
    let rpb = rows.div_ceil(threads);
    let nbands = rows.div_ceil(rpb);
    let mut bands: Vec<Vec<&mut [u8]>> = (0..nbands)
        .map(|_| Vec::with_capacity(maps.len()))
        .collect();
    for map in maps.iter_mut() {
        let mut rest = &mut map[gy0 * stride..gy1 * stride];
        for band in bands.iter_mut() {
            let take = (rpb * stride).min(rest.len());
            let (head, tail) = rest.split_at_mut(take);
            band.push(head);
            rest = tail;
        }
    }
    let faixa = &faixa;
    let touched_by_band: Vec<Vec<bool>> = std::thread::scope(|scope| {
        let handles: Vec<_> = bands
            .into_iter()
            .enumerate()
            .map(|(bi, mut maps)| {
                scope.spawn(move || {
                    let band_y0 = (gy0 + bi * rpb) as i64;
                    let band_y1 = band_y0 + (maps[0].len() / stride) as i64;
                    let mut touched = vec![false; ndabs];
                    faixa(&mut maps, band_y0, band_y1, &mut touched);
                    touched
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_else(|p| std::panic::resume_unwind(p)))
            .collect()
    });
    uniao(boxes, |di| touched_by_band.iter().any(|t| t[di]))
}

/// A caixa de um dab das rotas de carimbo assado (fused e RGBA): o carimbo cobre `[-reach, reach]²` raios
/// ([`crate::tip::stamp_domain`]); `grupos` são as camadas com o nível da pirâmide DESTE dab, juntas por lado
/// (B29.A2: cada carimbo lê o seu nível, nunca o passo do `stamps[0]`).
pub(super) struct CaixaCarimbo<'a> {
    pub di: usize,
    pub x0: i64,
    pub y0: i64,
    pub x1: i64,
    pub y1: i64,
    pub cx: f32,
    pub cy: f32,
    pub inv_radius: f32,
    pub coverage: f32,
    pub grupos: Grupos<'a>,
}

impl Caixa for CaixaCarimbo<'_> {
    fn di(&self) -> usize {
        self.di
    }
    fn caixa(&self) -> [i64; 4] {
        [self.x0, self.y0, self.x1, self.y1]
    }
}

/// As camadas `(i, nível)` de um dab, juntas pelo lado do nível (ordem da primeira aparição; um lote de um lado
/// só é UM grupo na ordem das camadas).
pub(super) type Grupos<'a> = Vec<Vec<(usize, &'a ColorStampMask)>>;

pub(super) fn grupos_por_lado(stamps: &[ColorStampMask], lado: f32) -> Grupos<'_> {
    let mut g: Grupos<'_> = Vec::with_capacity(1);
    for (i, s) in stamps.iter().enumerate() {
        let n = s.nivel_para(lado);
        match g.iter_mut().find(|v| v[0].1.size == n.size) {
            Some(v) => v.push((i, n)),
            None => g.push(vec![(i, n)]),
        }
    }
    g
}

/// As caixas recortadas dos dabs que pintam (cobertura e raio > 0), em ordem.
pub(super) fn caixas_do_carimbo<'a>(
    dabs: &[FusedDab],
    stamps: &'a [ColorStampMask],
    width: u32,
    height: u32,
) -> Vec<CaixaCarimbo<'a>> {
    let reach = stamps[0].reach;
    let mut boxes = Vec::with_capacity(dabs.len());
    for (di, d) in dabs.iter().enumerate() {
        let coverage = d.coverage.clamp(0.0, 1.0);
        if coverage <= 0.0 || d.radius <= 0.0 {
            continue;
        }
        let (cx, cy) = (d.center[0], d.center[1]);
        let r = d.radius * reach;
        let x0 = (cx - r).floor().max(0.0) as i64;
        let y0 = (cy - r).floor().max(0.0) as i64;
        let x1 = ((cx + r).ceil() as i64 + 1).min(width as i64);
        let y1 = ((cy + r).ceil() as i64 + 1).min(height as i64);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        boxes.push(CaixaCarimbo {
            di,
            x0,
            y0,
            x1,
            y1,
            cx,
            cy,
            inv_radius: 1.0 / r,
            coverage,
            grupos: grupos_por_lado(stamps, 2.0 * r),
        });
    }
    boxes
}

/// Contador de amostras do grão nas rotas Per-Layer (B29.A3), só para a tela de largura [`sonda::LARGURA`]
/// (os testes correm em paralelo no mesmo processo).
#[cfg(test)]
pub(super) mod sonda {
    use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};
    pub(crate) static LARGURA: AtomicU32 = AtomicU32::new(0);
    pub(crate) static AMOSTRAS: AtomicU64 = AtomicU64::new(0);
    pub(crate) fn conta(largura: u32) {
        if largura == LARGURA.load(Relaxed) {
            AMOSTRAS.fetch_add(1, Relaxed);
        }
    }
}
