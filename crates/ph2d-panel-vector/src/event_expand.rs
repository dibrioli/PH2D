//! **Os dois selectores do Offset Path** (a junção da quina e o lado do contorno) — irmão de
//! [`super`] pelo teto de 600 LOC do painel, com o mesmo corte do `event_texpat` e do
//! `event_contour`: um assunto, uma porta.
//!
//! ⚠️ **Os dois são PANEL-LOCAL de propósito:** escolher a quina ou o lado não edita o documento
//! (só o clique em *Offset Path* edita), então a escolha pára aqui em vez de virar um
//! `ToolPanelEvent` que a shell teria de guardar num 2.º lugar — e dois lugares a guardar a mesma
//! escolha divergem.
//!
//! ⚠️ Cada um tem **duas** funções e não uma: o `apply_event` consulta o índice para saber se
//! ATENDE o clique, e o `pick_*` para saber o QUE gravar. Duas perguntas, uma tabela.

use crate::ids;
use crate::state;
use ph2d_editor_core::panel::{PanelHostInternal, seam_reset_button};

/// Grava a junção escolhida para o **Offset Path** e ENGOLE o clique.
pub(super) fn pick_expand_join(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
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
pub(super) fn expand_join_index(id: ph2d_a11y::NodeId) -> Option<u8> {
    match id {
        _ if id == ids::VECTOR_EXPAND_JOIN_MITER => Some(0),
        _ if id == ids::VECTOR_EXPAND_JOIN_ROUND => Some(1),
        _ if id == ids::VECTOR_EXPAND_JOIN_BEVEL => Some(2),
        _ => None,
    }
}

/// **Qual contorno o Offset Path move** — panel-local, o irmão do [`pick_expand_join`].
pub(super) fn pick_expand_side(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) -> bool {
    seam_reset_button(host, id);
    if let Some(s) = expand_side_index(id) {
        state::set_expand_side(s);
    }
    true
}

/// O índice do lado do Offset (`0` Outer · `1` Inner · `2` Both), ou `None`. Porta única,
/// como a da junção.
pub(super) fn expand_side_index(id: ph2d_a11y::NodeId) -> Option<u8> {
    match id {
        _ if id == ids::VECTOR_EXPAND_SIDE_OUTER => Some(0),
        _ if id == ids::VECTOR_EXPAND_SIDE_INNER => Some(1),
        _ if id == ids::VECTOR_EXPAND_SIDE_BOTH => Some(2),
        _ => None,
    }
}
