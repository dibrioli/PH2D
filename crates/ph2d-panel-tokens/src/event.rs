//! Painel → shell. Cada braço sai da MESMA lista que o `paint` percorre.
//!
//! ⚠️ **A swatch NÃO aparece aqui, e a ausência é a feature.** Ela é alvo de picker: o clique
//! nela é tratado pelo dispatch genérico do `register_picker_swatch` (abre o OKLCH), e o valor
//! escolhido é lido pela shell no frame seguinte. Um braço para ela aqui seria a segunda resposta
//! a *"quem escreve a cor?"*.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_tokens::{ColorToken, NumToken};

use crate::TokensPanel;
use crate::state::{TokenFamily, TokensIntent, TokensPanelState, push_intent};

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
        // ⚠️ **O chip numérico não tem braço de `Click`, ele tem este.** Um `NumberInput` publica o
        // que o artista digitou/arrastou como `ValueChanged`, e o valor vive no store do HOST — o
        // painel lê-o e enfileira; quem escreve a camada continua a ser a shell.
        WidgetEvent::ValueChanged(id) => {
            if let Some(row) = num_row_of(id, ids::tokens_num_chip_id) {
                if let Some(px) = host.store().number_value(id) {
                    #[allow(clippy::cast_possible_truncation)]
                    push_intent(TokensIntent::NumSet { row, px: px as f32 });
                }
                true
            } else {
                false
            }
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
                apply_link_click(state, TokenFamily::Colour, row);
                true
            } else if let Some(row) = num_row_of(id, ids::tokens_num_reset_id) {
                push_intent(TokensIntent::NumReset(row));
                true
            } else if let Some(row) = num_row_of(id, ids::tokens_num_link_id) {
                apply_link_click(state, TokenFamily::Num, row);
                true
            } else if let Some(row) = num_row_of(id, ids::tokens_num_fx_id) {
                // ⚠️ Abrir um campo NÃO é uma edição — é o painel a lembrar-se de onde o gesto
                // começou —, então isto não atravessa a fila de intents. A mesma assimetria do
                // ARMAR do elo, e pelo mesmo motivo: só o que ESCREVE atravessa.
                state.fx_open = if state.fx_open == Some(row) {
                    None
                } else {
                    Some(row)
                };
                true
            } else {
                false
            }
        }
        // **O COMMIT da fórmula** — Enter (`Submit`) ou o campo a perder o foco (`Blur`), as duas
        // portas que o dispatch global já emite. ⚠️ As duas, e não só o Enter: um campo abandonado
        // com o texto certo escrito dentro dele lê-se como *"eu autorei isto"*.
        WidgetEvent::Submit(id) | WidgetEvent::Blur(id) => {
            if let Some(row) = num_row_of(id, ids::tokens_num_formula_id) {
                let src = host.store().text(id).unwrap_or_default().to_string();
                // ⚠️ Fechar o campo é do PAINEL e acontece sempre; escrever é da shell e pode ser
                // recusado. Deixá-lo aberto até a shell responder faria um Enter que não pegou
                // parecer um Enter que não chegou.
                state.fx_open = None;
                push_intent(TokensIntent::NumFormula { row, src });
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

/// A linha numérica de um id derivado, se for de alguma.
///
/// ⚠️ A varredura é sobre `NumToken::ALL` — o **mesmo** intervalo que o `populate` regista e que o
/// `paint` percorre. Um teto que só um dos três conhecesse deixaria as últimas linhas pintadas e
/// mortas sob o rato.
fn num_row_of(id: ph2d_a11y::NodeId, of: fn(usize) -> ph2d_a11y::NodeId) -> Option<usize> {
    (0..NumToken::ALL.len()).find(|&r| of(r) == id)
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
///
/// ⚠️ **Armar noutra FAMÍLIA abandona o gesto**, nunca o fecha: um elo atravessa famílias no modelo
/// numérico (px é px), mas **nunca** entre px e cor — não há valor que uma cor possa dar a um
/// espaçamento. Fechá-lo aqui exigiria um terceiro `TokenValue` que não existe; abandoná-lo é o
/// que o artista vê ao clicar noutra lista.
fn apply_link_click(state: &mut TokensPanelState, family: TokenFamily, row: usize) {
    match state.armed {
        Some((f, from)) if f == family && from == row => state.armed = None,
        Some((f, from)) if f == family => {
            match family {
                TokenFamily::Colour => push_intent(TokensIntent::Link { from, to: row }),
                TokenFamily::Num => push_intent(TokensIntent::NumLink { from, to: row }),
            }
            state.armed = None;
        }
        _ => state.armed = Some((family, row)),
    }
}
