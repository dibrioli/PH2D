//! **AS FERRAMENTAS QUE SÓ MOVEM PIXELS PRESERVAM A PRECISÃO** — plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! W4-bis.
//!
//! Enio, 2026-08-20: *"após aplicar algumas das tools a sprite volta para RGBA8 no inspector"*.
//!
//! # A distinção que este módulo existe para fazer
//!
//! As nove ferramentas de imagem não são todas iguais. Três delas — **Trim Transparency**,
//! **Make Square** e **Padding** — **nunca calculam o valor de um pixel**: recortam, copiam e
//! preenchem o resto com transparente. As outras (Upscale, Rasterize, Real Size, Color
//! Equalization, BG-Removal) fazem aritmética sobre as cores.
//!
//! ⚠️ Para as três primeiras, **a perda de precisão era gratuita**. Elas recebiam a imagem já
//! convertida para 8 bits só porque é isso que o `SpriteImage` sabe carregar, e devolviam 8 bits —
//! destruindo 16 bits sem sequer olhar para um valor. *Uma conversão que o algoritmo não precisa é
//! uma perda que ninguém pediu.*
//!
//! # Por que isto NÃO toca o contrato congelado
//!
//! ⛔ A saída óbvia seria alargar o `RasterEditTool` (§6 do `CLAUDE.md`) para falar 16 bits. Não é
//! preciso: as três ferramentas **já publicam a sua geometria** — o `TrimResult` traz os `bounds`,
//! o `MakeSquareResult` traz `size`/`offset_x`/`offset_y`, e o padding traz as suas margens. O
//! shell aplica **a mesma geometria** ao buffer de 16 bits e nunca pergunta nada de novo ao tool.
//!
//! *Um contrato congelado emenda-se por necessidade medida — e aqui a medição diz que não há
//! necessidade nenhuma.*
//!
//! # O que continua a descer para 8 bits, e porquê
//!
//! As que **resamplam ou recolorem** (Upscale, Rasterize, Real Size, Color Equalization,
//! BG-Removal) continuam a converter, com o aviso do funil. Preservar 16 bits nelas exigiria um
//! resampler e uma pilha de cor de 16 bits — código novo e real, não plumbing. Fingir que
//! preservam (converter o resultado de volta para cima) seria a **pior** das opções: o rótulo
//! diria 16 e os valores teriam passado por 8.

