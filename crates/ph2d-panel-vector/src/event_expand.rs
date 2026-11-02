//! **As opções da ferramenta Offset Path** (fila ②: Corners, Holes, Result e o Remove) — irmão de
//! [`super`] pelo teto de 600 LOC do painel: um assunto, uma porta.
//!
//! ⚠️ **PANEL-LOCAL de propósito:** a escolha fica no estado do painel e a shell lê-a a cada
//! quadro (com a `Distance (px)`), em vez de um `ToolPanelEvent` guardado num 2.º lugar.
//!
//! ⚠️ Cada um tem **duas** funções e não uma: o `apply_event` consulta o índice para saber se
//! ATENDE o clique, e o `pick_*` para saber o QUE gravar. Duas perguntas, uma tabela.

use crate::state;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};

/// ⭐ **Os eventos do Offset Path** (F48) — `None` = não é dele. Todos panel-local: a distância
/// (`VECTOR_EXPAND_OFFSET`) é lida do store pela shell a cada quadro, e o `Remove` zera-a (a shell
/// vê o campo mudar e DESARMA a seleção — a mesma porta de quem digita `0`).
pub(super) fn expand_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> Option<bool> {
    use crate::ids::{VECTOR_EXPAND_MITER, VECTOR_EXPAND_OFFSET, VECTOR_EXPAND_REMOVE};
    Some(match ev {
        WidgetEvent::Click(id) if expand_join_index(id).is_some() => pick_expand_join(host, id),
        WidgetEvent::Click(id) if expand_side_index(id).is_some() => pick_expand_side(host, id),
        WidgetEvent::Click(id) if result_of(id).is_some() => {
            seam_reset_button(host, id);
            state::set_expand_result(result_of(id).unwrap_or(0));
            true
        }
        WidgetEvent::Click(id) if id == VECTOR_EXPAND_REMOVE => {
            seam_reset_button(host, id);
            host.store_mut().set_number_value(VECTOR_EXPAND_OFFSET, 0.0);
            true
        }
        WidgetEvent::ValueChanged(id) if id == VECTOR_EXPAND_MITER => {
            state::set_expand_miter(host.store().number_value(id).unwrap_or(1.0));
            true
        }
        WidgetEvent::ValueChanged(id) if id == VECTOR_EXPAND_OFFSET => true,
        _ => return None,
    })
}

/// O código do `Result` que o chip `id` escolhe (`0` Replace · `1` Behind · `2` In front).
fn result_of(id: ph2d_a11y::NodeId) -> Option<u8> {
    use crate::ids::{
        VECTOR_EXPAND_RESULT_BEHIND, VECTOR_EXPAND_RESULT_FRONT, VECTOR_EXPAND_RESULT_REPLACE,
    };
    [
        VECTOR_EXPAND_RESULT_REPLACE,
        VECTOR_EXPAND_RESULT_BEHIND,
        VECTOR_EXPAND_RESULT_FRONT,
    ]
    .iter()
    .position(|r| *r == id)
    .and_then(|i| u8::try_from(i).ok())
}

/// Grava a junção escolhida para o **Offset Path** e ENGOLE o clique.
fn pick_expand_join(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    if let Some(j) = expand_join_index(id) {
        state::set_expand_join(j);
    }
    true
}

/// O índice da junção do Offset Path que este id escolhe (`0` Miter · `1` Round · `2`
/// Bevel), ou `None` se o id não é um dos três chips. Porta única: o `apply_event` a
/// consulta para saber se ATENDE o clique, e [`pick_expand_join`] para saber o QUE gravar —
/// duas perguntas, uma tabela.
fn expand_join_index(id: ph2d_a11y::NodeId) -> Option<u8> {
    match id {
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_MITER => Some(0),
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_ROUND => Some(1),
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_JOIN_BEVEL => Some(2),
        _ => None,
    }
}

/// **Qual contorno o Offset Path move** — panel-local, o irmão do [`pick_expand_join`].
fn pick_expand_side(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    if let Some(s) = expand_side_index(id) {
        state::set_expand_side(s);
    }
    true
}

/// O índice do lado do Offset (`0` Outer · `1` Inner · `2` Both), ou `None`. Porta única,
/// como a da junção.
fn expand_side_index(id: ph2d_a11y::NodeId) -> Option<u8> {
    match id {
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_OUTER => Some(0),
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_INNER => Some(1),
        _ if id == ph2d_tool_vector::ids::VECTOR_EXPAND_SIDE_BOTH => Some(2),
        _ => None,
    }
}
