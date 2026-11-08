//! O quadro da janela: um cartão com a lista, flutuando à esquerda da coluna.

use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::cartao;
use ph2d_editor_core::panel::cards::linhas_extra::ListRow;
use ph2d_editor_core::panel::cards::{CardDef, borda, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_pixel_core::StepState;
use ph2d_tokens::Spacing;

use crate::canal::{self, HistoryView};
use crate::{PixelHistoryPanel, event, ids};

/// O rect da janela: à esquerda da coluna, abaixo da régua, deslocável pelo título e fora da lista
/// docada da esquerda (a mesma régua da janela SLICES).
fn solta(ctx: &mut PaintCtx, piso: f32) -> Rect {
    let area = ctx.layout.draw_area;
    let livre = ctx.layout.clear_of_left_list(area);
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let fundo = borda::fundo(ctx.layout);
    // A KEYBOARD SHORTCUTS aberta ocupa o mesmo canto: a história encosta-se à esquerda dela.
    let atalhos = ctx
        .host
        .store_mut()
        .panel_rect(ph2d_editor_core::ids::PIXEL_KEYS_PANEL);
    let coluna = atalhos.map_or(ctx.slot.x, |r| r.x.min(ctx.slot.x));
    let store = ctx.host.store_mut();
    let node = PixelHistoryPanel::NODE_ID;
    let esquerda = livre.x.max(area.x);
    let w = borda::largura_com_piso(store, node, area, coluna, piso)
        .min((coluna - Spacing::Lg.px() - esquerda).max(0.0));
    let base = (coluna - Spacing::Lg.px() - w).max(esquerda);
    let (dx, dy) = store.blender_picker_offset(node);
    let x = (base + dx).clamp(esquerda, base);
    let y = (teto + dy).clamp(teto, (fundo - m::LIST_HEAD_H).max(teto));
    if (x - base, y - teto) != (dx, dy) {
        store.set_blender_picker_offset(node, x - base, y - teto);
    }
    Rect::new(x, y, w, (fundo - y).max(0.0))
}

fn cartao_lista(v: HistoryView) -> CardDef<'static> {
    cartao(
        ids::SEC_LIST,
        tr("panel.pixel.history.sec_list"),
        None,
        move |c, mut y| {
            for (i, r) in v.rows.iter().enumerate() {
                y = c.list_row(
                    &ListRow {
                        selected: i == v.current,
                        dim: r.state == StepState::Undone,
                        ..ListRow::new(ids::row(i), tr(r.label))
                    },
                    y,
                );
            }
            y
        },
    )
}

pub(crate) fn paint(ctx: &mut PaintCtx) {
    let node = PixelHistoryPanel::NODE_ID;
    let v = canal::view();
    if canal::take_abrir() {
        ctx.host.set_panel_visible(PixelHistoryPanel::ID, true);
    }
    if !ctx.host.panel_visible(PixelHistoryPanel::ID) || v.rows.is_empty() {
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    event::regista_linhas(ctx.host.store_mut(), v.rows.len());
    let titulo = canal::title(v.bytes);
    // O piso é o do conteúdo: o nome de passo mais largo inteiro (dono 08/11: «não consigo estreitar»).
    let nomes: Vec<String> = v.rows.iter().map(|r| tr(r.label).to_owned()).collect();
    let piso = borda::piso_das_linhas(ctx.text_system, nomes.iter().map(String::as_str));
    let cartoes = [cartao_lista(v)];
    let slot = solta(ctx, piso);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    borda::declara_fecho(ctx.host.store_mut(), node);
    let lista = paint_card_list(ctx, node, &titulo, &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    let reserva = borda::reserva_do_titulo();
    ctx.host.hit_index_mut().register(
        ids::DRAG,
        panel_drag_handle_rect(lista, m::LIST_HEAD_H, reserva),
    );
    borda::regista(ctx.host.store_mut(), ids::EDGE, node);
    borda::pega(ctx, ids::EDGE, lista);
}
