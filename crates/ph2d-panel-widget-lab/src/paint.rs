//! A moldura do laboratório — irmã da do Widget Gallery, e de propósito com a mesma forma:
//! visibilidade, limpeza do rect obsoleto, rect por omissão, arrasto/redimensionar, publicar,
//! corpo, e o par `content_h`/`visible_h` que faz a rolagem funcionar.

use crate::WidgetLabPanel;
use crate::state::WidgetLabState;
use crate::study::{Bench, paint_study};
use ph2d_editor_core::ids;
use ph2d_editor_core::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::clamp_panel_rect;
use ph2d_editor_core::widget::{SCROLLBAR_W, SliderState};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, PANEL_RADIUS_PX, Spacing, StrokeToken, TypeToken};

/// Recuo do corpo. ⚠️ Igual ao `BODY_PAD` do Inspector de propósito: a régua de largura da §2 só
/// diz a verdade se a bancada gastar o mesmo cromo que o painel que ela imita.
const BODY_PAD: f32 = 10.0; // LITERAL-PX-OK: espelho do BODY_PAD do Inspector
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
    let scroll = ctx.host.store().panel_scroll(ids::LAB_PANEL);
    let row_h = crate::study::DENSITIES[state.density % crate::study::DENSITIES.len()].row_h_px();

    // ── A moldura ──────────────────────────────────────────────────────────
    fill_rounded_rect(
        ctx.scene,
        rect,
        PANEL_RADIUS_PX,
        resolve(ColorToken::PanelBg, theme),
    );
    stroke_rounded_rect(
        ctx.scene,
        rect,
        PANEL_RADIUS_PX,
        StrokeToken::Hairline.px(),
        resolve(ColorToken::Border, theme),
    );
    let head = Rect::new(rect.x, rect.y, rect.w, HEAD_H);
    paint_text(
        ctx.text_system,
        ctx.scene,
        WidgetLabPanel::TITLE,
        head.x + BODY_PAD,
        head.y + (HEAD_H - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        head.w,
        resolve(ColorToken::Text1, theme),
    );

    let content_top = rect.y + HEAD_H;
    let inner_x = rect.x + BODY_PAD;
    let inner_w = (rect.w - BODY_PAD * 2.0 - SCROLLBAR_W - Spacing::Sm.px()).max(1.0);

    let used = {
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        // ⚠️ **A banda de recorte é obrigatória.** Sem ela os chips da bancada continuam a ser
        // alvos depois de rolarem para fora da janela — o defeito exacto que a `line/motion_value`
        // pagou no painel de parâmetros (`HitIndex::push_clip` já existia e o painel não o chamava).
        hit_index.push_clip(Rect::new(
            rect.x,
            content_top,
            rect.w,
            (rect.h - HEAD_H).max(0.0),
        ));
        ctx.scene
            .push_clip(&ph2d_editor_core::paint::rect_to_vello(Rect::new(
                rect.x,
                content_top,
                rect.w,
                (rect.h - HEAD_H).max(0.0),
            )));
        let mut b = Bench {
            scene: ctx.scene,
            text: ctx.text_system,
            hit: hit_index,
            theme,
            x: inner_x,
            w: inner_w,
            y: content_top - scroll + Spacing::Xs.px(),
        };
        let h = paint_study(&mut b, state, row_h, (live_t, live_drag));
        let _ = store;
        h
    };
    ctx.scene.pop_layer();
    ctx.host.store_and_hit_index_mut().1.pop_clip();

    let visible_h = (rect.h - HEAD_H).max(0.0);
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::LAB_PANEL, used);
    store.set_panel_visible_h(ids::LAB_PANEL, visible_h);
    let max_scroll = (used - visible_h).max(0.0);
    if store.panel_scroll(ids::LAB_PANEL) > max_scroll {
        store.set_panel_scroll(ids::LAB_PANEL, max_scroll);
    }
}

/// ⭐ **A largura por omissão é a do Inspector**, pela mesma razão que a do `BODY_PAD`: a bancada
/// só é honesta se nascer do tamanho do painel que ela está a redesenhar.
pub fn default_rect(viewport_w: f32, viewport_h: f32) -> Rect {
    let w = 420.0_f32.min(viewport_w - 16.0); // LITERAL-PX-OK: largura da bancada (Inspector + a coluna da regua)
    let h = 760.0_f32.min(viewport_h - 16.0).max(420.0); // LITERAL-PX-OK: altura da bancada
    let x = ((viewport_w - w) * 0.5).max(8.0); // LITERAL-PX-OK: recuo do bordo
    let y = ((viewport_h - h) * 0.5).max(8.0); // LITERAL-PX-OK: recuo do bordo
    Rect::new(x, y, w, h)
}
