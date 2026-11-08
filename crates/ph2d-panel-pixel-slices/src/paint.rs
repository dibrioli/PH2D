//! O quadro do painel: os cartões, flutuando ao lado do painel PIXEL, e o menu do botão direito.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids::PIXEL_TILES_PANEL;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::borda;
use ph2d_editor_core::panel::cards::paint_card_list;
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::widget::{Dropdown, DropdownOption};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_registry::hash_node_id;

use crate::canal::{self, Menu};
use crate::{PixelSlicesPanel, PixelSlicesPanelState, cards, event, ids};

/// A alça da borda de fora (`borda`): o id mora aqui para não tocar os ids dos cartões.
pub(crate) const SLICES_EDGE: NodeId = hash_node_id("pixel_slices.edge");

/// O rect da janela: à esquerda da coluna; se a janela Tilemap está pintada ali (o rect PUBLICADO dela,
/// não a suposição de que está aberta), à esquerda dela. O arrasto do título desloca-a, mas ela fica
/// dentro da área e fora da lista da esquerda (`clear_of_left_list`, a mesma régua das abas): a lista
/// docada nunca a tapa nem lhe rouba o ponteiro. O desvio limitado volta ao store (sem borracha).
fn solta(ctx: &mut PaintCtx) -> Rect {
    let area = ctx.layout.draw_area;
    let livre = ctx.layout.clear_of_left_list(area);
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let fundo = area.y + area.h - Spacing::Sm.px();
    let coluna = ctx.slot.x;
    let store = ctx.host.store_mut();
    let mut w = borda::largura(store, PixelSlicesPanel::NODE_ID, area, coluna);
    let mut direita = coluna - Spacing::Lg.px();
    if let Some(t) = store.panel_rect(PIXEL_TILES_PANEL)
        && t.x < direita
        && direita - w - Spacing::Lg.px() < t.x + t.w
    {
        direita = t.x - Spacing::Lg.px();
    }
    let esquerda = livre.x.max(area.x);
    w = w.min((coluna - Spacing::Lg.px() - esquerda).max(0.0));
    let base = (direita - w).max(esquerda);
    let (dx, dy) = store.blender_picker_offset(PixelSlicesPanel::NODE_ID);
    let x = (base + dx).clamp(esquerda, (coluna - Spacing::Lg.px() - w).max(esquerda));
    let y = (teto + dy).clamp(teto, (fundo - m::LIST_HEAD_H).max(teto));
    if (x - base, y - teto) != (dx, dy) {
        store.set_blender_picker_offset(PixelSlicesPanel::NODE_ID, x - base, y - teto);
    }
    Rect::new(x, y, w, (fundo - y).max(0.0))
}

/// As linhas do menu, com os nomes do Aseprite e o extra do dono.
fn menu_aberto(st: &PixelSlicesPanelState, ctx: &mut PaintCtx) {
    let aberto = matches!(
        ctx.host.store().get(ids::MENU),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let Some(Menu { x, y, remover }) = st.menu.filter(|_| aberto) else {
        return;
    };
    let linha = |id, chave| DropdownOption::new(id, id, tr(chave));
    let dd = Dropdown::new(
        ids::MENU,
        "",
        vec![
            linha(ids::MENU_PROPERTIES, "panel.pixel.slices.menu_properties"),
            linha(ids::MENU_DUPLICATE, "panel.pixel.slices.menu_duplicate"),
            linha(ids::MENU_DELETE, "panel.pixel.slices.menu_delete"),
            linha(ids::MENU_REMOVE_KEY, "panel.pixel.slices.menu_remove_key").disabled(!remover),
        ],
    )
    .open(true);
    let (viewport, theme) = (ctx.viewport, ctx.host.theme());
    ph2d_editor_core::panel::paint_dropdown_list(
        ctx,
        &dd,
        // A lista mede as linhas pela «caixa» dela: uma caixa invisível, logo acima do ponteiro.
        Rect::new(x, y - ROW_H_PX, 0.0, ROW_H_PX),
        viewport,
        theme,
        None,
    );
}

pub(crate) fn paint(st: &mut PixelSlicesPanelState, ctx: &mut PaintCtx) {
    let node = PixelSlicesPanel::NODE_ID;
    if !ctx.host.panel_visible(PixelSlicesPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    let x = canal::contexto();
    if !x.na_mao {
        canal::publish_menu_aberto(false);
        ctx.host.store_mut().clear_panel_rect(node);
        event::sem_nada(ctx.host);
        return;
    }
    if let Some(novo) = canal::take_menu() {
        st.menu = Some(novo);
        if let Some(InteractiveState::Dropdown { open, .. }) =
            ctx.host.store_mut().get_mut(ids::MENU)
        {
            *open = true;
        }
    }
    let store = ctx.host.store_mut();
    // `Esc` limpou a seleção: o menu ficaria a oferecer verbos sem alvo.
    let fechar = canal::take_fechar_menu() || x.vista.is_none();
    if let Some(InteractiveState::Dropdown { open, .. }) = store.get_mut(ids::MENU) {
        *open &= !fechar;
        canal::publish_menu_aberto(*open);
    }
    if canal::take_mostrar() {
        store.set_collapsed(ids::SEC_PROPS, false);
        // É o último cartão: a rolagem vai ao fim, e a lista corta o que passar. O alvo tem folga de
        // uma janela porque o cartão que abriu só conta na altura do quadro seguinte.
        let alvo =
            store.panel_content_h(node).unwrap_or(0.0) + store.panel_visible_h(node).unwrap_or(0.0);
        store.set_panel_scroll(node, alvo);
    }
    event::regista_linhas(store, x.linhas.len());
    event::semeia(store, &x);
    event::cor(st, store, &x);
    if let Some((_, u)) = &x.user {
        let ud = ids::user_data();
        user_data::prepare(
            ctx.host.store_mut(),
            &ud,
            &crate::user_data::vista(u),
            false,
        );
    }
    let cartoes = cards::cartoes(&x, st.user_aberto);
    let slot = solta(ctx);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let lista = paint_card_list(ctx, node, PixelSlicesPanel::TITLE.tr(), &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    ctx.host.hit_index_mut().register(
        ids::SLICES_DRAG,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    borda::regista(ctx.host.store_mut(), SLICES_EDGE, node);
    borda::pega(ctx, SLICES_EDGE, lista);
    menu_aberto(st, ctx);
}
