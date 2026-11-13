//! B20.11b: as formigas de uma seleção acima do orçamento vetorial
//! ([`super::selection_contorno::ORCAMENTO_DE_LADOS`]) saem por AMOSTRAGEM em espaço de ECRÃ: cada píxel
//! do ecrã na caixa da sprite amostra a máscara (vizinho-mais-próximo pelo afim inverso) nele e nos
//! vizinhos direito/baixo; dentro/fora diferente = píxel de formiga. Nenhuma borda some com o zoom e a
//! largura é 1 px de ecrã. O custo é limitado pelo ECRÃ, nunca pela imagem. Guardado por (fase, afim,
//! ecrã, identidade das máscaras): só se refaz quando o passo ou a vista mudam.

use super::PainterTool;
use super::selection_formigas_ecra::{
    COR_DO_OFFSET, COR_FORMIGA_CLARA, COR_FORMIGA_ESCURA, TRACO_DAS_FORMIGAS, desenha_formigas,
};
use ph2d_vector::{Affine, ImageQuality, Point, StableImage, VectorScene};
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};

fn rgba(c: [f32; 4]) -> [u8; 4] {
    c.map(|v| (v * 255.0).round() as u8)
}

/// A caixa `[x0, y0, x1, y1)` no ecrã que cobre a imagem `w×h` por `imagem_para_ecra`, com 1 px de
/// folga (a borda da imagem também é fronteira), cortada ao ecrã. `None` se vazia.
#[must_use]
pub fn caixa_no_ecra(
    imagem_para_ecra: Affine,
    (w, h): (u32, u32),
    ecra: (u32, u32),
) -> Option<[u32; 4]> {
    let (w, h) = (f64::from(w), f64::from(h));
    let cantos =
        [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)].map(|(x, y)| imagem_para_ecra * Point::new(x, y));
    let (mut x0, mut y0, mut x1, mut y1) = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );
    for p in cantos {
        (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
    }
    let corta = |v: f64, m: u32| v.clamp(0.0, f64::from(m)) as u32;
    let c = [
        corta(x0.floor() - 1.0, ecra.0),
        corta(y0.floor() - 1.0, ecra.1),
        corta(x1.ceil() + 1.0, ecra.0),
        corta(y1.ceil() + 1.0, ecra.1),
    ];
    (c[0] < c[2] && c[1] < c[3]).then_some(c)
}

