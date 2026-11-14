//! ⭐ **Arrastar o cartão da chave pelo título** (dono 13/11: «permita arrastar o modal»).
//!
//! O Down no título arma a pega (a distância do cursor ao canto); cada Move reabre o mesmo pedido
//! de menu no sítio novo; o Up solta. O cartão é pintado a partir de `ContextMenuRequest::{x, y}`,
//! por isso mover o pedido é mover o cartão. O mesmo Down/Move/Up edita a curva **Custom**
//! (cria, arrasta e apaga pontos; arrasta alças) (o lote vai à timeline por `WidgetStore::take_key_card_bez_edit`).

use crate::interaction::state::BezAlvo;
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
    if !crate::interaction::key_card_alvo::e_cartao(req.kind) {
        return false;
    }
    let Some((id, r)) = hit else {
        return false;
    };
    if editor(store, req.kind, id, event) {
        return true;
    }
    if event.button != PointerButton::Primary || id != crate::ids::CTX_MENU_TL_KEY_CARD_HEAD {
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

/// O Down no editor da curva Custom (consome-o): num ponto ou alça agarra-o; em cima da curva
/// cria um ponto ali e agarra-o; o botão direito num ponto apaga-o.
fn editor(store: &mut WidgetStore, kind: ContextMenuKind, id: NodeId, ev: PointerEvent) -> bool {
    use crate::interaction::key_card_alvo as ed;
    let ContextMenuKind::TimelineSegment { scope } = kind else {
        return false;
    };
    // Um dos tipos de alça (Corner · Smooth · Symmetric): troca o do ponto escolhido.
    if let Some(&(_, tipo, _)) = ed::TIPOS.iter().find(|t| t.0 == id) {
        let mut curva = store.key_card_curve().unwrap_or_else(ed::linear);
        if ev.button == PointerButton::Primary
            && let Some(i) = store.key_card_bez_escolhido()
            && curva.set_kind(i, tipo)
        {
            store.apply_key_card_bez(scope, curva);
        }
        return true;
    }
    if id != crate::ids::CTX_MENU_TL_KEY_BEZ_AREA {
        return false;
    }
    let Some(area) = store.key_card_bez_area() else {
        return false;
    };
    let mut curva = store.key_card_curve().unwrap_or_else(ed::linear);
    let alvo = ed::alvo_em(&curva, &area, ev.x, ev.y);
    match ev.button {
        PointerButton::Primary => {
            if let Some(a) = alvo {
                // Alt + agarrar uma alça quebra o ponto em Corner, como no Vector.
                if let BezAlvo::Alca(i, _) = a
                    && store.alt_held()
                {
                    curva.set_kind(i, ph2d_anim::HandleKind::Corner);
                }
                store.begin_key_card_bez(scope, a, curva);
            } else if let Some(u) = ed::na_curva(&curva, &area, ev.x, ev.y)
                && let Some(i) = curva.insert(u)
            {
                store.begin_key_card_bez(scope, BezAlvo::Ponto(i), curva);
            }
        }
        PointerButton::Secondary => {
            if let Some(BezAlvo::Ponto(i)) = alvo
                && curva.remove(i)
            {
                store.set_key_card_bez_escolhido(None);
                store.apply_key_card_bez(scope, curva);
            }
        }
        _ => {}
    }
    true
}

/// O Move: com uma alça do editor agarrada, ela segue o cursor; com a pega armada, o cartão segue
/// o cursor. Consome o Move.
pub(super) fn move_(store: &mut WidgetStore, x: f32, y: f32) -> bool {
    if store.key_card_bez_alvo().is_some() {
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