/// **Copia um rectângulo do buffer de origem para um canvas novo**, preenchendo o resto com zero
/// (transparente).
///
/// É a única operação que as três ferramentas geométricas fazem, dita uma vez:
/// - **Trim** = recorte (`src_rect` é o conteúdo, o canvas tem o tamanho dele, destino `(0, 0)`).
/// - **Make Square** / **Padding** = moldura (`src_rect` é tudo, o canvas é maior, destino é o
///   deslocamento).
///
/// ⚠️ Trabalha em **elementos**, não em bytes: são quatro por pixel em qualquer precisão. Foi
/// confundir as duas grandezas que produziu o pânico das ferramentas (um `bytes_per_pixel` fixo
/// em 4 numa textura de 8 bytes por pixel).
///
/// Recusa devolvendo `None` quando o rectângulo não cabe na origem ou no destino — a mesma lei do
/// `crop_region` irmão: *errar de forma visível, nunca com aritmética que dá a volta*.
#[allow(clippy::too_many_arguments)]
pub(crate) fn blit_rgba16(
    src: &[u16],
    src_w: u32,
    src_h: u32,
    src_rect: [u32; 4],
    out_w: u32,
    out_h: u32,
    dst_x: u32,
    dst_y: u32,
) -> Option<Vec<u16>> {
    const CH: usize = 4;
    let [rx, ry, rw, rh] = src_rect;
    if src.len() != (src_w as usize) * (src_h as usize) * CH {
        return None;
    }
    if u64::from(rx) + u64::from(rw) > u64::from(src_w)
        || u64::from(ry) + u64::from(rh) > u64::from(src_h)
        || u64::from(dst_x) + u64::from(rw) > u64::from(out_w)
        || u64::from(dst_y) + u64::from(rh) > u64::from(out_h)
    {
        return None;
    }
    let mut out = vec![0u16; (out_w as usize) * (out_h as usize) * CH];
    let src_row = src_w as usize * CH;
    let out_row = out_w as usize * CH;
    let run = rw as usize * CH;
    for r in 0..rh as usize {
        let from = (ry as usize + r) * src_row + rx as usize * CH;
        let to = (dst_y as usize + r) * out_row + dst_x as usize * CH;
        out[to..to + run].copy_from_slice(&src[from..from + run]);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma imagem em que cada pixel carrega o próprio `(x, y)` — assim um blit errado é legível:
    /// o pixel diz de onde veio.
    fn tagged(w: u32, h: u32) -> Vec<u16> {
        let mut v = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                v.extend_from_slice(&[x as u16, y as u16, 0, 0xFFFF]);
            }
        }
        v
    }

    /// **O recorte (Trim) leva os pixels CERTOS** — e cada um diz de onde veio.
    #[test]
    fn a_crop_takes_the_right_pixels() {
        let src = tagged(8, 8);
        let out = blit_rgba16(&src, 8, 8, [2, 3, 4, 2], 4, 2, 0, 0).expect("cabe");
        assert_eq!(out.len(), 4 * 2 * 4);
        // Primeiro pixel do recorte = origem (2, 3).
        assert_eq!(&out[0..4], &[2, 3, 0, 0xFFFF]);
        // Último = (5, 4).
        assert_eq!(&out[out.len() - 4..], &[5, 4, 0, 0xFFFF]);
    }

    /// **A moldura (Make Square / Padding) põe a origem no sítio e o resto TRANSPARENTE.**
    ///
    /// ⚠️ O `0` no alfa importa: uma moldura preenchida com opaco seria uma borda preta, que é o
    /// defeito clássico deste tipo de operação.
    #[test]
    fn a_frame_places_the_source_and_leaves_the_rest_transparent() {
        let src = tagged(2, 2);
        let out = blit_rgba16(&src, 2, 2, [0, 0, 2, 2], 4, 4, 1, 1).expect("cabe");
        // O canto (0,0) do destino não foi tocado.
        assert_eq!(
            &out[0..4],
            &[0, 0, 0, 0],
            "a moldura tem de ficar transparente"
        );
        // A origem (0,0) foi para o destino (1,1) — índice escrito como a conta que é:
        // `(linha × largura + coluna) × canais`, com o canvas 4×4 e 4 canais.
        let (row, col, out_w, ch) = (1usize, 1usize, 4usize, 4usize);
        let at = (row * out_w + col) * ch;
        assert_eq!(&out[at..at + 4], &[0, 0, 0, 0xFFFF]);
    }

    /// **A ida-e-volta de uma geometria é EXACTA** — nenhum valor é recalculado.
    ///
    /// É esta propriedade que justifica o módulo: se o blit alterasse um único valor, preservar
    /// 16 bits seria uma promessa vazia e mais valeria converter honestamente para 8.
    #[test]
    fn no_pixel_value_is_ever_recomputed() {
        let src = tagged(6, 5);
        // Emoldura e depois recorta de volta exactamente a mesma janela.
        let framed = blit_rgba16(&src, 6, 5, [0, 0, 6, 5], 10, 9, 2, 2).expect("cabe");
        let back = blit_rgba16(&framed, 10, 9, [2, 2, 6, 5], 6, 5, 0, 0).expect("cabe");
        assert_eq!(back, src, "a geometria nao pode mexer num unico valor");
    }

    /// **Um rectângulo que não cabe é RECUSADO**, nos dois lados.
    ///
    /// ⚠️ Sem isto a aritmética daria a volta e o resultado seria lixo bem-formado — o modo de
    /// falha que o `crop_region` irmão também recusa de propósito.
    #[test]
    fn a_rect_that_does_not_fit_is_refused_on_both_sides() {
        let src = tagged(4, 4);
        assert!(
            blit_rgba16(&src, 4, 4, [2, 0, 4, 2], 4, 2, 0, 0).is_none(),
            "sai pela direita da ORIGEM"
        );
        assert!(
            blit_rgba16(&src, 4, 4, [0, 0, 4, 4], 4, 4, 1, 0).is_none(),
            "sai pela direita do DESTINO"
        );
        assert!(
            blit_rgba16(&src, 4, 5, [0, 0, 4, 4], 4, 4, 0, 0).is_none(),
            "o buffer nao tem o tamanho que as dimensoes declaram"
        );
    }
}
