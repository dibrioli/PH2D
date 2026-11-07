//! A imagem de um mapa em iso e hex, onde as caixas dos tiles se sobrepõem.

use std::collections::BTreeMap;

use super::{TileGrid, Tileset, virada};
use crate::blend::BlendMode;

/// Um píxel de tile por cima do que já lá está: em iso e hex as caixas sobrepõem-se.
fn por_cima(dst: &mut [u8], src: &[u8], vazio: &[u8]) {
    if src == vazio {
        return;
    }
    let meio = src.len() == 4 && src[3] != u8::MAX && dst != vazio;
    if meio {
        let (d, s) = (
            [dst[0], dst[1], dst[2], dst[3]],
            [src[0], src[1], src[2], src[3]],
        );
        dst.copy_from_slice(&crate::blend::blend(BlendMode::Normal, d, s, u8::MAX));
    } else {
        dst.copy_from_slice(src);
    }
}

/// A imagem do mapa em iso e hex. Coluna a coluna, de cima para baixo: a ordem que o Godot desenha
/// (`docs/PixelArt/P10_oraculos.md`, M8a).
pub(super) fn expand_forma(g: &TileGrid, set: &Tileset, bpp: usize) -> Vec<u8> {
    let (tw, th) = (i32::from(set.tile_w), i32::from(set.tile_h));
    let caixa = g.caixa(set);
    let (lw, lh) = (caixa.w as usize, caixa.h as usize);
    let vazio = &set.tiles[0][..bpp];
    let mut out = vazio.repeat(lw * lh);
    let ((c0, r0), (fx, fy)) = g.ancora(set.forma, tw, th);
    let mut viradas: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    for i in 0..g.w as i32 {
        for j in 0..g.h as i32 {
            let t = g.tiles[j as usize * g.w as usize + i as usize];
            let Some(src) = set.tiles.get(t.id() as usize).filter(|_| t.id() != 0) else {
                continue;
            };
            let img = viradas
                .entry(t.0)
                .or_insert_with(|| virada(src, t, tw as u32, th as u32, bpp));
            let (ox, oy) = set.forma.origem(c0 + i, r0 + j, tw, th);
            let (dx, dy) = (fx + ox - caixa.x, fy + oy - caixa.y);
            for y in 0..th {
                for x in 0..tw {
                    let s = (y * tw + x) as usize * bpp;
                    let d = ((dy + y) as usize * lw + (dx + x) as usize) * bpp;
                    por_cima(&mut out[d..d + bpp], &img[s..s + bpp], vazio);
                }
            }
        }
    }
    out
}
