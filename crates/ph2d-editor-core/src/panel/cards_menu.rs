//! ⭐ **D21 — o menu «…» da lista** (Reset Positions · Reset Colors · Reset All) e o «+» do pé
//! repetido no cabeçalho (D25): o ícone, o clique que abre o menu e as linhas que repõem os cartões.

use super::*;

/// O id do ícone de menu da lista de `panel` (D21).
#[must_use]
pub fn list_menu_of(panel: NodeId) -> NodeId {
    const SAL_DO_MENU: u64 = 0x3E7A_C4D1_0B5F_92A6;
    NodeId(panel.0 ^ SAL_DO_MENU)
}

/// ⭐ D21: o clique no ícone de menu abre o menu do painel; uma linha dele repõe os cartões daquele
/// painel. Devolve se consumiu o evento.
pub fn apply_list_menu_click(
    store: &mut WidgetStore,
    event: crate::interaction::WidgetEvent,
) -> bool {
    use crate::interaction::{ContextMenuKind, ContextMenuRequest};
    let crate::interaction::WidgetEvent::Click(id) = event else {
        return false;
    };
    if let Some((panel, b)) = store.card_list_menu(id) {
        store.open_context_menu(ContextMenuRequest {
            x: b.x,
            y: b.y + b.h,
            kind: ContextMenuKind::CardListMenu { panel },
        });
        return true;
    }
    let (order, colors, folds) = if id == crate::ids::CTX_MENU_CARDS_RESET_ORDER {
        (true, false, false)
    } else if id == crate::ids::CTX_MENU_CARDS_RESET_COLORS {
        (false, true, false)
    } else if id == crate::ids::CTX_MENU_CARDS_RESET_ALL {
        (true, true, true)
    } else {
        return false;
    };
    if let Some(req) = store.consume_last_context_menu()
        && let ContextMenuKind::CardListMenu { panel } = req.kind
    {
        store.reset_card_list(panel, order, colors, folds);
    }
    true
}

/// O «×» de `panel`, na ponta do cabeçalho `cab`, como em toda janela, se foi declarado. Devolve a caixa
/// do ícone seguinte, que se encosta a ele.
pub(super) fn pinta_fecho(ctx: &mut PaintCtx, panel: NodeId, theme: Theme, cab: Rect) -> Rect {
    let ponta = ponta_do_fecho(cab);
    let Some(f) = ctx.host.store().card_list_close(panel) else {
        return ponta;
    };
    let store = ctx.host.store_mut();
    if store.tooltip_for(f).is_none() {
        store.set_tooltip(f, ph2d_i18n::tr("card.menu.close"));
    }
    let aceso = store.hover_live(f) > 0.0;
    ctx.host.hit_index_mut().register(f, ponta);
    if aceso {
        fill_rounded_rect(
            ctx.scene,
            ponta,
            m::HEAD_ICON_RADIUS,
            resolve(ColorToken::CardFieldHover, theme),
        );
    }
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        ctx.scene,
        IconId::Close,
        Rect::new(
            ponta.x + (ponta.w - g) * 0.5,
            ponta.y + (ponta.h - g) * 0.5,
            g,
            g,
        ),
        resolve(ColorToken::KanbanListFg, theme),
        StrokeToken::Default.px(),
    );
    vizinha_do_fecho(cab)
}

/// A caixa do «×» na ponta do cabeçalho `cab`.
pub(super) fn ponta_do_fecho(cab: Rect) -> Rect {
    Rect::new(
        cab.x + cab.w - m::LIST_PAD_X - m::HEAD_ICON_BOX,
        cab.y + (m::LIST_HEAD_H - m::HEAD_ICON_BOX) * 0.5,
        m::HEAD_ICON_BOX,
        m::HEAD_ICON_BOX,
    )
}

/// A caixa do ícone que se encosta à esquerda do «×».
pub(super) fn vizinha_do_fecho(cab: Rect) -> Rect {
    let p = ponta_do_fecho(cab);
    Rect::new(p.x - m::HEAD_ICON_BOX, p.y, p.w, p.h)
}

/// Os ícones do cabeçalho da lista `cab` (x, topo, largura, altura do cabeçalho): «…» e, se o pé o
/// pede, «+».
pub(super) fn pinta(
    ctx: &mut PaintCtx,
    panel: NodeId,
    theme: Theme,
    cab: Rect,
    footer: Option<Footer<'_>>,
) {
    let menu = list_menu_of(panel);
    let b = pinta_fecho(ctx, panel, theme, cab);
    let store = ctx.host.store_mut();
    store.register_if_absent(
        menu,
        crate::interaction::InteractiveState::Button {
            state: crate::widget::ButtonState::Normal,
        },
    );
    store.mark_card_list_menu(menu, panel, b);
    if store.tooltip_for(menu).is_none() {
        store.set_tooltip(menu, ph2d_i18n::tr("card.menu.panel"));
    }
    let aceso = store.hover_live(menu) > 0.0;
    ctx.host.hit_index_mut().register(menu, b);
    if aceso {
        fill_rounded_rect(
            ctx.scene,
            b,
            m::HEAD_ICON_RADIUS,
            resolve(ColorToken::CardFieldHover, theme),
        );
    }
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        ctx.scene,
        IconId::MoreHorizontal,
        Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
        resolve(ColorToken::KanbanListFg, theme),
        StrokeToken::Default.px(),
    );
    // ⭐ D25: o «+» do pé repete-se no cabeçalho, à esquerda do «…» — o MESMO id (um comando,
    // um id, como o olho da T21).
    if let Some(f) = footer.filter(|f| f.at_top) {
        let a = Rect::new(b.x - m::HEAD_ICON_BOX, b.y, b.w, b.h);
        let store = ctx.host.store_mut();
        if store.tooltip_for(f.id).is_none() {
            store.set_tooltip(f.id, f.text);
        }
        let aceso = store.hover_live(f.id) > 0.0;
        ctx.host.hit_index_mut().register(f.id, a);
        if aceso {
            fill_rounded_rect(
                ctx.scene,
                a,
                m::HEAD_ICON_RADIUS,
                resolve(ColorToken::CardFieldHover, theme),
            );
        }
        paint_icon(
            ctx.scene,
            IconId::Add,
            Rect::new(a.x + (a.w - g) * 0.5, a.y + (a.h - g) * 0.5, g, g),
            resolve(ColorToken::KanbanListFg, theme),
            StrokeToken::Default.px(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interaction::WidgetStore;

    /// O ícone abre o menu do painel dele.
    #[test]
    fn the_list_icon_opens_its_panel_menu() {
        let painel = NodeId(1);
        let botao = list_menu_of(painel);
        let mut s = WidgetStore::default();
        s.mark_card_list_menu(botao, painel, crate::zones::Rect::new(5.0, 6.0, 7.0, 8.0));
        assert!(apply_list_menu_click(
            &mut s,
            crate::interaction::WidgetEvent::Click(botao)
        ));
        let m = s.context_menu().expect("menu aberto");
        assert_eq!(
            m.kind,
            crate::interaction::ContextMenuKind::CardListMenu { panel: painel }
        );
    }
}
