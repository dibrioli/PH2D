//! ⭐ **Arrastar o cartão da chave pelo título** (dono 13/11: «permita arrastar o modal»).
//!
//! O Down no título arma a pega (a distância do cursor ao canto); cada Move reabre o mesmo pedido
//! de menu no sítio novo; o Up solta. O cartão é pintado a partir de `ContextMenuRequest::{x, y}`,
//! por isso mover o pedido é mover o cartão. O mesmo Down/Move/Up arrasta as alças do editor da
//! curva **Custom** (o lote vai à timeline por `WidgetStore::take_key_card_bez_edit`).

use crate::interaction::{ContextMenuKind, ContextMenuRequest, WidgetStore};
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
    if alca(store, req.kind, id, (event.x, event.y)) {
        return true;
    }
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

/// O Down no editor da curva Custom: numa alça agarra-a; na área agarra a mais perto e leva-a ao
/// cursor. Consome o Down.
fn alca(store: &mut WidgetStore, kind: ContextMenuKind, id: NodeId, (x, y): (f32, f32)) -> bool {
    use crate::ids as c;
    let ContextMenuKind::TimelineSegment { scope } = kind else {
        return false;
    };
    let Some(area) = store.key_card_bez_area() else {
        return false;
    };
    let h = store
        .key_card_bezier()
        .unwrap_or(crate::screens::hero::key_card::BEZ_LINEAR);
    let qual = match id {
        _ if id == c::CTX_MENU_TL_KEY_BEZ_P1 => 0,
        _ if id == c::CTX_MENU_TL_KEY_BEZ_P2 => 1,
        _ if id == c::CTX_MENU_TL_KEY_BEZ_AREA => {
            let d = |i: usize| {
                let (px, py) = area.tela(h[i * 2], h[i * 2 + 1]);
                (px - x).powi(2) + (py - y).powi(2)
            };
            usize::from(d(1) < d(0))
        }
        _ => return false,
    };
    store.begin_key_card_bez(scope, qual, h);
    if id == c::CTX_MENU_TL_KEY_BEZ_AREA {
        store.move_key_card_bez(area.unidade(x, y));
    }
    true
}

/// O Move: com uma alça do editor agarrada, ela segue o cursor; com a pega armada, o cartão segue
/// o cursor. Consome o Move.
pub(super) fn move_(store: &mut WidgetStore, x: f32, y: f32) -> bool {
    if store.key_card_bez_alca().is_some() {
        if let Some(area) = store.key_card_bez_area() {
            store.move_key_card_bez(area.unidade(x, y));
        }
        return true;
    }
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
