//! Paint da tira: chrome + barra de ferramentas + as células.
//!
//! ⭐ **O rect É `ctx.slot`** — a banda do encaixe `Bottom`, e nada mais.
//!
//! ⛔ Esta nota dizia que a tira *«sobe uma altura de dock quando o timeline global está aberto,
//! porque as duas são faixas inferiores e não podem se sobrepor»*. As duas metades dissolveram: o
//! timeline e a tira declaram o **mesmo encaixe**, e dois painéis no mesmo encaixe são **abas**
//! desde a entrega 21 — nunca duas faixas ao mesmo tempo. Ver o corpo do `paint`.

use crate::state::{self, FlipStripState};
use crate::{FlipFramesPanel, ids};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_TITLE_BASELINE, paint_panel_close_button,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_close_button_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

pub(crate) fn paint(state: &mut FlipStripState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(FlipFramesPanel::ID) {
        // Limpa o rect publicado, senão o `panel_at` continua devolvendo a tira
        // depois de trocar de tool (e ela comeria cliques do canvas).
        ctx.host.store_mut().clear_panel_rect(ids::FLIP_STRIP_PANEL);
        return;
    }
    let snap = state::current_flip_strip();
    let theme = ctx.host.theme();

    // ⭐⭐ **A TIRA É A BANDA** (2026-08-31). ⛔ Ela empilhava-se ACIMA do timeline
    // (`base.y - TIMELINE_DOCK_H - grow`) e **crescia para fora** da faixa que lhe foi dada —
    // 292 px numa banda de 240, começando 52 px acima dela.
    //
    // ⚠️ **Duas coisas dissolveram por baixo dela e a nota não foi reconferida:**
    //
    // 1. **Empilhar deixou de ser o modelo.** O timeline e esta tira declaram o MESMO encaixe
    //    (`Slot::Bottom`), e desde as abas de encaixe dois painéis no mesmo sítio são **abas** —
    //    só um está à frente de cada vez, e `ctx.slot` já desconta a faixa de abas. *O `lift`
    //    resolvia uma sobreposição que já não pode acontecer.*
    // 2. **A altura da banda passou a ser AUTORADA** (a costura do topo do timeline), então o
    //    `TIMELINE_DOCK_H` que o `lift` somava era uma constante a descrever um número que o
    //    artista move.
    //
    // ⇒ o rect É `ctx.slot`. Uma faixa docada que se pinta maior do que a banda não é uma faixa
    // com mais conteúdo: é um painel por cima do desenho, que é a foto 2 da **D1**.
    let base: Rect = ctx.slot;
    // A barra QUEBRA em linhas quando a janela é estreita. ⚠️ O plano dela continua a ser medido
    // (as linhas decidem onde as células começam, mais abaixo) — o que mudou é que ele já não pode
    // fazer a tira **transbordar**: quem quiser mais espaço arrasta a costura do topo da banda.
    let bar_w = (base.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let bar_rows = crate::toolbar_plan::rows(Rect::new(0.0, 0.0, bar_w, ROW_H_PX), ROW_H_PX, &snap);
    let rect = base;

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
    // `bar` é a PRIMEIRA linha; as demais descem a partir dela, dentro da banda.
    let bar = Rect::new(inner_x, y, inner_w, ROW_H_PX);
    let pending_cycle = crate::paint_toolbar::paint(ctx, theme, bar, &snap);

    // As células, no que sobra até a base do painel.
    let cells_y = y + crate::toolbar_plan::bar_height(bar_rows, ROW_H_PX) + Spacing::Sm.px();
    let cells_h = (rect.y + rect.h - cells_y - PANEL_HEAD_PAD).max(0.0);
    crate::paint_cells::paint(
        state,
        ctx,
        theme,
        Rect::new(inner_x, cells_y, inner_w, cells_h),
        &snap,
    );

    // O popover ABERTO vai por ÚLTIMO (fora de qualquer clip, por cima de tudo). A barra
    // tem dois dropdowns, e cada um mostra a SUA seleção — o `pending` diz qual é.
    if let Some(chip) = pending_cycle {
        let cur = if chip.id == ids::FLIP_TWEEN_EASE_DD {
            snap.tween_ease
        } else {
            snap.cycle
        };
        crate::paint_toolbar::paint_cycle_popover(ctx, theme, chip, cur);
    }

    // Re-registra o X depois do corpo (o último registro vence).
    ctx.host
        .hit_index_mut()
        .register(ids::FLIP_STRIP_CLOSE, panel_close_button_rect(rect));
}
