//! ⭐ **O preview da Borracha** (dono 24/10: *«para Eraser o preview deve ter um fundo cheio de cor sólida com um
//! traço similar ao Brush normal»*): a tela cheia de [`COR_DO_FUNDO_DA_BORRACHA`] e a borracha faz o S do Pincel,
//! pelo mesmo `PainterTool` e o mesmo gesto da tela — o que ela apaga mostra a folha.
//!
//! O Blur, o Smear (e a Faca), o Clone e a Máscara não têm preview (dono 24/10; `LinhaDaFerramenta::Preview`).

use super::super::PaintMode;
use crate::PainterTool;

/// A borracha do trilho está na mão (o modo Paint com a borracha armada)?
#[must_use]
pub(super) fn e_a_borracha(t: &PainterTool) -> bool {
    t.paint.paint_mode == PaintMode::Paint && t.paint.eraser
}

/// A cor cheia do fundo do preview da borracha — o vermelho das miniaturas das ferramentas.
pub const COR_DO_FUNDO_DA_BORRACHA: [u8; 3] = [191, 0, 18];

/// A tela `lw × lh` cheia de [`COR_DO_FUNDO_DA_BORRACHA`], opaca: o que a borracha do preview apaga.
#[must_use]
pub(super) fn tela_cheia(lw: u32, lh: u32) -> Vec<u8> {
    let [r, g, b] = COR_DO_FUNDO_DA_BORRACHA;
    [r, g, b, 255].repeat((lw * lh) as usize)
}

/// `px` (RGBA, alfa direto) composto sobre o branco opaco.
#[must_use]
pub(super) fn sobre_o_branco(px: &[u8]) -> Vec<u8> {
    px.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|&[r, g, b, a]| {
            let sobre = |c: u8| {
                ((u32::from(c) * u32::from(a) + 255 * (255 - u32::from(a)) + 127) / 255) as u8
            };
            [sobre(r), sobre(g), sobre(b), 255]
        })
        .collect()
}

#[cfg(test)]
#[path = "preview_borracha_tests.rs"]
mod tests;
