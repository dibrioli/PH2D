//! ⭐ **Arrastar o cartão da chave pelo título** (dono 13/11: «permita arrastar o modal»).
//!
//! O Down no título arma a pega (a distância do cursor ao canto); cada Move reabre o mesmo pedido
//! de menu no sítio novo; o Up solta. O cartão é pintado a partir de `ContextMenuRequest::{x, y}`,
//! por isso mover o pedido é mover o cartão.

use crate::interaction::{ContextMenuRequest, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_host::{PointerButton, PointerEvent};

/// O Down: se caiu no título de um cartão da chave aberto, arma o arrasto e consome o Down.
pub(super) fn pega(
    store: &mut WidgetStore,
    hit: Option<(NodeId, Rect)>,
    event: PointerEvent,
) -> bool {
    let Some(req) = store.context_menu() else {
        return false;
    };
    if event.button != PointerButton::Primary || !crate::screens::hero::key_card::e_cartao(req.kind)
    {
        return false;
    }
    let Some((id, r)) = hit else {
        return false;
    };
    if id != crate::ids::CTX_MENU_TL_KEY_CARD_HEAD {
        return false;
    }
    // O título começa no canto do cartão PINTADO (já encostado à tela): o pedido passa a ser ele.
    store.open_context_menu(ContextMenuRequest {
        x: r.x,
        y: r.y,
        ..req
    });
    store.set_key_card_grab(Some((event.x - r.x, event.y - r.y)));
    true
}

/// O Move: com a pega armada, o cartão segue o cursor (e consome o Move).
pub(super) fn move_(store: &mut WidgetStore, x: f32, y: f32) -> bool {
    let (Some((dx, dy)), Some(req)) = (store.key_card_grab(), store.context_menu()) else {
        return false;
    };
    store.open_context_menu(ContextMenuRequest {
        x: x - dx,
        y: y - dy,
        ..req
    });
    true
}
