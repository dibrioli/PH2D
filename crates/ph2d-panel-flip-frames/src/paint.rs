//! Paint da tira: chrome + barra de ferramentas + as células.
//!
//! O rect vem do slot `layout.flip_strip` (faixa inferior). Se o **timeline
//! global** estiver aberto, a tira sobe uma altura de dock: as duas são faixas
//! inferiores e não podem se sobrepor — quem sabe da visibilidade é o paint, não o
//! layout (que é geometria pura).

use crate::state::{self, FlipStripState};
use crate::{FlipFramesPanel, ids};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::screens::layout::TIMELINE_DOCK_H;
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

pub(crate) fn paint(_state: &mut FlipStripState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(FlipFramesPanel::ID) {
        // Limpa o rect publicado, senão o `panel_at` continua devolvendo a tira
        // depois de trocar de tool (e ela comeria cliques do canvas).
        ctx.host.store_mut().clear_panel_rect(ids::FLIP_STRIP_PANEL);
        return;
    }
    let snap = state::current_flip_strip();
    let theme = ctx.host.theme();

    // A tira empilha ACIMA do timeline global quando ele está aberto.
    let base: Rect = ctx.layout.flip_strip;
    let lift = if ctx.host.panel_visible("timeline") {
        TIMELINE_DOCK_H + Spacing::Sm.px()
    } else {
        0.0
    };
    let rect = Rect::new(base.x, (base.y - lift).max(0.0), base.w, base.h);

    ctx.host
        .store_mut()
        .set_panel_rect(ids::FLIP_STRIP_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Título: "Frames" + a camada que se está animando (a tira é de UMA camada).
    let title = if snap.has_layer && !snap.layer_name.is_empty() {
        format!("Frames — {}", snap.layer_name)
    } else {
        "Frames".to_owned()
    };
    let title_size = paint_panel_title(
        rect,
        &title,
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::FLIP_STRIP_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Sm.px();

    // Barra de ferramentas (transporte / ghost / autoria / chaves / tween / ciclo).
    let bar = Rect::new(inner_x, y, inner_w, ROW_H_PX);
    let pending_cycle = crate::paint_toolbar::paint(ctx, theme, bar, &snap);

    // As células, no que sobra até a base do painel.
    let cells_y = y + ROW_H_PX + Spacing::Sm.px();
    let cells_h = (rect.y + rect.h - cells_y - PANEL_HEAD_PAD).max(0.0);
    crate::paint_cells::paint(
        ctx,
        theme,
        Rect::new(inner_x, cells_y, inner_w, cells_h),
        &snap,
    );

    // O popover do ciclo vai por ÚLTIMO (fora de qualquer clip, por cima de tudo).
    if let Some(chip) = pending_cycle {
        crate::paint_toolbar::paint_cycle_popover(ctx, theme, chip, snap.cycle);
    }

    // Re-registra o X depois do corpo (o último registro vence).
    ctx.host
        .hit_index_mut()
        .register(ids::FLIP_STRIP_CLOSE, panel_close_button_rect(rect));
}
