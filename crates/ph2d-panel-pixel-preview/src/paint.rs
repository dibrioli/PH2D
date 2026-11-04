//! O quadro da Preview — porta de visibilidade (+ limpeza do rect e da área da roda) · a janela
//! flutuante (rect próprio, arrasto pela alça, preso ao ecrã) · o título com o tocar e o `×` · a
//! imagem do quadro mostrado, nítida (vizinho mais próximo), no zoom da janela.

use crate::{PixelPreviewPanel, PixelPreviewPanelState, canal, ids};
use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{paint_text_block, rect_to_vello, resolve};
use ph2d_editor_core::panel::cards::banda::{self, metrics as m};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_ADD_RESERVE, PANEL_TITLE_BASELINE, clamp_panel_rect,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface_floating, paint_panel_title, panel_close_button_rect,
    panel_drag_handle_rect, panel_header_add_button_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{IconButtonStyle, IconGlyph, paint_icon_button};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, PANEL_MIN_W_PX, Spacing, TypeToken};

/// Os degraus de zoom (células por píxel de ecrã) — os do menu de zoom do Aseprite 1.3.
#[rustfmt::skip]
pub(crate) const ZOOMS: [f32; 14] = [0.0625, 0.125, 0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 32.0, 64.0]; // LITERAL-PX-OK: degraus de zoom do Aseprite (razões, não píxeis de UI)

