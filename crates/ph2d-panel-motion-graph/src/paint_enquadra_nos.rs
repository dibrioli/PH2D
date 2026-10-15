//! ⭐ **«Traz estes nós à vista»** — o pedido de uma vez (`request_frame_nodes`) que o balão do
//! tutorial faz quando a etapa muda: se os cartões pedidos já estão TODOS dentro do painel, nada
//! muda; senão a vista desliza com o MESMO zoom até a caixa deles ficar ao centro, e só afasta o
//! bastante para caberem, nunca abaixo do piso LEGÍVEL do enquadramento (`ZOOM_FIT_MIN`). Se ao
//! piso não cabem, centra o PRIMEIRO (a porta de onde se arrasta). A conta é a do enquadramento
//! ([`super::caixa`] + [`super::enquadra_entre`]).
//!
//! ⚠️ O painel muda de tamanho nos primeiros quadros (os painéis laterais chegam depois — ver
//! `refit_if_the_panel_moved_under_an_untouched_view`): enquanto a vista for a que este pedido
//! deixou (ou a que um auto-enquadramento acabou de pôr), um painel novo volta a verificá-lo. A
//! vista que o artista mexeu é dele.

use super::{
    FIT_PAD, GraphViewSnapshot, MotionGraphPanelState, Rect, ZOOM_FIT_MIN, caixa, enquadra_entre,
};
use crate::geom::{View, card_rect};
use std::collections::BTreeSet;

/// Consome o pedido pendente e cumpre-o (ou volta a cumpri-lo num painel que mudou).
pub(super) fn aplica(
    state: &mut MotionGraphPanelState,
    snap: &GraphViewSnapshot,
    rect: Rect,
    auto_fit: bool,
) {
    if let Some(ids) = crate::snapshot::take_frame_request() {
        state.nos_a_ver = Some((ids, None));
    }
    let Some((ids, feito)) = &state.nos_a_ver else {
        return;
    };
    let refaz = match feito {
        None => true,
        Some((painel, vista, _)) => *painel != rect && (auto_fit || *vista == state.view),
    };
    let alvo: BTreeSet<u32> = ids.iter().copied().collect();
    // Os nós ainda não estão no retrato (o 1.º quadro): o pedido espera por eles.
    if !refaz || !snap.nodes.iter().any(|n| alvo.contains(&n.id)) {
        return;
    }
    let view = View::new(rect, state.view);
    // Um painel que mudou debaixo da vista que o pedido MEXEU volta a centrar (o `refit_…` do grafo).
    let fora = feito.is_some_and(|(_, _, mexeu)| mexeu)
        || snap
            .nodes
            .iter()
            .filter(|n| alvo.contains(&n.id))
            .any(|n| !dentro(card_rect(n, &view), rect));
    let mut mexeu = false;
    if fora && let Some(c) = caixa(snap, Some(&alvo), &view) {
        let zoom = state.view.zoom;
        let piso = ZOOM_FIT_MIN.min(zoom);
        let mut v = enquadra_entre(rect, c, piso, zoom);
        // O zoom novo pode mudar o desenho (cartão ↔ pílula): mede-se no regime em que se desenha.
        let regime = View::new(rect, v);
        let c = caixa(snap, Some(&alvo), &regime).unwrap_or(c);
        v = enquadra_entre(rect, c, piso, v.zoom);
        let cabe = (c[2] - c[0]) * v.zoom <= rect.w - 2.0 * FIT_PAD
            && (c[3] - c[1]) * v.zoom <= rect.h - 2.0 * FIT_PAD;
        let primeiro = ids.first().map(|i| BTreeSet::from([*i]));
        state.view = match primeiro.and_then(|p| caixa(snap, Some(&p), &regime)) {
            Some(c1) if !cabe => enquadra_entre(rect, c1, v.zoom, v.zoom),
            _ => v,
        };
        mexeu = true;
    }
    if let Some((_, feito)) = &mut state.nos_a_ver {
        *feito = Some((rect, state.view, mexeu));
    }
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
