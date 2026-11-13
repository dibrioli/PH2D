//! ⭐ **A Timeline nos cartões arrasta-se e redimensiona-se** (dono 10/11): o título arrasta (o
//! `DragHandle` dos painéis flutuantes), as quatro bordas e os quatro cantos redimensionam (o
//! `ResizeEdge` do painel, com o cursor de sempre), e o «encaixar» ao lado do «×» devolve-a ao lugar de
//! origem. O lugar é um desvio sobre o encaixe do layout (`WidgetStore::place_floating_card`).
//!
//! ⚠️ **A borda de CIMA na origem continua a ser a costura da banda** (`resize::apply_resize`): ela
//! muda a altura do encaixe e o cartão fica encaixado. Qualquer outra borda, ou o topo fora da
//! origem, redimensiona o cartão.

use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::interaction::{
    GesturePhase, InteractiveState, TIMELINE_EDGE_B, TIMELINE_EDGE_L, TIMELINE_EDGE_R,
    TIMELINE_EDGE_T, TimelineGesture, TimelineHitKind, WidgetStore,
};
use ph2d_editor_core::paint::{fill_rounded_rect, paint_icon, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::banda::{Moldura, geometria, metrics as m};
use ph2d_editor_core::screens::layout::HeroLayout;
use ph2d_editor_core::widget::panel_chrome::PANEL_RESIZE_HANDLE_SIZE;
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, StrokeToken};

use crate::geom::{GRIP, MIN_H, MIN_W};
use crate::ids;
use crate::state::{ResizeDrag, TimelinePanelState};

const PAINEL: ph2d_a11y::NodeId = ph2d_editor_core::ids::TIMELINE_PANEL;

/// O piso do cartão: o da Timeline.
pub(crate) const MINIMO: (f32, f32) = (MIN_W, MIN_H);

/// O rect do cartão neste quadro: o encaixe `docked` mais o desvio, preso na janela abaixo do cromo
/// do topo (barra e fila de ferramentas: por baixo dele o «×» e o «encaixar» ficariam sem volta).
pub(crate) fn lugar(store: &mut WidgetStore, docked: Rect, layout: &HeroLayout) -> Rect {
    let v = layout.viewport;
    let topo = layout.draw_area.y.clamp(v.y, v.y + v.h); // CLAMP-OK: viewport bounds, min<=max
    let janela = Rect::new(v.x, topo, v.w, (v.y + v.h - topo).max(0.0));
    store.place_floating_card(PAINEL, docked, janela, MINIMO)
}

/// As oito alças do cartão como `(id, bordas, rect)`: as bordas primeiro, os cantos por cima.
pub(crate) fn alcas(rect: Rect) -> [(ph2d_a11y::NodeId, u8, Rect); 8] {
    let (c, g) = (PANEL_RESIZE_HANDLE_SIZE, GRIP);
    let (dir, fundo) = (rect.x + rect.w, rect.y + rect.h);
    [
        (
            ids::TIMELINE_RESIZE_T,
            TIMELINE_EDGE_T,
            Rect::new(rect.x, rect.y, rect.w, g),
        ),
        (
            ids::TIMELINE_RESIZE_B,
            TIMELINE_EDGE_B,
            Rect::new(rect.x, fundo - g, rect.w, g),
        ),
        (
            ids::TIMELINE_RESIZE_L,
            TIMELINE_EDGE_L,
            Rect::new(rect.x, rect.y, g, rect.h),
        ),
        (
            ids::TIMELINE_RESIZE_R,
            TIMELINE_EDGE_R,
            Rect::new(dir - g, rect.y, g, rect.h),
        ),
        (
            ids::TIMELINE_RESIZE_TL,
            TIMELINE_EDGE_T | TIMELINE_EDGE_L,
            Rect::new(rect.x, rect.y, c, c),
        ),
        (
            ids::TIMELINE_RESIZE_TR,
            TIMELINE_EDGE_T | TIMELINE_EDGE_R,
            Rect::new(dir - c, rect.y, c, c),
        ),
        (
            ids::TIMELINE_RESIZE_BL,
            TIMELINE_EDGE_B | TIMELINE_EDGE_L,
            Rect::new(rect.x, fundo - c, c, c),
        ),
        (
            ids::TIMELINE_RESIZE_BR,
            TIMELINE_EDGE_B | TIMELINE_EDGE_R,
            Rect::new(dir - c, fundo - c, c, c),
        ),
    ]
}

