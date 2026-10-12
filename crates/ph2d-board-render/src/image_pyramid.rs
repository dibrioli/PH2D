//! A PIRÂMIDE de uma imagem do quadro (W14; separada do [`super`] na W16 pelo tecto de LOC): os
//! tamanhos dos níveis, a metade pesada pelo alfa e a pirâmide inteira de uns píxeis.

use std::sync::Arc;

use ph2d_vector::StableImage;

use super::{ATLAS_SIDE, Levels, Pixels, THUMB_SIDE};

/// Os tamanhos da pirâmide de uma imagem `natural`: o 1.º cabe no atlas, cada um metade do anterior
/// (arredondada para cima), até um que caiba em [`THUMB_SIDE`]. Um SVG começa no lado do atlas (a
/// proporção dele), por pequeno que seja o natural: é vector.
pub(super) fn level_sizes(natural: [u32; 2], vector: bool) -> Vec<[u32; 2]> {
    let mut s = natural.map(|v| v.max(1));
    if vector {
        let k = f64::from(ATLAS_SIDE) / f64::from(s[0].max(s[1]));
        s = s.map(|v| ((f64::from(v) * k).round() as u32).clamp(1, ATLAS_SIDE));
    }
    while s[0].max(s[1]) > ATLAS_SIDE {
        s = s.map(|v| v.div_ceil(2));
    }
    let mut out = vec![s];
    while s[0].max(s[1]) > THUMB_SIDE {
        s = s.map(|v| v.div_ceil(2));
        out.push(s);
    }
    out
}

/// Metade de `src` (`w × h`, RGBA reto) para `to`: a média de cada 2×2 PESADA pelo alfa (um píxel
/// transparente não escurece a borda), a última coluna/linha ímpar repetida.
pub(super) fn halve(src: &[u8], [w, h]: [u32; 2], to: [u32; 2]) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; to[0] as usize * to[1] as usize * 4];
    for y in 0..to[1] as usize {
        for x in 0..to[0] as usize {
            let mut acc = [0u32; 4];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let (sx, sy) = ((2 * x + dx).min(w - 1), (2 * y + dy).min(h - 1));
                let p = &src[(sy * w + sx) * 4..][..4];
                let a = u32::from(p[3]);
                for c in 0..3 {
                    acc[c] += u32::from(p[c]) * a;
                }
                acc[3] += a;
            }
            let o = &mut out[(y * to[0] as usize + x) * 4..][..4];
            for c in 0..3 {
                o[c] = (acc[c] + acc[3] / 2).checked_div(acc[3]).unwrap_or(0) as u8;
            }
            o[3] = ((acc[3] + 2) / 4) as u8;
        }
    }
    out
}

/// W16 — metade (ou o tamanho `to` de uma halving) de uns píxeis.
pub(crate) fn halve_to(px: &Pixels, to: [u32; 2]) -> Pixels {
    Pixels {
        size: to,
        rgba: halve(&px.rgba, px.size, to),
    }
}

/// A pirâmide inteira de uns píxeis (o 1.º nível já reduzido ao atlas, se preciso).
pub(super) fn pyramid(px: Pixels) -> Option<Levels> {
    let sizes = level_sizes(px.size, false);
    let at0 = px.size;
    if px.rgba.len() != at0[0] as usize * at0[1] as usize * 4 {
        return None;
    }
    // Cada nível é o `Arc` do anterior meio: nenhuma cópia dos píxeis.
    let (mut cur, mut at) = (Arc::new(px.rgba), at0);
    let mut out = Vec::with_capacity(sizes.len());
    for s in sizes {
        while at != s {
            let next = at.map(|v| v.div_ceil(2));
            cur = Arc::new(halve(&cur, at, next));
            at = next;
        }
        let img = StableImage::from_rgba(cur.clone(), s[0], s[1])?;
        out.push((img, s));
    }
    Some(out)
}
