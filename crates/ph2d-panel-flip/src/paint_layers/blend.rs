//! **O chip e o popover de BLEND de uma camada do Flip** — filho cortado do `paint_layers.rs` por
//! responsabilidade quando a porta da moldura do tema (wave 4 do redesenho, 2026-09-05) o fez
//! passar o teto de LOC do painel. O pai orquestra a secção; este sabe UM dropdown.

use super::*;

/// Paint the blend chip (registered as a `Dropdown` for the generic open/close
/// dispatch). If open (single-open enforced), stash it into `pending` for the
/// deferred popover pass.
pub(super) fn paint_blend_chip(
    ctx: &mut PaintCtx,
    theme: Theme,
    id: ph2d_a11y::NodeId,
    row: &FlipLayerRow,
    rect: Rect,
    pending: &mut Option<PendingBlend>,
) {
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

/// Deferred paint of the single open blend dropdown popover (on top of the rows,
/// clamped to the viewport + scrollable — the 22 modes overflow the dock).
/// Registers each option as a Button + its hit rect so option clicks dispatch.
/// Thin replica of the Painter panel's `paint_dropdown_popover`.
pub(crate) fn paint_blend_popover(ctx: &mut PaintCtx, theme: Theme, pending: &PendingBlend) {
    let id = ids::flip_layer_widget_id(pending.layer, FlipLayerWidget::Blend);
    let options = blend_options(pending.layer);
    let dd = Dropdown::new(id, "", options)
        .selected(pending.cur)
        .open(true);
    let viewport = ctx.viewport;
    let panel = dd.popover_rect_clamped(pending.chip_rect, viewport);
    let content_h = dd.content_height(pending.chip_rect.h);
    let visible_h = panel.h;
    {
        let store = ctx.host.store_mut();
        store.set_dropdown_popover(id, panel);
        store.set_panel_content_h(id, content_h);
        store.set_panel_visible_h(id, visible_h);
    }
    let max_scroll = (content_h - visible_h).max(0.0);
    if ctx.host.store().panel_scroll(id) > max_scroll {
        ctx.host.store_mut().set_panel_scroll(id, max_scroll);
    }
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent
    paint_dropdown_popover_scrolled(
        &dd,
        pending.chip_rect,
        panel,
        scroll,
        ctx.host
            .store()
            .scrollbar_visual_for(DROPDOWN_SCROLLBAR_ID, Some(id)),
        ctx.scene,
        ctx.text_system,
        theme,
    );
    {
        let store = ctx.host.store_mut();
        for opt in dd.options.iter() {
            store.register_if_absent(
                opt.id,
                InteractiveState::Button {
                    state: ButtonState::Normal,
                },
            );
        }
    }
    // Register only the VISIBLE part of each option row (scrolled-out = no hit).
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(pending.chip_rect, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            ctx.host
                .hit_index_mut()
                .register(opt.id, Rect::new(r.x, top, r.w, bot - top));
        }
    }
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
