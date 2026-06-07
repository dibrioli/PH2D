//! Brush Studio paint — chrome + three scrollable sections (Stroke Path /
//! Shape / Rendering), mirroring the Inspector section layout and the
//! sidebar dock chrome.
//!
//! Render canon (mirror sidebar / layers dock pattern):
//! - Visibility gate via `PanelHostInternal::panel_visible`
//! - Right-dock rect from `ctx.layout.painter_sidebar` (shared slot — only one
//!   of sidebar / layers / studio is visible at a time)
//! - Chrome: dark-glass surface + corner dots + title "Brush Studio" + close X
//!   + drag/resize handles (Inspector slot shared canon)
//! - Scrollable body: sliders via `paint_slider_with_chip_layout_adaptive`,
//!   checkboxes via `paint_checkbox`, enum dials as cycling buttons; sections
//!   separated by `paint_section_separator`; scrollbar + content_h publish

use crate::BrushStudioPanel;
use crate::ids;
use crate::state::{self, BrushStudioPanelState, set_last_content_h, set_last_visible_h};
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    PAINTER_BRUSH_STUDIO_SCROLLBAR_ID, paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::Spacing;

pub(crate) fn paint(_state: &mut BrushStudioPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(BrushStudioPanel::ID) {
        ctx.host
            .store_mut()
            .clear_panel_rect(core_ids::PAINTER_BRUSH_STUDIO_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.painter_sidebar;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    ctx.host
        .store_mut()
        .set_panel_rect(core_ids::PAINTER_BRUSH_STUDIO_PANEL, rect);

    // Chrome.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(core_ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(core_ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }
    let title_size = paint_panel_title(
        rect,
        "Brush Studio",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(rect, ids::CLOSE, ctx.host.hit_index_mut(), ctx.scene, theme);

    // Body region (clipped) + scroll.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    ctx.scene.push_clip(&rect_to_vello(body_rect));

    let scroll_y = ctx
        .host
        .store()
        .panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL)
        .max(0.0);
    let x = rect.x + PANEL_HEAD_PAD;
    let w = rect.w - PANEL_HEAD_PAD * 2.0;
    let body_paint_top = body_top + Spacing::Sm.px() - scroll_y;
    let y = crate::sections::paint_sections(ctx, x, w, body_paint_top, &snapshot, theme);

    let content_h = (y - body_paint_top + PANEL_HEAD_PAD).max(0.0);
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // Visual scrollbar (self-gates when the content fits) + drag thumb id.
    let scrollbar_active = matches!(
        ctx.host.store().scrollbar_drag(),
        Some(d) if d.panel == core_ids::PAINTER_BRUSH_STUDIO_PANEL
    );
    paint_scrollbar(
        body_rect,
        scroll_y,
        content_h,
        body_h,
        scrollbar_active,
        ctx.scene,
        theme,
    );
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll_y, content_h, body_h);
        ctx.host
            .hit_index_mut()
            .register(PAINTER_BRUSH_STUDIO_SCROLLBAR_ID, thumb);
    }

    // Corner dots last so they sit visually atop any body drift.
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);
    // Re-register close after the body so scrolled widgets cannot shadow it.
    ctx.host
        .hit_index_mut()
        .register(ids::CLOSE, panel_close_button_rect(rect));

    // Publish scroll bounds + clamp right after paint (next-event correctness).
    let store = ctx.host.store_mut();
    store.set_panel_content_h(core_ids::PAINTER_BRUSH_STUDIO_PANEL, content_h);
    store.set_panel_visible_h(core_ids::PAINTER_BRUSH_STUDIO_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL) > max_scroll {
        store.set_panel_scroll(core_ids::PAINTER_BRUSH_STUDIO_PANEL, max_scroll);
    }
}
