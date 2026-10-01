//! **O borrão POR PESO da pilha do Composite Brush** — a convolução sobre uma região e a MISTURA
//! de volta por um peso por pixel já acumulado. Cortado do [`crate::blur`] por responsabilidade
//! (2026-10-01): ele é a porta da pilha, e o irmão é o borrão por DAB da ferramenta.

use crate::blur::{BlurKernel, blur_region};

/// **UMA passagem de borrão sobre a região, misturada por um PESO por pixel já acumulado.**
///
/// ⭐ É a porta que a pilha do Composite Brush usa desde que ela deixou de REPLAYAR a história: em
/// vez de `N` chamadas de [`blur_dab`] (uma por dab de cada lote), o peso de todos os dabs é
/// acumulado num plano e o borrão corre **uma vez**.
///
/// ⚠️ **Isto NÃO é igual a `N` passagens sequenciais, e a diferença é declarada:** `N` borrões
/// compõem-se num borrão MAIOR (`σ_total = σ√N`), enquanto uma passagem com peso `1 − Π(1−wᵢ)`
/// mistura o mesmo borrão mais fundo. A lei que a pilha declara é *«a camada Blur é aplicada sobre
/// o traço INTEIRO»* — uma aplicação —, e o `N` sequencial era um artefacto da implementação.
///
/// `peso` é do tamanho da REGIÃO (`bw·bh`, linha-maior), `1` byte por pixel, `255` = totalmente
/// borrado. Ele é da região e não do canvas de propósito: quem o monta já o tem por região, e uma
/// fatia do tamanho do canvas obrigaria uma alocação de `W·H` por evento.
#[allow(clippy::too_many_arguments)]
pub fn blur_region_por_peso(
    buf: &mut [u8],
    width: u32,
    height: u32,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    peso: &[u8],
    wrap: [bool; 2],
    nucleo: BlurKernel,
) {
    let (fw, fh) = (i64::from(width), i64::from(height));
    if bw == 0 || bh == 0 || peso.len() != bw * bh {
        return;
    }
    let blurred = blur_region(buf, fw, fh, min_x, min_y, bw, bh, k, wrap, nucleo);
    let stride = width as usize * 4;
    for j in 0..bh {
        let o = (min_y as usize + j) * stride + min_x as usize * 4;
        mistura_linha_por_peso(
            &mut buf[o..o + bw * 4],
            &blurred[j * bw..(j + 1) * bw],
            &peso[j * bw..(j + 1) * bw],
        );
    }
}

/// **A convolução de [`blur_region_por_peso`] SEM a mistura de volta** — a região borrada, em
/// premultiplicado `[f32; 4]` linha-maior (`bw·bh`), para quem mistura as linhas por conta própria.
///
/// ⭐ Existe para a pilha do Composite misturar as linhas na equipa de threads (ADR-0172): a mistura
/// é por pixel e cada linha é disjunta, e medida no rabisco do dono ela custava MAIS do que a
/// convolução (`1,66` contra `1,19 ms` por quadro), num núcleo só. A lei da mistura continua UMA —
/// [`mistura_linha_por_peso`] —, chamada pelas duas rotas.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn blur_region_borrado(
    buf: &[u8],
    width: u32,
    height: u32,
    min_x: i64,
    min_y: i64,
    bw: usize,
    bh: usize,
    k: usize,
    wrap: [bool; 2],
    nucleo: BlurKernel,
) -> Vec<[f32; 4]> {
    blur_region(
        buf,
        i64::from(width),
        i64::from(height),
        min_x,
        min_y,
        bw,
        bh,
        k,
        wrap,
        nucleo,
    )
}

/// **A mistura de volta de UMA linha:** `px ← px + (borrado − px)·(peso/255)`, canal a canal, com o
/// arredondamento de sempre. `dst` é a linha RGBA na tela (`4·n` bytes), `borrada` e `peso` os `n`
/// píxeis dela. ⚠️ É a mesma conta, bit a bit, do [`crate::blur::blend_blurred`] com o peso
/// `f32::from(p)/255` — um pixel de peso `0` não é tocado.
pub fn mistura_linha_por_peso(dst: &mut [u8], borrada: &[[f32; 4]], peso: &[u8]) {
    for ((px, src), &p) in dst.as_chunks_mut::<4>().0.iter_mut().zip(borrada).zip(peso) {
        let w = f32::from(p) / 255.0;
        if w <= 0.0 {
            continue;
        }
        for c in 0..4 {
            let d = f32::from(px[c]);
            px[c] = (d + (src[c] - d) * w).round().clamp(0.0, 255.0) as u8;
        }
    }
}
