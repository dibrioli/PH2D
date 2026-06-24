//! The panel's shared **scrollable** dropdown-popover renderer. Split from `paint_brush.rs` for the
//! panel-crate file-LOC cap. Every brush/texture/blend/adjustment menu in this panel defers to it, so
//! long option lists (texture Kind = 26, Blend = 24, Adjustment = 24…) clamp to the viewport and then
//! SCROLL — wheel over the list, or drag the bar (Enio: "menus com scroll devem ser o padrão").

use crate::paint::register_button;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::{
    DROPDOWN_SCROLLBAR_ID, Dropdown, DropdownOption, paint_dropdown_popover_scrolled,
    scrollbar_is_needed, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;

/// Deferred paint of an open dropdown popover that **scrolls** when the option list overflows the
/// (viewport-clamped) popover. Generic over the option value type so the u8-valued chips (Blend /
/// Falloff / Texture / Ramp) and the usize-valued Adjustment menu share one renderer. Publishes the
/// popover geometry so `dispatch_wheel` + the `DROPDOWN_SCROLLBAR_ID` drag can scroll it.
pub(crate) fn paint_dropdown_popover<T: Clone + PartialEq>(
    ctx: &mut PaintCtx,
    theme: ph2d_tokens::Theme,
    id: ph2d_a11y::NodeId,
    options: Vec<DropdownOption<T>>,
    chip_rect: Rect,
    cur: T,
) {
    let dd = Dropdown::new(id, "", options).selected(cur).open(true);
    let viewport = ctx.viewport;
    let panel = dd.popover_rect_clamped(chip_rect, viewport);
    let content_h = dd.content_height(chip_rect.h);
    let visible_h = panel.h;
    // Publish geometry so the wheel + scrollbar drag can scroll THIS open popover (the value lives in
    // `panel_scroll(id)`; the dropdown rect routes the wheel without polluting `panel_at`).
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
    let scroll = ctx.host.store().panel_scroll(id).clamp(0.0, max_scroll); // CLAMP-OK: 0.0 literal; max_scroll is a non-negative px extent, never NaN
    let scrollbar_active = matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == id);
    paint_dropdown_popover_scrolled(
        &dd,
        chip_rect,
        panel,
        scroll,
        scrollbar_active,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    {
        let store = ctx.host.store_mut();
        for opt in dd.options.iter() {
            register_button(store, opt.id);
        }
    }
    // Register only the VISIBLE part of each option row — scrolled-out rows must not be clickable.
    let hit_index = ctx.host.hit_index_mut();
    for (i, opt) in dd.options.iter().enumerate() {
        let r = dd.option_rect_in_scrolled(chip_rect, panel, i, scroll);
        let top = r.y.max(panel.y);
        let bot = (r.y + r.h).min(panel.y + panel.h);
        if bot - top >= 1.0 {
            hit_index.register(opt.id, Rect::new(r.x, top, r.w, bot - top));
        }
    }
    // The scrollbar track is the drag zone (grab anywhere) when the list overflows.
    if scrollbar_is_needed(content_h, visible_h) {
        ctx.host
            .hit_index_mut()
            .register(DROPDOWN_SCROLLBAR_ID, scrollbar_track_rect(panel));
    }
}