pub(crate) fn paint(st: &mut PixelPreviewPanelState, ctx: &mut PaintCtx) {
    let node = PixelPreviewPanel::NODE_ID;
    if !ctx.host.panel_visible(PixelPreviewPanel::ID) {
        let store = ctx.host.store_mut();
        store.clear_panel_rect(node);
        store.remove_graph_canvas(node);
        let _ = store.take_graph_zoom(node);
        return;
    }
    // ⛔ A régua de cima do canvas pinta-se DEPOIS dos painéis: um título debaixo da faixa dela
    // fica escondido (foto da cena 17, 01/11). A janela nunca sobe acima do fundo da régua.
    let teto = {
        let r = ph2d_editor_core::ruler::top_band(ctx.layout.draw_area);
        r.y + r.h + Spacing::Sm.px()
    };
    let base = *st.rect.get_or_insert_with(|| {
        let w = PANEL_MIN_W_PX;
        Rect::new(
            (ctx.slot.x - w - Spacing::Lg.px()).max(ctx.viewport.x),
            ctx.slot.y.max(teto),
            w,
            w,
        )
    });
    let off = ctx.host.store().blender_picker_offset(node);
    let resize = ctx.host.store().panel_resize_delta(node);
    let (mut rect, mut off2, resize2) = clamp_panel_rect(base, off, resize, ctx.viewport);
    if rect.y < teto {
        off2.1 += teto - rect.y;
        rect.y = teto;
    }
    // Escreve de volta o LIMITADO: o próximo arrasto parte do que se vê (sem borracha).
    {
        let store = ctx.host.store_mut();
        if off2 != off {
            store.set_blender_picker_offset(node, off2.0, off2.1);
        }
        if resize2 != resize {
            store.set_panel_resize_delta(node, resize2.0, resize2.1);
        }
        store.set_panel_rect(node, rect);
    }
    let theme = ctx.host.theme();
    let cartoes = ph2d_editor_core::panel::cards::ferramenta::cards_on();
    let header_h = if cartoes {
        banda::pinta_pilula(ctx, rect);
        banda::pinta_titulo_da_lista(
            ctx.scene,
            ctx.text_system,
            theme,
            rect,
            PixelPreviewPanel::TITLE.tr(),
            rect.w - m::LIST_PAD_X * 2.0 - PANEL_HEADER_ADD_RESERVE,
        );
        // A imagem é o trabalho: o título não fica mais alto que o de antes (42 px, medido).
        m::LIST_HEAD_H - m::SEG_PAD
    } else {
        paint_panel_surface_floating(rect, ctx.scene, theme);
        let title_size = paint_panel_title(
            rect,
            PixelPreviewPanel::TITLE.tr(),
            PANEL_HEADER_ADD_RESERVE,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        PANEL_TITLE_BASELINE + title_size + Spacing::Sm.px()
    };
    let frame = canal::current();

    // A alça primeiro: o tocar e o `×` registam-se por cima dela (o último registo ganha).
    let alca = panel_drag_handle_rect(rect, header_h, PANEL_HEADER_ADD_RESERVE);
    ctx.host.hit_index_mut().register(ids::DRAG, alca);
    let play = panel_header_add_button_rect(rect);
    let icon = if frame.as_ref().is_some_and(|f| f.playing) {
        IconId::Stop
    } else {
        IconId::Play
    };
    if cartoes {
        // Na linha do título da lista: o `x` de sempre, o meio da altura dela.
        let no_meio = |r: Rect| Rect {
            y: rect.y + (m::LIST_HEAD_H - r.h) * 0.5,
            ..r
        };
        banda::pinta_icone_de_cabecalho(ctx, ids::PLAY, icon, no_meio(play));
        let x = no_meio(panel_close_button_rect(rect));
        banda::pinta_icone_de_cabecalho(ctx, ids::PREVIEW_CLOSE, IconId::Close, x);
    } else {
        let visual = ctx.host.store().button_visual(ids::PLAY);
        paint_icon_button(
            play,
            IconGlyph::Builtin(icon),
            IconButtonStyle::Compact,
            visual,
            ctx.scene,
            theme,
        );
        ctx.host.hit_index_mut().register(ids::PLAY, play);
        paint_panel_close_button(
            rect,
            ids::PREVIEW_CLOSE,
            ctx.host.hit_index_mut(),
            ctx.scene,
            theme,
        );
    }
    // As pegas dos dois cantos de baixo (as das outras janelas soltas): por cima do corpo.
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    let hits = ctx.host.hit_index_mut();
    hits.register(ids::RESIZE, panel_resize_handle_rect(rect));
    hits.register(ids::RESIZE_BL, panel_resize_handle_rect_bl(rect));

    let pad = PANEL_HEAD_PAD;
    let body = Rect::new(
        rect.x + pad,
        rect.y + header_h,
        (rect.w - pad * 2.0).max(0.0),
        (rect.h - header_h - pad).max(0.0),
    );
    ctx.host.store_mut().set_graph_canvas(node, body);
    let Some(f) = frame else {
        st.imagem = None;
        // Uma frase é PROSA: quebra a linha, nunca é cortada.
        let _ = paint_text_block(
            ctx.text_system,
            ctx.scene,
            tr("panel.pixel.preview.empty"),
            body.x,
            body.y,
            TypeToken::Sm.px(),
            body.w.max(1.0),
            resolve(ColorToken::Text3, theme),
        );
        return;
    };
    if st.imagem.as_ref().is_none_or(|(seq, _)| *seq != f.seq) {
        st.imagem =
            ph2d_vector::StableImage::from_rgba(f.rgba.clone(), f.w, f.h).map(|img| (f.seq, img));
    }
    #[expect(clippy::cast_precision_loss, reason = "lados de sprite ≤ 65 535")]
    let (w, h) = (f.w as f32, f.h as f32);
    let caber = (body.w / w.max(1.0)).min(body.h / h.max(1.0));
    if let Some(z) = ctx.host.store_mut().take_graph_zoom(node)
        && z.delta != 0.0
    {
        let agora = st.zoom.map_or(caber, |i| ZOOMS[i]);
        st.zoom = Some(degrau(agora, z.delta > 0.0));
    }
    let escala = st.zoom.map_or(caber, |i| ZOOMS[i]);
    st.escala = escala;
    let (dw, dh) = (w * escala, h * escala);
    let x = body.x + (body.w - dw) * 0.5;
    let y = body.y + (body.h - dh) * 0.5;
    // ⛔ O `dest` do `draw_stable_image` são os CANTOS `(x0, y0, x1, y1)`, não `(x, y, w, h)`: com
    // a largura no lugar do canto a escala saía negativa e a imagem caía fora do recorte (a
    // Preview vazia da cena 17, 01/11).
    let dest = (
        f64::from(x),
        f64::from(y),
        f64::from(x + dw),
        f64::from(y + dh),
    );
    st.destino = Some(dest);
    if let Some((_, img)) = &st.imagem {
        ctx.scene.push_clip(&rect_to_vello(body));
        // O transparente lê-se no xadrez, como no canvas (o do Aseprite: a Preview mostra o fundo).
        let to_screen = ph2d_vector::Affine::translate((f64::from(x), f64::from(y)))
            * ph2d_vector::Affine::scale(f64::from(escala));
        crate::state::desenha_xadrez(
            &mut st.xadrez,
            &mut st.xadrez_pintado,
            ctx.scene,
            (f.w, f.h),
            to_screen,
        );
        ctx.scene
            .draw_stable_image(img, dest, ph2d_vector::ImageQuality::Low);
        ctx.scene.pop_layer();
    }
}

/// O degrau de zoom seguinte (`mais`) ou anterior a partir da escala `agora`.
fn degrau(agora: f32, mais: bool) -> usize {
    let n = ZOOMS.len();
    if mais {
        ZOOMS.iter().position(|&z| z > agora).unwrap_or(n - 1)
    } else {
        ZOOMS.iter().rposition(|&z| z < agora).unwrap_or(0)
    }
}
