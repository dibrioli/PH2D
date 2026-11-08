//! O quadro do painel: a janela de criar ou converter, ou os cartões do tileset.

use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::{pinta_lista_curta, prepara_numeros};
use ph2d_editor_core::panel::cards::user_data;
use ph2d_editor_core::panel::cards::{borda, paint_card_list};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{PANEL_MIN_W_PX, Spacing};

use crate::canal::{self, Contexto, Tela};
use crate::{PixelTilesPanel, PixelTilesPanelState, cards, event, ids};

/// Recria só a imagem do tile que mudou (um traço não refaz as outras).
fn imagens(st: &mut PixelTilesPanelState, x: &Contexto) {
    if std::sync::Arc::ptr_eq(&st.fontes, &x.imagens) {
        return;
    }
    let lados = (u32::from(x.tile_w), u32::from(x.tile_h));
    let velhas = std::mem::take(&mut st.imagens);
    st.imagens = x
        .imagens
        .iter()
        .enumerate()
        .map(|(i, px)| {
            let igual = st.fontes.get(i) == Some(px);
            match velhas.get(i) {
                Some(v) if igual => v.clone(),
                _ => ph2d_vector::StableImage::from_rgba(
                    std::sync::Arc::new(px.clone()),
                    lados.0,
                    lados.1,
                ),
            }
        })
        .collect();
    st.fontes = x.imagens.clone();
}

/// O rect de uma janela solta, na área do desenho; o arrasto soma-se a ele. `x_de` recebe a área e a
/// largura.
fn solta(ctx: &PaintCtx, w: f32, x_de: impl FnOnce(Rect, f32) -> f32) -> Rect {
    let area = ctx.layout.draw_area;
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(area);
        r.y + r.h + Spacing::Sm.px()
    };
    let (dx, dy) = ctx
        .host
        .store()
        .blender_picker_offset(PixelTilesPanel::NODE_ID);
    Rect::new(
        x_de(area, w) + dx,
        teto + dy,
        w,
        (area.y + area.h - teto - Spacing::Sm.px()).max(0.0),
    )
}

/// A faixa do título arrasta a janela.
fn alca(ctx: &mut PaintCtx, tudo: Rect) {
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    ctx.host.hit_index_mut().register(
        ids::TILES_DRAG,
        panel_drag_handle_rect(tudo, m::LIST_HEAD_H, reserva),
    );
}

fn janela(st: &PixelTilesPanelState, x: &Contexto, ctx: &mut PaintCtx) {
    prepara_numeros(ctx.host.store_mut(), &cards::numeros(st));
    event::regista_conjuntos(ctx.host.store_mut(), x.conjuntos.len());
    let (titulo, verbo) = if st.tela == Tela::Converter {
        (
            tr("panel.pixel.tiles.title_convert"),
            tr("panel.pixel.tiles.convert"),
        )
    } else {
        (
            tr("panel.pixel.tiles.title_new"),
            tr("panel.pixel.tiles.create"),
        )
    };
    let slot = solta(ctx, PANEL_MIN_W_PX, |a, w| a.x + (a.w - w) * 0.5);
    let cartoes = cards::janela(st, x);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let cancel = (ids::TILES_CANCEL, tr("panel.pixel.tiles.cancel"));
    let node = PixelTilesPanel::NODE_ID;
    let tudo = pinta_lista_curta(ctx, node, titulo, &cartoes, cancel, (ids::TILES_OK, verbo));
    ctx.slot = encaixe;
    alca(ctx, tudo);
}

pub(crate) fn paint(st: &mut PixelTilesPanelState, ctx: &mut PaintCtx) {
    let node = PixelTilesPanel::NODE_ID;
    if !ctx.host.panel_visible(PixelTilesPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(node);
        return;
    }
    let x = canal::contexto();
    if let Some(t) = canal::take_abrir() {
        st.tela = t;
        st.conjunto = x.padrao;
    }
    if st.tela != Tela::Tileset {
        janela(st, &x, ctx);
        return;
    }
    if x.mao.is_none() {
        ctx.host.store_mut().clear_panel_rect(node);
        event::sem_nada(ctx.host);
        return;
    }
    imagens(st, &x);
    event::regista_tiles(ctx.host.store_mut(), x.imagens.len());
    event::regista_terrenos(ctx.host.store_mut(), x.terrenos.len());
    if let Some(e) = x
        .user
        .as_ref()
        .and_then(|v| user_data::frame(ctx.host.store_mut(), &ids::user_data(), v))
    {
        canal::pede(canal::Pedido::UserData(e));
    }
    let cartoes = cards::tileset(st, &x);
    // Ao lado do encaixe do painel PIXEL, como a Preview: a área do desenho passa por baixo da coluna.
    let coluna = ctx.slot.x;
    let area = ctx.layout.draw_area;
    let w = borda::largura(ctx.host.store_mut(), node, area, coluna);
    let slot = solta(ctx, w, |a, w| (coluna - w - Spacing::Lg.px()).max(a.x));
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let lista = paint_card_list(ctx, node, PixelTilesPanel::TITLE.tr(), &cartoes);
    ctx.slot = encaixe;
    ctx.host.store_mut().set_panel_rect(node, lista);
    alca(ctx, lista);
    borda::pega(ctx, ids::TILES_EDGE, lista);
}
