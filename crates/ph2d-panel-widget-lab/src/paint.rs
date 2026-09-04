//! A moldura do laboratório — irmã da do Widget Gallery, e de propósito com a **mesma** forma.
//!
//! ⛔⛔ **A 1.ª redacção deste ficheiro desenhava a moldura à mão** (um rectângulo, um contorno e
//! um título) e nunca registava as alças nem a barra. Resultado, no report do Enio (2026-09-01):
//! *«o painel não tem scroll nem modo de estreitar para testar»* — uma janela que **recortava e não
//! rolava** (a pior das três formas: um painel sem recorte desenha por cima e vê-se; um que recorta
//! e rola funciona; **um que recorta e não rola esconde os controlos e não diz nada**) e que não se
//! podia mover nem redimensionar, o que numa bancada cuja pergunta central é *«aguenta estreito?»*
//! apagava metade da razão de ela existir.
//!
//! ⚠️ **Os ids das alças estavam registados no `populate` desde o primeiro dia.** Registar não
//! pinta e não indexa o hit: as três alças eram ids vivos que nenhum rectângulo alcançava. *É a
//! terceira espécie de controlo morto — nem órfão, nem sem consumidor: **sem geometria**.*
//!
//! ⇒ passa a usar o `widget::panel_chrome`, que é a porta que a galeria já usava.

use crate::WidgetLabPanel;
use crate::state::WidgetLabState;
use crate::study::{Bench, paint_study};
use ph2d_editor_core::icons::IconId;
use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{paint_icon, paint_text, rect_to_vello, resolve};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, clamp_panel_rect, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface_floating, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    LAB_SCROLLBAR_ID, SCROLLBAR_W, SliderState, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

/// Recuo do corpo. ⚠️ Igual ao `BODY_PAD` do Inspector de propósito: a régua de largura da §2 só
/// diz a verdade se a bancada gastar o mesmo cromo que o painel que ela imita.
const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: espelho do BODY_PAD do Inspector
/// Altura da faixa de título. ⚠️ **Mais baixa que o `PANEL_HEADER_H_DEFAULT` (56) de propósito** —
/// a bancada não tem subtítulo, e cada px de cabeçalho é um px que a §2 não mostra.
const HEAD_H: f32 = 34.0; // LITERAL-PX-OK: faixa de titulo do laboratorio

