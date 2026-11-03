//! ⭐⭐ **A lei responsiva dos corpos da galeria** (doc `CartoesUI/08`): cada corpo dispõe-se no
//! `w` que recebe, até à coluna lateral no mínimo (`PANEL_MIN_W_PX`). O que não cabe numa fileira
//! passa a duas — o rótulo EM CIMA do controlo (regra D13 dos cartões), a frase quebra, os botões
//! repartem-se por fileiras. Na janela flutuante (larga) tudo cabe e nada muda.

use super::*;
use ph2d_text::FontWeight;

/// A medida de um corpo: `dentro` de um cartão ([`crate::widget::card_ink::com`]), `fora` na janela
/// flutuante (`PH2D_CARDS=0`), que não muda.
pub(super) fn no_cartao(dentro: f32, fora: f32) -> f32 {
    if crate::widget::card_ink::card_body().is_some() {
        dentro
    } else {
        fora
    }
}

/// A altura de uma fileira de controlo: a do campo do cartão, `fora` na janela flutuante.
pub(super) fn fila_h(fora: f32) -> f32 {
    no_cartao(m::FIELD_H, fora)
}

/// A cor de um nome de linha ou de uma legenda: a do rótulo dentro de um cartão.
pub(super) fn tom_de_nome(theme: Theme) -> ph2d_vector::Color {
    crate::widget::card_ink::card_body().map_or_else(
        || resolve(ColorToken::Text1, theme),
        |ink| ink.color(ColorToken::CardLabel),
    )
}

/// O nome de uma amostra: o de ferramenta (`chave`, da tabela) num cartão; o de catálogo na janela
/// flutuante.
pub(super) fn nome(na_janela: &'static str, chave: &'static str) -> &'static str {
    if crate::widget::card_ink::card_body().is_some() {
        ph2d_i18n::tr(chave)
    } else {
        na_janela
    }
}

/// O rótulo acima de um campo no corpo de um cartão: a lei `LABEL_TOP`/`LABEL_PX`/`LABEL_BOTTOM` do
/// vocabulário, na tinta do rótulo; devolve o topo do campo. Só o que não cabe numa fileira quebra.
#[allow(clippy::too_many_arguments)]
pub(super) fn rotulo_em_cima(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    ink: crate::widget::card_ink::CardInk,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
) -> f32 {
    let topo = y + m::LABEL_TOP;
    if cabe(text_system, label, m::LABEL_PX, w) {
        crate::widget::card_field::paint_label(scene, text_system, ink, x, topo, w, label);
        return topo + m::LABEL_PX + m::LABEL_BOTTOM;
    }
    let cor = ink.color(ColorToken::CardLabel);
    let h = frase(scene, text_system, label, x, topo, m::LABEL_PX, w, cor);
    topo + h + m::LABEL_BOTTOM
}

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
    use super::*;
    use crate::widget::card_ink::{CardInk, com};

    /// ⭐ Larga: as três numa fileira; estreita: duas; mais estreita que a palavra: uma.
    #[test]
    fn as_pecas_repartem_se_pela_largura() {
        assert_eq!(pecas_por_fileira(300.0, 4.0, 50.0, 3), 3); // LITERAL-PX-OK: teste
        assert_eq!(pecas_por_fileira(174.0, 4.0, 60.0, 3), 2); // LITERAL-PX-OK: teste
        assert_eq!(pecas_por_fileira(40.0, 4.0, 60.0, 3), 1); // LITERAL-PX-OK: teste
    }

    /// ⭐ A lei do ritmo: dentro de um cartão o corpo de sempre responde as medidas dos campos do
    /// vocabulário; fora dele, as de sempre. *Mutação: tirar o ramo do `no_cartao` ⇒ a 1.ª asserção.*
    #[test]
    fn inside_a_card_the_gallery_body_answers_the_card_rhythm_and_outside_nothing_changes() {
        let fora = (row_gap(), field_h(), fila_h(ROW_H_PX));
        assert_eq!(fora.0, ph2d_tokens::control_gap_px());
        assert_eq!(fora.1, Spacing::Xl3.px());
        assert_eq!(fora.2, ROW_H_PX);
        assert_ne!(
            fora.0,
            m::FIELD_ROW_GAP,
            "fixtura: as duas leis têm de diferir"
        );
        com(CardInk::of_list(Theme::Dark), || {
            assert_eq!(row_gap(), m::FIELD_ROW_GAP);
            assert_eq!(field_h(), m::FIELD_H);
            assert_eq!(fila_h(ROW_H_PX), m::FIELD_H);
        });
        assert_eq!(
            (row_gap(), field_h(), fila_h(ROW_H_PX)),
            fora,
            "a lei vazou"
        );
    }
}
