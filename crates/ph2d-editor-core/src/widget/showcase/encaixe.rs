//! ⭐⭐ **A lei responsiva dos corpos da galeria** (doc `CartoesUI/08`): cada corpo dispõe-se no
//! `w` que recebe, até à coluna lateral no mínimo (`PANEL_MIN_W_PX`). O que não cabe numa fileira
//! passa a duas — o rótulo EM CIMA do controlo (regra D13 dos cartões), a frase quebra, os botões
//! repartem-se por fileiras. Na janela flutuante (larga) tudo cabe e nada muda.

use super::*;
use ph2d_text::FontWeight;

/// A largura que `texto` pinta a `font` (o peso do `paint_text`).
pub(super) fn largura(text_system: &mut TextSystem, texto: &str, font: f32) -> f32 {
    text_system.prefix_width_weighted(texto, font, FontWeight::MEDIUM)
}

/// O texto cabe inteiro em `w`?
pub(super) fn cabe(text_system: &mut TextSystem, texto: &str, font: f32, w: f32) -> bool {
    largura(text_system, texto, font) <= w
}

/// Quantas peças de largura IGUAL vão numa fileira: a maior contagem que dá a cada uma a caixa que
/// a palavra mais larga pede (`precisa`). Nunca menos de uma.
pub(super) fn pecas_por_fileira(w: f32, gap: f32, precisa: f32, n: usize) -> usize {
    (1..=n.max(1))
        .rev()
        .find(|&k| (w - gap * (k - 1) as f32) / k as f32 >= precisa)
        .unwrap_or(1)
}

/// Uma frase que QUEBRA em `w` em vez de se cortar; devolve a altura que ocupou.
#[allow(clippy::too_many_arguments)]
#[track_caller]
pub(super) fn frase(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    texto: &str,
    x: f32,
    y: f32,
    font: f32,
    w: f32,
    cor: ph2d_vector::Color,
) -> f32 {
    crate::paint::paint_text_block(text_system, scene, texto, x, y, font, w, cor)
}

#[cfg(test)]
mod tests {
    use super::pecas_por_fileira;

    /// ⭐ Larga: as três numa fileira; estreita: duas; mais estreita que a palavra: uma.
    #[test]
    fn as_pecas_repartem_se_pela_largura() {
        assert_eq!(pecas_por_fileira(300.0, 4.0, 50.0, 3), 3); // LITERAL-PX-OK: teste
        assert_eq!(pecas_por_fileira(174.0, 4.0, 60.0, 3), 2); // LITERAL-PX-OK: teste
        assert_eq!(pecas_por_fileira(40.0, 4.0, 60.0, 3), 1); // LITERAL-PX-OK: teste
    }
}
