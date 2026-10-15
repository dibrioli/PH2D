//! ⭐ **«Traz estes nós à vista»** — o pedido de uma vez (`request_frame_nodes`) que o balão do
//! tutorial faz quando a etapa muda: se os cartões pedidos já estão TODOS dentro do painel, nada
//! muda; senão a vista desliza com o MESMO zoom até a caixa deles ficar ao centro, e só afasta o
//! bastante para caberem — até ao zoom mínimo da roda (`ZOOM_MIN`), não ao piso do enquadramento
//! automático: os dois sítios de uma ligação têm de estar à vista (cena `=3`: Box e Wind a `650`
//! unidades na mesma coluna, num painel de `205` px). A conta é a do enquadramento ([`super::caixa`] + [`super::enquadra_entre`]).
//!
//! ⚠️ O painel muda de tamanho nos primeiros quadros (os painéis laterais chegam depois — ver
//! `refit_if_the_panel_moved_under_an_untouched_view`): enquanto a vista for a que este pedido
//! deixou (ou a que um auto-enquadramento acabou de pôr), um painel novo volta a verificá-lo. A
//! vista que o artista mexeu é dele.

use super::{GraphViewSnapshot, MotionGraphPanelState, Rect, caixa, enquadra_entre};
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
        state.view = enquadra_entre(rect, c, crate::interact::ZOOM_MIN.min(zoom), zoom);
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
