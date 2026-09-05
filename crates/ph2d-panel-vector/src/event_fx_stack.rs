//! ⭐ **AS ROTAS DA PILHA DE EFEITOS** (Live Path Effects, ADR-0132) — as duas perguntas que o
//! `apply_event` faz sobre um id desta família, num sítio só.
//!
//! Irmão do [`crate::event_paint_stack`] pela mesma lei e pelo mesmo corte: uma família de ids
//! derivados de um índice responde às perguntas dela **num módulo**, e não em linhas espalhadas
//! por dois ficheiros de encaminhamento — que é onde elas estavam (o predicado do clique vivia no
//! `event.rs` e era consumido pelo `event_clicks.rs`, a três centenas de linhas de distância).
//!
//! ⚠️ **A varredura é sobre os TETOS** (`MAX_FX_ROWS` × `MAX_FX_ROW_PARAMS` = 16 comparações): os
//! ids são hashes de NOME, então não há aritmética que os inverta. É barato, e é o mesmo padrão
//! que os presets do Envelope e a pilha de aparência já usam.

use crate::ids;

/// A linha e o parâmetro que este id endereça, se ele for um slider da pilha de efeitos.
pub(super) fn param_of(id: ph2d_a11y::NodeId) -> Option<(usize, usize)> {
    (0..ids::MAX_FX_ROWS).find_map(|row| {
        (0..ids::MAX_FX_ROW_PARAMS)
            .find(|&p| id == ids::vector_fx_param_id(row, p))
            .map(|p| (row, p))
    })
}

/// Este id é um botão da pilha (Add / Remove / Up / Down / 👁)?
pub(super) fn is_button(id: ph2d_a11y::NodeId) -> bool {
    (0..ids::MAX_FX_KINDS).any(|k| id == ids::vector_fx_add_id(k))
        || (0..ids::MAX_FX_ROWS).any(|r| {
            id == ids::vector_fx_remove_id(r)
                || id == ids::vector_fx_up_id(r)
                || id == ids::vector_fx_down_id(r)
                || id == ids::vector_fx_hide_id(r)
                // A CAIXINHA de um parâmetro também é um botão. Ela tem id próprio desde
                // 2026-07-18: partilhar o do slider punha dois tipos de widget num id só, e um
                // slider não emite `Click` no Up.
                || (0..ids::MAX_FX_ROW_PARAMS).any(|p| id == ids::vector_fx_toggle_id(r, p))
        })
}