/// A faixa do título que arrasta: a fileira do cabeçalho até ao «encaixar» (os controlos da barra
/// registam-se DEPOIS e ganham-lhe onde a cobrem).
pub(crate) fn faixa_do_titulo(rect: Rect) -> Rect {
    let g = geometria(crate::abas_laterais::sem_a_calha(rect));
    let fim = g.vizinha_do_fecho.x;
    Rect::new(
        g.lista.x,
        g.lista.y,
        (fim - g.lista.x).max(0.0),
        m::LIST_HEAD_H,
    )
}

/// Regista a faixa do título e as alças, por esta ordem (a alça ganha à faixa onde se tocam).
pub(crate) fn registra_alcas(ctx: &mut PaintCtx, rect: Rect) {
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CARD_DRAG, faixa_do_titulo(rect));
    for (id, edges, r) in alcas(rect) {
        ctx.host.store_mut().register(
            id,
            InteractiveState::TimelineSurface {
                parent: PAINEL,
                kind: TimelineHitKind::ResizeEdge { edges },
                canvas: rect,
            },
        );
        ctx.host.hit_index_mut().register(id, r);
    }
}

/// O «encaixar» na caixa encostada ao «×»: apagado na origem (o clique não faz nada lá).
pub(crate) fn pinta_encaixe(ctx: &mut PaintCtx, mol: &Moldura) {
    let r = mol.vizinha_do_fecho;
    let theme = ctx.host.theme();
    let fora = ctx.host.store().floating_card_away(PAINEL);
    let store = ctx.host.store_mut();
    if store.tooltip_for(ids::TIMELINE_CARD_DOCK).is_none() {
        store.set_tooltip(
            ids::TIMELINE_CARD_DOCK,
            ph2d_i18n::tr("panel.timeline.dock"),
        );
    }
    if fora && store.hover_live(ids::TIMELINE_CARD_DOCK) > 0.0 {
        fill_rounded_rect(
            ctx.scene,
            r,
            m::HEAD_ICON_RADIUS,
            resolve(ColorToken::CardFieldHover, theme),
        );
    }
    ctx.host
        .hit_index_mut()
        .register(ids::TIMELINE_CARD_DOCK, r);
    let tinta = if fora {
        ColorToken::KanbanListFg
    } else {
        ColorToken::Text3
    };
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        ctx.scene,
        IconId::AlignBottom,
        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
        resolve(tinta, theme),
        StrokeToken::Default.px(),
    );
}

/// O clique no «encaixar»: fora da origem, o cartão volta a ela.
pub(crate) fn encaixa(store: &mut WidgetStore) {
    if store.floating_card_away(PAINEL) {
        store.dock_floating_card(PAINEL);
    }
}

/// O arrasto de uma borda nos cartões. `false` = é a costura da banda (o topo na origem): quem
/// chama segue pela altura do encaixe.
pub(crate) fn apply_resize(
    state: &mut TimelinePanelState,
    store: &mut WidgetStore,
    rect: Rect,
    edges: u8,
    g: TimelineGesture,
) -> bool {
    match g.phase {
        GesturePhase::Begin => {
            if edges == TIMELINE_EDGE_T && !store.floating_card_away(PAINEL) {
                return false;
            }
            state.resize = Some(ResizeDrag {
                start_h: rect.h,
                start_y: g.y,
                start_x: g.x,
                cartao: Some((store.floating_card_offsets(PAINEL), (rect.w, rect.h))),
            });
            true
        }
        GesturePhase::Update => {
            let Some(ResizeDrag {
                start_x,
                start_y,
                cartao: Some((base, size0)),
                ..
            }) = state.resize
            else {
                return false;
            };
            let d = (g.x - start_x, g.y - start_y);
            store.resize_floating_card(PAINEL, base, size0, edges, d, MINIMO);
            true
        }
        _ => {
            let era = state.resize.is_some_and(|d| d.cartao.is_some());
            if era {
                state.resize = None;
                store.mark_floating_card_settled();
            }
            era
        }
    }
}

#[cfg(test)]
#[path = "cartao_solto_tests.rs"]
mod tests;
