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

/// **A mistura de volta de UMA linha:** `px ← lerp(px, borrado, peso/255)` em alfa
/// PRÉ-MULTIPLICADO ([`crate::mistura`]). `dst` é a linha RGBA na tela (`4·n` bytes), `borrada`
/// (recta, `0..=255`) e `peso` os `n` píxeis dela. É a mesma conta, bit a bit, do
/// [`crate::blur::blend_blurred`] com o peso `f32::from(p)/255` — um pixel de peso `0` não é tocado.
/// Até B24.2 era a recta nos 4 canais: o vazio `0,0,0,0` puxava o RGB para o preto (orla escura).
pub fn mistura_linha_por_peso(dst: &mut [u8], borrada: &[[f32; 4]], peso: &[u8]) {
    for ((px, src), &p) in dst.as_chunks_mut::<4>().0.iter_mut().zip(borrada).zip(peso) {
        let w = f32::from(p) / 255.0;
        if w <= 0.0 {
            continue;
        }
        crate::mistura::mistura_premultiplicada(px, *src, w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AMARELO: [u8; 4] = [250, 225, 20, 160];
    const W: u32 = 40;
    const H: u32 = 16;

    /// Vazio à esquerda, tinta amarela de α INTERMÉDIO à direita (T1-06, GT-44).
    fn camada() -> Vec<u8> {
        let mut b = vec![0u8; (W * H * 4) as usize];
        for y in 0..H {
            for x in 20..W {
                let i = ((y * W + x) * 4) as usize;
                b[i..i + 4].copy_from_slice(&AMARELO);
            }
        }
        b
    }

    fn escurecidos(b: &[u8]) -> (usize, usize) {
        let (mut tocados, mut escuros) = (0, 0);
        for p in b.as_chunks::<4>().0 {
            if p[3] > 0 {
                tocados += 1;
                escuros += usize::from((0..3).any(|c| i32::from(p[c]) < i32::from(AMARELO[c]) - 1));
            }
        }
        (tocados, escuros)
    }

    /// ⭐ **B24.2 (DF-46, IN-22): o borrão da pilha numa camada transparente não escurece a tinta.**
    /// O controlo é a mistura RECTA de antes, escrita à letra: na mesma fixtura ela escurece.
    #[test]
    fn o_borrao_da_pilha_em_alfa_intermedio_nao_escurece_a_orla() {
        let (bw, bh) = (W as usize, H as usize);
        let peso = vec![128u8; bw * bh];
        for nucleo in [BlurKernel::Caixa, BlurKernel::Binomial] {
            let tela = camada();
            let borrada = blur_region_borrado(&tela, W, H, 0, 0, bw, bh, 6, [false; 2], nucleo);
            let mut nova = tela.clone();
            blur_region_por_peso(&mut nova, W, H, 0, 0, bw, bh, 6, &peso, [false; 2], nucleo);
            let mut recta = tela.clone();
            for ((px, src), &p) in recta.as_chunks_mut::<4>().0.iter_mut().zip(&borrada).zip(&peso) {
                let w = f32::from(p) / 255.0;
                for c in 0..4 {
                    let d = f32::from(px[c]);
                    px[c] = (d + (src[c] - d) * w).round().clamp(0.0, 255.0) as u8;
                }
            }
            let (tocados, escuros) = escurecidos(&nova);
            let (_, escuros_recta) = escurecidos(&recta);
            assert!(tocados > bh * 20, "{nucleo:?}: CONTROLO — o borrão não chegou ao vazio");
            assert!(escuros_recta > bh, "{nucleo:?}: CONTROLO — a recta já não escurece ({escuros_recta})");
            assert_eq!(escuros, 0, "{nucleo:?}: {escuros} de {tocados} texels mais escuros que a tinta");
        }
    }
}
