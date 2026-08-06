//! Painel → shell. Cada braço sai da MESMA lista que o `paint` percorre.
//!
//! ⚠️ **A swatch NÃO aparece aqui, e a ausência é a feature.** Ela é alvo de picker: o clique
//! nela é tratado pelo dispatch genérico do `register_picker_swatch` (abre o OKLCH), e o valor
//! escolhido é lido pela shell no frame seguinte. Um braço para ela aqui seria a segunda resposta
//! a *"quem escreve a cor?"*.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_tokens::ColorToken;

use crate::TokensPanel;
use crate::state::{TokensIntent, TokensPanelState, push_intent};

pub(crate) fn apply_event(
    state: &mut TokensPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::Click(id) if id == ids::TOKENS_CLOSE => {
            // Fechar o painel desiste de um elo em curso — um gesto não sobrevive à superfície
            // onde ele estava a ser feito.
            state.armed = None;
            host.set_panel_visible(TokensPanel::ID, false);
            true
        }
        WidgetEvent::Click(id) if id == ids::TOKENS_RESET_ALL => {
            push_intent(TokensIntent::ResetAll);
            true
        }
        WidgetEvent::Click(id) => {
            // As varreduras são sobre `ColorToken::ALL` — o mesmo intervalo que o `populate`
            // regista. Um teto que o roteador conhecesse e o registro não deixaria as últimas
            // linhas mortas sob o rato.
            let n = ColorToken::ALL.len();
            if let Some(row) = (0..n).find(|&r| ids::tokens_reset_id(r) == id) {
                push_intent(TokensIntent::Reset(row));
                true
            } else if let Some(row) = (0..n).find(|&r| ids::tokens_link_id(r) == id) {
                apply_link_click(state, row);
                true
            } else {
                false
            }
        }
        _ => false,
    };
    if consumed {
        EventOutcome::Consumed
    } else {
        EventOutcome::Ignored
    }
}

/// **A máquina do elo, e ela tem três respostas** para o mesmo botão.
///
/// Nada armado ⇒ arma esta linha. Armada ESTA ⇒ desiste (o mesmo botão desfaz o próprio gesto, sem
/// um "cancelar" que ocuparia a tela permanentemente por causa de um estado que dura segundos).
/// Armada OUTRA ⇒ fecha o elo *aquela → esta*.
///
/// ⚠️ **O sentido é `armada → clicada`**, e não o contrário: o artista arma a linha que quer MUDAR
/// e depois aponta para onde ela deve olhar — a mesma ordem em que ele fala (*"o border segue o
/// surface"*). Invertê-lo compila igual e faz o gesto escrever no token errado.
///
/// ⚠️ E o auto-elo é **inalcançável por construção** aqui (clicar a própria linha desarma), o que
/// não dispensa a recusa no modelo: esta é a UI, e a porta é a lei.
fn apply_link_click(state: &mut TokensPanelState, row: usize) {
    match state.armed {
        Some(from) if from == row => state.armed = None,
        Some(from) => {
            push_intent(TokensIntent::Link { from, to: row });
            state.armed = None;
        }
        None => state.armed = Some(row),
    }
}
