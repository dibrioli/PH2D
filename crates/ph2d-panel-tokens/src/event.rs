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
    _state: &mut TokensPanelState,
    host: &mut dyn PanelHostInternal,
    ev: WidgetEvent,
) -> EventOutcome {
    let consumed = match ev {
        WidgetEvent::Click(id) if id == ids::TOKENS_CLOSE => {
            host.set_panel_visible(TokensPanel::ID, false);
            true
        }
        WidgetEvent::Click(id) if id == ids::TOKENS_RESET_ALL => {
            push_intent(TokensIntent::ResetAll);
            true
        }
        WidgetEvent::Click(id) => {
            // A varredura é sobre `ColorToken::ALL` — o mesmo intervalo que o `populate` regista.
            // Um teto que o roteador conhecesse e o registro não deixaria as últimas linhas mortas
            // sob o rato.
            if let Some(row) = (0..ColorToken::ALL.len()).find(|&r| ids::tokens_reset_id(r) == id) {
                push_intent(TokensIntent::Reset(row));
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