/// As formigas da `mascara` (claro + escuro tracejado) e o ciano dos `niveis`, RGBA da `caixa` do
/// ecrã. Paralelo em faixas de linhas, o mesmo byte em qualquer número de fios.
#[must_use]
pub fn formigas_raster(
    mascara: &[u8],
    niveis: &[&[u8]],
    (w, h): (usize, usize),
    imagem_para_ecra: Affine,
    caixa: [u32; 4],
    passo: u32,
) -> Vec<u8> {
    let [x0, y0, x1, y1] = caixa;
    let (rw, rh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let mut out = vec![0u8; rw * rh * 4];
    if rw == 0 || rh == 0 || mascara.len() != w * h {
        return out;
    }
    let [a, b, c, d, e, f] = imagem_para_ecra.inverse().as_coeffs();
    let periodo = (2.0 * TRACO_DAS_FORMIGAS) as u32;
    let fase = passo % periodo;
    let meio = periodo / 2;
    let (clara, escura, ciano) = (
        rgba(COR_FORMIGA_CLARA),
        rgba(COR_FORMIGA_ESCURA),
        rgba(COR_DO_OFFSET),
    );
    // Uma linha de amostras (rw + 1: o vizinho direito) da máscara `m` na linha `sy` do ecrã.
    let amostra = |m: &[u8], sy: u32, linha: &mut [bool]| {
        let cy = f64::from(sy) + 0.5;
        for (k, s) in linha.iter_mut().enumerate() {
            let cx = f64::from(x0) + k as f64 + 0.5;
            let (ix, iy) = ((a * cx + c * cy + e).floor(), (b * cx + d * cy + f).floor());
            *s = ix >= 0.0
                && iy >= 0.0
                && (ix as usize) < w
                && (iy as usize) < h
                && m[iy as usize * w + ix as usize] >= 128;
        }
    };
    let todas: Vec<&[u8]> = std::iter::once(mascara)
        .chain(niveis.iter().copied())
        .collect();
    let faixa = |banda: &mut [u8], r0: usize| {
        let n = todas.len();
        let mut cur = vec![vec![false; rw + 1]; n];
        let mut nxt = vec![vec![false; rw + 1]; n];
        for (k, m) in todas.iter().enumerate() {
            amostra(m, y0 + r0 as u32, &mut cur[k]);
        }
        for (r, linha) in banda.chunks_exact_mut(rw * 4).enumerate() {
            let sy = y0 + (r0 + r) as u32;
            for (k, m) in todas.iter().enumerate() {
                amostra(m, sy + 1, &mut nxt[k]);
            }
            let borda = |k: usize, i: usize| cur[k][i] != cur[k][i + 1] || cur[k][i] != nxt[k][i];
            for i in 0..rw {
                let sx = x0 + i as u32;
                let escuro = ((sx + sy + periodo - fase) / meio).is_multiple_of(2);
                let px = if borda(0, i) {
                    Some(if escuro { escura } else { clara })
                } else if escuro && (1..n).any(|k| borda(k, i)) {
                    Some(ciano)
                } else {
                    None
                };
                if let Some(p) = px {
                    linha[i * 4..i * 4 + 4].copy_from_slice(&p);
                }
            }
            std::mem::swap(&mut cur, &mut nxt);
        }
    };
    let threads = crate::compositor::parallel_threads(rw * rh, rh);
    crate::compositor::em_faixas_de_linhas(&mut out, rw * 4, threads, faixa);
    out
}

struct Chave {
    fase: u32,
    afim: [u64; 6],
    caixa: [u32; 4],
    tamanho: (u32, u32),
    mascara: Weak<Vec<u8>>,
    niveis: Vec<Weak<Vec<u8>>>,
}

impl PartialEq for Chave {
    fn eq(&self, o: &Self) -> bool {
        (self.fase, self.afim, self.caixa, self.tamanho) == (o.fase, o.afim, o.caixa, o.tamanho)
            && self.mascara.ptr_eq(&o.mascara)
            && self.niveis.len() == o.niveis.len()
            && self.niveis.iter().zip(&o.niveis).all(|(a, b)| a.ptr_eq(b))
    }
}

thread_local! {
    static GUARDADO: RefCell<Option<(Chave, StableImage)>> = const { RefCell::new(None) };
    static CONSTRUCOES: Cell<u64> = const { Cell::new(0) };
}

/// Quantas vezes a imagem das formigas em ecrã foi construída neste fio (gates).
#[doc(hidden)]
#[must_use]
pub fn construcoes_das_formigas_raster() -> u64 {
    CONSTRUCOES.with(Cell::get)
}

impl PainterTool {
    /// As formigas da Seleção viva em `vs`, pelo afim imagem→ecrã, num ecrã `ecra` (px): o traço
    /// vetorial abaixo do orçamento, a amostragem em ecrã acima dele. No-op sem seleção viva.
    pub fn desenha_formigas_da_selecao(
        &self,
        vs: &mut VectorScene,
        imagem_para_ecra: Affine,
        ecra: (u32, u32),
        passo: u32,
    ) {
        let Some(c) = self.selection_contornos() else {
            return;
        };
        if !c.no_raster {
            desenha_formigas(vs, &c, imagem_para_ecra, passo);
            return;
        }
        let Some(caixa) = caixa_no_ecra(imagem_para_ecra, self.source_size, ecra) else {
            return;
        };
        let niveis = self.niveis_do_offset_vivos();
        let chave = Chave {
            fase: passo % (2.0 * TRACO_DAS_FORMIGAS) as u32,
            afim: imagem_para_ecra.as_coeffs().map(f64::to_bits),
            caixa,
            tamanho: self.source_size,
            mascara: Arc::downgrade(&self.paint.selection_mask),
            niveis: niveis.iter().map(Arc::downgrade).collect(),
        };
        let img = GUARDADO.with_borrow_mut(|g| {
            if g.as_ref().is_none_or(|(k, _)| *k != chave) {
                *g = None;
                let (w, h) = self.source_size;
                let n: Vec<&[u8]> = niveis.iter().map(|m| m.as_slice()).collect();
                let px = formigas_raster(
                    &self.paint.selection_mask,
                    &n,
                    (w as usize, h as usize),
                    imagem_para_ecra,
                    caixa,
                    passo,
                );
                CONSTRUCOES.set(CONSTRUCOES.get() + 1);
                let img =
                    StableImage::from_rgba(Arc::new(px), caixa[2] - caixa[0], caixa[3] - caixa[1])?;
                *g = Some((chave, img));
            }
            g.as_ref().map(|(_, i)| i.clone())
        });
        if let Some(img) = img {
            let t = Affine::translate((f64::from(caixa[0]), f64::from(caixa[1])));
            vs.draw_stable_image_transformed(&img, t, ImageQuality::Low);
        }
    }
}

#[cfg(test)]
#[path = "selection_formigas_raster_tests.rs"]
mod tests;
