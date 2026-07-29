//! **A lei de MISTURA de um degrau de filtro** — irmão do `paint_filters` pelo teto de 600 LOC do
//! painel.
//!
//! O corte é por RESPONSABILIDADE: aqui vive *como um degrau se mistura com o que está embaixo* (o
//! chip de modo + o popover das opções), enquanto o irmão fica com *as rows do próprio degrau* e o
//! `paint_filters_ramp` com *o trilho da rampa*. As três perguntas são distintas, e o teto de LOC só
//! forçou a que já era a linha de corte natural.

use super::*;
use crate::ids;
use crate::state::filters as fst;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_popover_scrolled,
    scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;

/// **O popover das vinte leis de mistura** de um degrau — pintado no passe DIFERIDO de `paint.rs`,
/// POR CIMA de todas as seções. Espelho exato do `paint_markers::paint_marker_popover`.
///
/// ⚠️ As opções saem da tabela PUBLICADA (`blend_names`), nunca de uma lista escrita aqui: o painel
/// não alcança o `BlendMode`, e uma cópia derivaria dele na primeira lei nova — com o rótulo a
/// nomear outra coisa.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, row: usize, chip: Rect, theme: Theme) {
    let id = ids::filter_blend_id(row);
    let names = fst::blend_names();
    if names.is_empty() {
        return;
    }
    let sel = fst::stack()
        .get(row)
        .map_or(0, |fx| usize::from(fx.blend))
        .min(names.len() - 1);
    let options: Vec<DropdownOption<usize>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| DropdownOption::new(ids::filter_blend_option_id(row, i), i, *n))
        .collect();
    let dd = Dropdown::new(id, "", options).selected(sel).open(true);

    let panel = dd.popover_rect_clamped(chip, ctx.viewport);
    let content_h = dd.content_height(chip.h);
    let visible_h = panel.h;
    let max_scroll = (content_h - visible_h).max(0.0);
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
        if store.panel_scroll(id) > max_scroll {
            store.set_panel_scroll(id, max_scroll);
        }
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        chip,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Hit-register só a parte VISÍVEL de cada linha (a barra de rolagem é o alvo do drag).
    let hit_index = ctx.host.hit_index_mut();
    for i in 0..names.len() {
        let r = dd.option_rect_in_scrolled(chip, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(
                ids::filter_blend_option_id(row, i),
                Rect::new(r.x, top, r.w, bot - top),
            );
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
