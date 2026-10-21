//! **O chip e o popover de BLEND de uma camada do Flip** — filho cortado do `paint_layers.rs` por
//! responsabilidade quando a porta da moldura do tema (wave 4 do redesenho, 2026-09-05) o fez
//! passar o teto de LOC do painel. O pai orquestra a secção; este sabe UM dropdown.

use super::*;

/// Paint the blend chip (registered as a `Dropdown` for the generic open/close
/// dispatch) and, in the decorator column of `line`, the blend's key mark (a mistura é canal,
/// plano 35 §A.8). If open (single-open enforced), stash it into `pending` for the
/// deferred popover pass.
pub(super) fn paint_blend_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    row: &FlipLayerRow,
    line: Rect,
    pending: &mut Option<PendingBlend>,
) {
    let (chip_w, dot) = ph2d_editor_core::widget::form_row_columns(line.x, line.w, line.y, line.h);
    let rect = Rect::new(line.x, line.y, chip_w, line.h);
    if let Some(alvo) =
        ph2d_editor_core::widget::key_mark::paint_decorator_for(ctx.scene, theme, dot, &[id])
    {
        ctx.host.hit_index_mut().register(alvo, dot);
    }
    ctx.host.store_mut().register_if_absent(
        id,
        InteractiveState::Dropdown {
            state: DropdownState::Normal,
            open: false,
            selected_index: Some(row.blend as usize),
        },
    );
    let store_open = matches!(
        ctx.host.store().get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    // One popover at a time: only the first open dropdown (top→bottom) wins.
    let open = store_open && pending.is_none();
    if store_open
        && !open
        && let Some(InteractiveState::Dropdown { open: o, .. }) = ctx.host.store_mut().get_mut(id)
    {
        *o = false;
    }

    let dd_visual = ctx.host.store().dropdown_visual(id);
    let dd = Dropdown::new(id, "", blend_options(row.id))
        .selected(row.blend)
        .open(open)
        .visual(dd_visual);
    paint_dropdown_chip(&dd, rect, ctx.scene, ctx.text_system, theme);
    ctx.host.hit_index_mut().register(id, rect);
    if open {
        *pending = Some(PendingBlend {
            layer: row.id,
            chip_rect: rect,
            cur: row.blend,
        });
    }
}

/// Deferred paint of the single open blend dropdown popover (on top of the rows, clamped to the
/// viewport + scrollable — the 22 modes overflow the dock), by the foundation door.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, theme: Theme, pending: &PendingBlend) {
    let id = ids::flip_layer_widget_id(pending.layer, FlipLayerWidget::Blend);
    let dd = Dropdown::new(id, "", blend_options(pending.layer))
        .selected(pending.cur)
        .open(true);
    let viewport = ctx.viewport;
    paint_dropdown_list(ctx, &dd, pending.chip_rect, viewport, theme, None);
}

#[cfg(test)]
#[path = "blend_tests.rs"]
mod tests;
