//! O quadro de uma janela: abre o que a família pediu, semeia os campos e pinta os cartões com os
//! verbos numa faixa presa, numa coluna no meio do canvas, que se arrasta pelo título.

use ph2d_editor_core::panel::cards::CardDef;
use ph2d_editor_core::panel::cards::banda::metrics as m;
use ph2d_editor_core::panel::cards::ferramenta::{
    Verbos, pinta_lista, pinta_lista_curta, prepara_numeros,
};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::panel_drag_handle_rect;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{PANEL_MIN_W_PX, Spacing};

use crate::canal::{self, Imagem, Tela};
use crate::comum::ImagemDoCartao;
use crate::{
    PixelExportPanel, PixelExportPanelState, cards_export, cards_folha, cards_import,
    cards_import_como, cards_tileset, event, ids,
};

/// A largura da janela, em larguras mínimas de painel: cabe a folha e duas colunas de campos.
const LARGURA: f32 = 1.25; // LITERAL-PX-OK: razão de larguras, não píxeis

/// Uma imagem nova do canal vira imagem da GPU; a mesma imagem não se recria.
fn imagem_do_canal(st: &mut PixelExportPanelState) -> Option<(ImagemDoCartao, Imagem)> {
    let Some(p) = canal::imagem() else {
        st.imagem = None;
        return None;
    };
    if st.imagem.as_ref().is_none_or(|(seq, _)| *seq != p.seq) {
        st.imagem =
            ph2d_vector::StableImage::from_rgba(p.rgba.clone(), p.w, p.h).map(|img| (p.seq, img));
    }
    let (_, img) = st.imagem.as_ref()?;
    Some((
        ImagemDoCartao {
            img: img.clone(),
            lados: (p.w, p.h),
            xadrez: st.xadrez.clone(),
        },
        p,
    ))
}

pub(crate) fn paint(st: &mut PixelExportPanelState, ctx: &mut PaintCtx) {
    let node = PixelExportPanel::NODE_ID;
    if !ctx.host.panel_visible(PixelExportPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(node);
        canal::pede_folha(None);
        canal::pede_tiles(None);
        return;
    }
    if let Some(t) = canal::take_abrir() {
        st.tela = t;
        if let (Tela::ImportComo, Some(p)) = (t, canal::pergunta()) {
            st.como_pixel = p.pixel_de_fabrica && p.erro.is_none();
        }
    }
    let x = canal::contexto();
    let store = ctx.host.store_mut();
    event::regista_linhas(store, &x);
    // Uma imagem nova do Import traz a grelha sugerida.
    if st.tela == Tela::Import
        && let Some(i) = canal::imagem()
        && st.grelha_de != Some(i.seq)
    {
        st.grelha_de = Some(i.seq);
        if let Some(g) = i.sugestao {
            st.grelha = g;
        }
    }
    let numeros = match st.tela {
        Tela::ExportAs => cards_export::numeros(&st.as_, &x),
        Tela::Folha => cards_folha::numeros(&st.fo, &x),
        Tela::Import => cards_import::numeros(&st.grelha),
        Tela::ImportComo => Vec::new(),
        Tela::Tileset => cards_tileset::numeros(st.tiles),
    };
    prepara_numeros(store, &numeros);
    canal::pede_folha((st.tela == Tela::Folha).then(|| st.fo.folha.clone()));
    canal::pede_tiles(if st.tela == Tela::Tileset {
        st.tiles.pedido(&x)
    } else {
        None
    });
    let (titulo, verbo, cartoes): (_, _, Vec<CardDef<'static>>) = match st.tela {
        Tela::ExportAs => {
            cards_export::prepare();
            (
                tr("panel.pixel.export.title_as"),
                tr("panel.pixel.export.export"),
                cards_export::cards(st.as_, &x),
            )
        }
        Tela::Folha => {
            cards_folha::prepare();
            let img = imagem_do_canal(st);
            (
                tr("panel.pixel.export.title_sheet"),
                tr("panel.pixel.export.export"),
                cards_folha::cards(&st.fo, &x, img),
            )
        }
        Tela::Tileset => {
            let img = imagem_do_canal(st);
            (
                tr("panel.pixel.export.title_tileset"),
                tr("panel.pixel.export.export"),
                cards_tileset::cards(st.tiles, &x, img),
            )
        }
        Tela::ImportComo => {
            let img = imagem_do_canal(st);
            let p = canal::pergunta().unwrap_or_default();
            (
                tr("panel.pixel.export.title_import_as"),
                tr("panel.pixel.export.import"),
                cards_import_como::cards(p, st.como_pixel, img),
            )
        }
        Tela::Import => {
            let img = imagem_do_canal(st);
            (
                tr("panel.pixel.export.title_import"),
                tr("panel.pixel.export.import"),
                cards_import::cards(st.grelha, img),
            )
        }
    };
    // No meio do canvas, debaixo da régua; o arrasto soma-se a isto.
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(ctx.layout.draw_area);
        r.y + r.h + Spacing::Sm.px()
    };
    let w = PANEL_MIN_W_PX * LARGURA;
    let area = ctx.layout.draw_area;
    let (dx, dy) = ctx.host.store().blender_picker_offset(node);
    let slot = Rect::new(
        area.x + (area.w - w) * 0.5 + dx,
        teto + dy,
        w,
        (area.y + area.h - teto - Spacing::Sm.px()).max(0.0),
    );
    let cancel = (ids::CANCEL, tr("panel.pixel.export.cancel"));
    let apply = (ids::OK, verbo);
    let encaixe = ctx.slot;
    ctx.slot = slot;
    let tudo = if st.tela == Tela::ImportComo {
        pinta_lista_curta(ctx, node, titulo, &cartoes, cancel, apply)
    } else {
        let verbos = Verbos {
            reset: (ids::EXPORT_RESET, tr("panel.pixel.export.reset")),
            cancel,
            apply,
        };
        pinta_lista(ctx, node, titulo, &cartoes, &verbos)
    };
    ctx.slot = encaixe;
    let reserva = (m::HEAD_ICON_BOX + m::LIST_HEAD_PAD_X) * 2.0;
    ctx.host.hit_index_mut().register(
        ids::EXPORT_DRAG,
        panel_drag_handle_rect(tudo, m::LIST_HEAD_H, reserva),
    );
}
