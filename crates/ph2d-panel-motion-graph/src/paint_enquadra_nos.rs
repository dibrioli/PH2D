//! ⭐ **«Traz estes nós à vista»** — o pedido de uma vez (`request_frame_nodes`) que o balão do
//! tutorial faz quando a etapa muda: se os cartões pedidos já estão TODOS dentro do painel, nada
//! muda; senão a vista desliza com o MESMO zoom até a caixa deles ficar ao centro, e só afasta o
//! bastante para caberem. A conta é a do enquadramento ([`super::caixa`] + [`super::enquadra_entre`]).
//!
//! ⚠️ O painel muda de tamanho nos primeiros quadros (os painéis laterais chegam depois — ver
//! `refit_if_the_panel_moved_under_an_untouched_view`): enquanto a vista for a que este pedido
//! deixou (ou a que um auto-enquadramento acabou de pôr), um painel novo volta a verificá-lo. A
//! vista que o artista mexeu é dele.

use super::{GraphViewSnapshot, MotionGraphPanelState, Rect, ZOOM_FIT_MIN, caixa, enquadra_entre};
use crate::geom::{View, card_rect};
use std::collections::BTreeSet;

/// Consome o pedido pendente e cumpre-o (ou volta a cumpri-lo num painel que mudou).
pub(super) fn aplica(
    state: &mut MotionGraphPanelState,
    snap: &GraphViewSnapshot,
    rect: Rect,
    auto_fit: bool,
) {
    let ids = match crate::snapshot::take_frame_request() {
        Some(ids) => ids,
        None => match &state.nos_a_ver {
            Some((ids, painel, vista)) if *painel != rect && (auto_fit || *vista == state.view) => {
                ids.clone()
            }
            _ => return,
        },
    };
    let alvo: BTreeSet<u32> = ids.iter().copied().collect();
    let view = View::new(rect, state.view);
    let fora = snap
        .nodes
        .iter()
        .filter(|n| alvo.contains(&n.id))
        .any(|n| !dentro(card_rect(n, &view), rect));
    if fora && let Some(c) = caixa(snap, Some(&alvo), &view) {
        let zoom = state.view.zoom;
        state.view = enquadra_entre(rect, c, ZOOM_FIT_MIN.min(zoom), zoom);
    }
    state.nos_a_ver = Some((ids, rect, state.view));
}

fn dentro(r: Rect, painel: Rect) -> bool {
    r.x >= painel.x
        && r.y >= painel.y
        && r.x + r.w <= painel.x + painel.w
        && r.y + r.h <= painel.y + painel.h
}

#[cfg(test)]
#[path = "paint_enquadra_nos_tests.rs"]
mod tests;
