//! O corpo do painel — **uma row por entrada da tabela gerada**.
//!
//! ⚠️ **Não há um `match` sobre os doze tipos aqui.** Cada row é pintada por
//! [`paint_widget_skin_with`], que é a MESMA porta que a prévia do canvas percorre: a prévia é
//! literalmente esta chamada sem estado. Um segundo despacho seria a segunda resposta a *"que
//! aparência tem um Slider?"*, e a divergência entre as duas só apareceria numa screenshot.
//!
//! ⚠️ **O rótulo não é pintado à parte.** Todo widget do catálogo já carrega o dele
//! (`Button::new(id, label)`), então uma coluna de rótulo desenhada por este painel escreveria o
//! nome DUAS vezes — e no `Divider`, que não tem rótulo, escreveria um nome sobre uma linha.
//!
//! # A doca é ao LADO, e não por cima
//!
//! O interruptor que abre este painel mora na seção **Frame** do painel Vector. Se ele tomasse a
//! mesma vaga, abrir cobriria o abridor — e o artista perderia de vista exactamente os controles
//! com que está a montar a moldura. É a doca do painel de Wet Tuning, pelo mesmo motivo.

use ph2d_editor_core::ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    clamp_panel_rect, paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    AUTHORED_SCROLLBAR_ID, SkinParam, icon_glyph, paint_scrollbar, paint_widget_skin_with,
    scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};

use crate::AuthoredPanel;
use crate::rows::rows;
use crate::state::AuthoredPanelState;

pub(crate) fn paint(_state: &mut AuthoredPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(AuthoredPanel::ID) {
        // Limpeza simétrica do rect: sem ela o `panel_at` continua a devolver AUTHORED_PANEL
        // depois de fechado, e a roda do rato roda um painel que não está na tela.
        ctx.host.store_mut().clear_panel_rect(ids::AUTHORED_PANEL);
        return;
    }

    let insp: Rect = ctx.layout.inspector;
    let gap = Spacing::Xs.px();
    let base = Rect::new((insp.x - insp.w - gap).max(0.0), insp.y, insp.w, insp.h);
    let off = ctx.host.store().blender_picker_offset(ids::AUTHORED_PANEL);
    let resize = ctx.host.store().panel_resize_delta(ids::AUTHORED_PANEL);
    let (rect, clamped_off, clamped_resize) = clamp_panel_rect(base, off, resize, ctx.viewport);
    // Escreve de volta o valor LIMITADO (o contrato do hero-paint): o próximo begin-de-arrasto tem
    // de capturar o deslocamento visível, e não um cru acumulado — senão o painel dá borracha
    // quando o arrasto inverte de direção.
    if clamped_off != off {
        ctx.host.store_mut().set_blender_picker_offset(
            ids::AUTHORED_PANEL,
            clamped_off.0,
            clamped_off.1,
        );
    }
    if clamped_resize != resize {
        ctx.host.store_mut().set_panel_resize_delta(
            ids::AUTHORED_PANEL,
            clamped_resize.0,
            clamped_resize.1,
        );
    }

    let theme = ctx.host.theme();
    ctx.host
        .store_mut()
        .set_panel_rect(ids::AUTHORED_PANEL, rect);

    paint_panel_surface(rect, ctx.scene, theme);
    // ⚠️ O título é o `Name` da moldura, e não uma chave de i18n: ele é dado AUTORADO. Passá-lo
    // por `tr()` procuraria uma chave que não existe e devolveria a própria string, dando a
    // impressão de que este painel é traduzível quando ele é desenhado.
    let title_size = paint_panel_title(
        rect,
        crate::generated::PANEL_TITLE,
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::AUTHORED_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::AUTHORED_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let x = rect.x + PANEL_HEAD_PAD;
    let w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let y_after = paint_body(ctx, theme, x, w, body_top - scroll);
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);

    // Punhos de chrome — registados DEPOIS do corpo de propósito: o hit é
    // último-registado-ganha, e as rows guardam o retângulo delas mesmo rolando para debaixo do
    // título (o registro não é recortado). A faixa MOVE o painel e BLINDA o cabeçalho.
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    let hit_index = ctx.host.hit_index_mut();
    hit_index.register(
        ids::AUTHORED_DRAG_HANDLE,
        panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE),
    );
    hit_index.register(ids::AUTHORED_RESIZE_HANDLE, panel_resize_handle_rect(rect));
    hit_index.register(
        ids::AUTHORED_RESIZE_HANDLE_BL,
        panel_resize_handle_rect_bl(rect),
    );
}

/// **UM cabeçalho manda nas rows ATÉ o próximo** — o colapso de seção (plano UI/UX, §6.3 item 2).
///
/// ⚠️ **O cabeçalho é sempre pintado, dobrado ou não**: ele é a alça de volta. E as rows dobradas
/// nem pintam **nem avançam o `y`** — é isso que faz a seção *encolher* em vez de deixar um vão, e
/// é também o que mantém a altura honesta: o `content_h` sai do MESMO caminho (`y_after`), então o
/// scrollbar não pode descrever um corpo que não foi desenhado.
///
/// ⚠️ **O estado do colapso é o do app, e não um segundo** (`WidgetStore::is_collapsed`, alternado
/// pelo despacho genérico via `mark_collapsible_section`). Um mapa próprio aqui daria um painel
/// gerado que dobra por regras diferentes das dos vinte e três painéis escritos à mão.
fn paint_body(ctx: &mut PaintCtx, theme: Theme, x: f32, w: f32, mut y: f32) -> f32 {
    let mut folded = false;
    for row in rows() {
        if row.folds_a_section() {
            folded = ctx.host.store().is_collapsed(row.id);
        } else if folded {
            continue;
        }
        let r = Rect::new(x, y, w, ROW_H_PX);
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        // ⚠️ Quem RESPONDE ou quem DOBRA ganha retângulo de hit — a pergunta é feita UMA vez, ao
        // `wants_pointer`. Um `Divider` registado seria um controle que acende sob o rato e não
        // faz nada; um cabeçalho SEM registo seria um chevron desenhado que não dobra.
        if row.wants_pointer() {
            hit_index.register(row.id, r);
        }
        paint_widget_skin_with(
            row.kind,
            row.label,
            row.id,
            store.get(row.id),
            SkinParam {
                rgba: row.rgba,
                // ⚠️ A precedência sai da porta única do catálogo — a MESMA que o canvas
                // percorre. Um `if` aqui seria a terceira cópia da regra.
                icon: icon_glyph(row.icon_id, row.icon.as_ref()),
            },
            r,
            scene,
            text_system,
            theme,
        );
        y += ROW_H_PX + Spacing::Xs.px();
    }
    y
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == ids::AUTHORED_PANEL
        );
        paint_scrollbar(body, scroll, content_h, body_h, is_active, ctx.scene, theme);
        ctx.host
            .hit_index_mut()
            .register(AUTHORED_SCROLLBAR_ID, thumb);
    }
    // ⚠️ Publicado SEMPRE, e não só quando a barra é pintada: a roda do rato lê estes dois números
    // para saber quanto pode rolar, e um painel que só os publica quando transborda ficaria com o
    // último valor de quando transbordava.
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::AUTHORED_PANEL, content_h);
    store.set_panel_visible_h(ids::AUTHORED_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::AUTHORED_PANEL) > max_scroll {
        store.set_panel_scroll(ids::AUTHORED_PANEL, max_scroll);
    }
}