pub(crate) fn paint(state: &mut WidgetLabState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(WidgetLabPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::LAB_PANEL);
        return;
    }
    let base_rect = match state.rect {
        Some(r) => r,
        None => {
            let r = default_rect(ctx.layout.viewport.w, ctx.layout.viewport.h);
            state.rect = Some(r);
            r
        }
    };
    let viewport = ctx.viewport;
    let store_imm = ctx.host.store();
    let off = store_imm.blender_picker_offset(ids::LAB_PANEL);
    let resize = store_imm.panel_resize_delta(ids::LAB_PANEL);
    let (rect, c_off, c_resize) = clamp_panel_rect(base_rect, off, resize, viewport);
    let (live_t, live_drag) = {
        let (s, v) = store_imm.slider_visual(ids::LAB_LIVE_BOX);
        (v, s == SliderState::Dragging)
    };
    let scroll = store_imm.panel_scroll(ids::LAB_PANEL);
    {
        let store = ctx.host.store_mut();
        if (c_off.0 - off.0).abs() > f32::EPSILON || (c_off.1 - off.1).abs() > f32::EPSILON {
            store.set_blender_picker_offset(ids::LAB_PANEL, c_off.0, c_off.1);
        }
        if (c_resize.0 - resize.0).abs() > f32::EPSILON
            || (c_resize.1 - resize.1).abs() > f32::EPSILON
        {
            store.set_panel_resize_delta(ids::LAB_PANEL, c_resize.0, c_resize.1);
        }
        store.set_panel_rect(ids::LAB_PANEL, rect);
    }
    let theme = ctx.host.theme();
    // ⭐⭐⭐ **É AQUI que a bancada deixa de ser um estudo e vira a CUSTOMIZAÇÃO do app.** A
    // aparência escolhida é publicada no thread-local que todo pintor de linha de propriedade lê —
    // o mesmo canal do `set_text_rendering`. ⚠️ Antes do corpo, de propósito: publicar depois faria
    // as amostras desta janela usarem a aparência do quadro ANTERIOR.
    // ⭐ **A bancada é do REDESENHO, mesmo quando o app não é** (Enio, 2026-09-03: a UI nova entra
    // desligada). É aqui que ele se estuda; mostrá-la no clássico seria uma bancada a medir o que
    // ela não estuda. ⚠️ Reposta no fim, senão o painel seguinte herdava-a.
    let app_look = ph2d_editor_core::paint::ui_look();
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
    ph2d_editor_core::paint::set_slider_style(state.style);

    paint_panel_surface_floating(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    paint_text(
        ctx.text_system,
        ctx.scene,
        WidgetLabPanel::TITLE,
        rect.x + PANEL_HEAD_PAD,
        rect.y + (HEAD_H - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        rect.w,
        resolve(ColorToken::Text1, theme),
    );
    let close_size = Spacing::Xl2.px();
    let close_rect = Rect::new(
        rect.x + rect.w - PANEL_HEAD_PAD - close_size,
        rect.y + (HEAD_H - close_size) * 0.5,
        close_size,
        close_size,
    );
    paint_icon(
        ctx.scene,
        IconId::Close,
        close_rect,
        resolve(ColorToken::Text2, theme),
        ph2d_tokens::StrokeToken::Default.px(),
    );

    let content_top = rect.y + HEAD_H;
    let visible_h = (rect.h - HEAD_H).max(0.0);
    let body = Rect::new(rect.x, content_top, rect.w, visible_h);
    let inner_x = rect.x + BODY_PAD;
    let inner_w = (rect.w - BODY_PAD * 2.0 - SCROLLBAR_W - Spacing::Sm.px()).max(1.0);

    let used = {
        let (_store, hit_index) = ctx.host.store_and_hit_index_mut();
        // ⚠️ **As alças ANTES do recorte.** Elas vivem no bordo da janela, e o recorte do corpo
        // começa abaixo do título — registá-las lá dentro deixaria a de cima fora da banda.
        hit_index.register(
            ids::LAB_DRAG_HANDLE,
            panel_drag_handle_rect(rect, HEAD_H, PANEL_HEAD_PAD + close_size),
        );
        hit_index.register(ids::LAB_RESIZE_HANDLE, panel_resize_handle_rect(rect));
        hit_index.register(ids::LAB_RESIZE_HANDLE_BL, panel_resize_handle_rect_bl(rect));
        hit_index.register(ids::LAB_CLOSE, close_rect);

        hit_index.push_clip(body);
        ctx.scene.push_clip(&rect_to_vello(body));
        let mut b = Bench {
            scene: ctx.scene,
            text: ctx.text_system,
            hit: hit_index,
            theme,
            x: inner_x,
            w: inner_w,
            y: content_top - scroll + Spacing::Xs.px(),
        };
        paint_study(&mut b, state, (live_t, live_drag))
    };
    ctx.scene.pop_layer();
    ctx.host.store_and_hit_index_mut().1.pop_clip();

    // ── A barra de rolagem ─────────────────────────────────────────────────
    // ⚠️ **Fora do recorte, de propósito:** ela é cromo da janela, não conteúdo — dentro da banda
    // ela rolaria com o que está a medir.
    if scrollbar_is_needed(used, visible_h) {
        let track = scrollbar_track_rect(body);
        let thumb = scrollbar_thumb_rect(track, scroll, used, visible_h);
        let vis = ctx.host.store().scrollbar_visual(LAB_SCROLLBAR_ID);
        paint_scrollbar(body, scroll, used, visible_h, vis, ctx.scene, theme);
        ctx.host
            .store_and_hit_index_mut()
            .1
            .register(LAB_SCROLLBAR_ID, thumb);
    }

    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::LAB_PANEL, used);
    store.set_panel_visible_h(ids::LAB_PANEL, visible_h);
    let max_scroll = (used - visible_h).max(0.0);
    if store.panel_scroll(ids::LAB_PANEL) > max_scroll {
        store.set_panel_scroll(ids::LAB_PANEL, max_scroll);
    }
    // ⚠️ **Repõe a aparência do app.** A bancada força o redesenho para si, e o thread-local é
    // partilhado — sem isto, o painel pintado a seguir herdava-a e o app inteiro mudava de cara
    // por a bancada estar aberta.
    ph2d_editor_core::paint::set_ui_look(app_look);
}

/// ⭐ **A largura por omissão é a do Inspector mais a coluna da régua**, pela mesma razão do
/// `BODY_PAD`: a bancada só é honesta se nascer do tamanho do painel que ela está a redesenhar.
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    let w = 420.0_f32.min(viewport_w - 16.0); // LITERAL-PX-OK: Inspector (304) + a coluna que a §2 rotula
    let h = 760.0_f32.min(viewport_h - 16.0).max(420.0); // LITERAL-PX-OK: altura da bancada
    let x = ((viewport_w - w) * 0.5).max(8.0); // LITERAL-PX-OK: recuo do bordo
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: recuo do bordo
    Rect::new(x, y, w, h)
}
