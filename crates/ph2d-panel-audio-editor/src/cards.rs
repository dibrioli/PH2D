//! ⭐ **O Audio Editor em cartões** (`docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): as mesmas oito secções, cada uma
//! um cartão com o MESMO id, o corpo pintado pelos mesmos pintores, na tinta do cartão. Mesmos ids e eventos;
//! `PH2D_CARDS=0` dá o de antes. A onda fica na janela flutuante da shell: só a moldura dela muda ([`moldura`]).

use crate::paint_sections::{SECTIONS, header_of, paint_corpo};
use crate::{AEDIT_CLOSE, AEDIT_PANEL, AudioEditorPanel};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Widget, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Color;

#[path = "cards_fileira.rs"]
pub(crate) mod fileira;
#[path = "cards_moldura.rs"]
pub mod moldura;

/// O papel de cada token deste painel num cartão: a caixa do campo, o acento do cartão, o texto dele.
const fn papel(token: ColorToken) -> ColorToken {
    match token {
        ColorToken::Bg2 | ColorToken::Bg3 => ColorToken::CardField,
        ColorToken::BgElev | ColorToken::AccentSoft => ColorToken::CardFieldHover,
        ColorToken::Accent | ColorToken::AccentHover | ColorToken::AccentPress => {
            ColorToken::KanbanBlueBold
        }
        ColorToken::AccentFg => ColorToken::KanbanBtnOnFg,
        ColorToken::Text1 => ColorToken::KanbanCardFg,
        ColorToken::Text2 | ColorToken::Text3 => ColorToken::CardLabel,
        other => other,
    }
}

/// A cor de `token`: no corpo de um cartão o tom dele; fora, o token de sempre, ao bit.
#[must_use]
pub(crate) fn tom(token: ColorToken, theme: Theme) -> Color {
    card_ink::card_body().map_or_else(
        || ph2d_editor_core::paint::resolve(token, theme),
        |ink| ink.color(papel(token)),
    )
}

/// O fundo de um botão no cartão: `(repouso, quente, premido)` nos papéis acima.
pub(crate) fn fundo_do_botao(
    ink: CardInk,
    (rest, hot, press): (ColorToken, ColorToken, ColorToken),
    (estado, t): (ButtonState, f32),
    enabled: bool,
) -> Color {
    let token = if !enabled {
        rest
    } else if estado == ButtonState::Pressed {
        press
    } else if estado == ButtonState::Hovered || t > 0.0 {
        hot
    } else {
        rest
    };
    ink.color(papel(token))
}

/// A altura de uma fileira de botões: a do campo do cartão, ou a linha de sempre.
#[must_use]
pub(crate) fn fila() -> f32 {
    if card_ink::card_body().is_some() {
        m::FIELD_H
    } else {
        crate::paint::ROW_H
    }
}

/// A altura da caixa do nome do clip.
#[must_use]
pub(crate) fn altura_do_nome() -> f32 {
    use ph2d_tokens::{Spacing, TypeToken};
    if card_ink::card_body().is_some() {
        m::FIELD_H
    } else {
        TypeToken::Sm.px() + Spacing::Sm.px() * 2.0
    }
}

/// O corpo aberto repete o resumo que o título só mostra dobrado: o facto continua à vista.
fn corpo(id: NodeId, b: &crate::paint_sections::Body, c: &mut CardCtx<'_>, y0: f32) -> f32 {
    let (_, resumo) = header_of(id);
    let mut y = y0;
    if let Some(texto) = resumo.as_deref().filter(|t| !t.is_empty()) {
        y = c.row(&[Cell::new(Widget::Readout { text: texto })], y);
    }
    let (x, w, ink) = (c.x, c.w, c.ink);
    let mut hits = crate::paint::ClippedHits::new(c.store, c.hit_index, fileira::SEM_RECORTE);
    let (scene, ts) = (&mut *c.scene, &mut *c.text_system);
    card_ink::com(ink, || {
        paint_corpo(id, y, x, w, b, scene, ts, ink.theme, &mut hits)
    })
}

/// A lista: o `Slot` da coluna, o título, o «×» de sempre e um cartão por secção.
pub(crate) fn paint(ctx: &mut PaintCtx) {
    let body = crate::paint::monta_body(ctx);
    ctx.host
        .store_mut()
        .mark_card_list_close(AEDIT_PANEL, AEDIT_CLOSE);
    let b = &body;
    let cartoes: Vec<CardDef<'_>> = SECTIONS
        .iter()
        .map(|&id| {
            let (title, summary) = header_of(id);
            CardDef {
                section: id,
                title,
                actions: Vec::new(),
                add: None,
                summary: summary.filter(|t| !t.is_empty()),
                body: Some(Box::new(move |c, y| corpo(id, b, c, y))),
            }
        })
        .collect();
    paint_card_list(ctx, AEDIT_PANEL, AudioEditorPanel::TITLE.tr(), &cartoes);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ Os papéis: repouso é a caixa do campo, quente e premido sobem, aceso é o acento do cartão, e o que diz um
    /// estado (aviso, sucesso) fica. *Mutação: tirar um braço de `papel` ⇒ vermelho.*
    #[test]
    fn cada_token_tem_o_seu_papel_no_cartao() {
        let tema = Theme::Dark;
        let ink = CardInk::of_list(tema);
        let fundo = |v, on| {
            fundo_do_botao(
                ink,
                (ColorToken::Bg3, ColorToken::BgElev, ColorToken::AccentSoft),
                v,
                on,
            )
        };
        let c = |t| ink.color(t);
        assert_eq!(
            fundo((ButtonState::Normal, 0.0), true),
            c(ColorToken::CardField)
        );
        assert_eq!(
            fundo((ButtonState::Hovered, 1.0), true),
            c(ColorToken::CardFieldHover)
        );
        assert_eq!(
            fundo((ButtonState::Pressed, 1.0), true),
            c(ColorToken::CardFieldHover)
        );
        assert_eq!(
            fundo((ButtonState::Hovered, 1.0), false),
            c(ColorToken::CardField)
        );
        assert_eq!(papel(ColorToken::Accent), ColorToken::KanbanBlueBold);
        assert_eq!(papel(ColorToken::Text1), ColorToken::KanbanCardFg);
        assert_eq!(papel(ColorToken::Text2), ColorToken::CardLabel);
        assert_eq!(papel(ColorToken::Warn), ColorToken::Warn);
        assert_eq!(
            tom(ColorToken::Text2, tema),
            ph2d_editor_core::paint::resolve(ColorToken::Text2, tema)
        );
    }
}
