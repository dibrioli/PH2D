//! As contas de píxeis da subida de um palco: recortar, premultiplicar e misturar a marca do artista.

use super::Caixa;
use ph2d_tool_painter::macro_track::marca::{self as marca_do_artista, MarcaDoArtista};

/// Acrescenta a `out` o retângulo `(x, y, w, h)` de uma tela RGBA de largura `largura`, linha a linha.
pub(super) fn recorta(
    rgba: &[u8],
    largura: u32,
    (x, y, w, h): (u32, u32, u32, u32),
    out: &mut Vec<u8>,
) {
    let (x, w) = (x as usize, w as usize);
    let linhas = rgba.chunks_exact(largura as usize * 4).skip(y as usize);
    for linha in linhas.take(h as usize) {
        out.extend_from_slice(&linha[x * 4..(x + w) * 4]);
    }
}

/// Copia o retângulo `c` de `rgba` (straight) para `espelho`, premultiplicado; as duas de largura `largura`.
pub(super) fn premultiplica_na_caixa(espelho: &mut [u8], rgba: &[u8], largura: u32, c: Caixa) {
    let (lw, x, w) = (largura as usize, c.0 as usize, c.2 as usize);
    for j in c.1 as usize..(c.1 + c.3) as usize {
        let a = (j * lw + x) * 4;
        let linha = &mut espelho[a..a + w * 4];
        linha.copy_from_slice(&rgba[a..a + w * 4]);
        ph2d_render::premul::premultiply_rgba8(linha);
    }
}

/// **A lei da mistura** (W10, `docs/Macros/08_plano_W10_cobertura.md`): em `out` (o retângulo `c` do
/// palco, premultiplicado), cada texel que o artista tocou com a cobertura `k` sobre o que estava por
/// baixo `B` passa a `atual + (1 − k)·(palco − B)` — a tinta dele por cima do clipe naquele instante
/// (`k` cheia é a imagem `atual`, premultiplicada, de largura `largura`).
pub(super) fn mistura_na_caixa(
    out: &mut [u8],
    marca: &MarcaDoArtista,
    atual: &[u8],
    largura: u32,
    c: Caixa,
) {
    use marca_do_artista::LADO;
    let (cx, cy, cw, ch) = c;
    let (lw, w) = (largura as usize, cw as usize);
    for ly in cy / LADO..(cy + ch).div_ceil(LADO) {
        for lx in cx / LADO..(cx + cw).div_ceil(LADO) {
            let Some(l) = marca.ladrilho(lx, ly) else {
                continue;
            };
            let (x0, x1) = ((lx * LADO).max(cx), ((lx + 1) * LADO).min(cx + cw));
            for y in (ly * LADO).max(cy)..((ly + 1) * LADO).min(cy + ch) {
                for x in x0..x1 {
                    let k = l.c(((y % LADO) * LADO + x % LADO) as usize);
                    if k == 0 {
                        continue;
                    }
                    let o = (((y - cy) as usize) * w + (x - cx) as usize) * 4;
                    let a = ((y as usize) * lw + x as usize) * 4;
                    let (o, a) = (&mut out[o..o + 4], &atual[a..a + 4]);
                    if k == marca_do_artista::CHEIA {
                        o.copy_from_slice(a);
                        continue;
                    }
                    let b = l.b(((y % LADO) * LADO + x % LADO) as usize);
                    // O `B` premultiplicado (o arredondamento de `premultiply_rgba8`) e `(1 − k)` em 16
                    // bits: `· resta >> 16` em vez de `/ CHEIA` erra menos de 0,004 nível.
                    let alfa = u32::from(b[3]);
                    let resta = i32::from(marca_do_artista::CHEIA - k);
                    for (n, (o, &a)) in o.iter_mut().zip(a).enumerate() {
                        let bp = if n == 3 || alfa == 255 {
                            u32::from(b[n])
                        } else {
                            (u32::from(b[n]) * alfa + 127) / 255
                        };
                        let d = (resta * (i32::from(*o) - bp as i32) + 32_768) >> 16;
                        *o = (i32::from(a) + d).clamp(0, 255) as u8;
                    }
                }
            }
        }
    }
}
