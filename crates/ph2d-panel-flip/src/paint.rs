//! Flip Style panel paint (mirror of the Vector Style panel).
//!
//! Per-frame: visibility gate + stale-rect cleanup on hide; right-dock rect from
//! `ctx.layout.inspector`; canonical chrome (surface + corner dots + drag/resize
//! handles + title + X close); then the scrollable body — Mode / Brush / Color /
//! Erase (fixed widgets, via `BodyCtx`) then the Layers section (dynamic
//! per-row widgets, `ctx`-based). The single open blend dropdown popover is
//! painted OUTSIDE the body clip so it isn't cut off. Finally the Stroke swatch
//! is marked a picker swatch so its Down opens the shared OKLCH picker.

use crate::paint_layers::{self, LayerMetrics, PendingBlend};
use crate::paint_sections::BodyCtx;
use crate::state::{self, FlipPanelState, set_last_content_h, set_last_visible_h};
use crate::{FlipPanel, ids};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_close_button_rect, panel_drag_handle_rect,
    panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    FLIP_SCROLLBAR_ID, NUMBER_INPUT_MIN_W_PX, SCROLLBAR_W, paint_scrollbar, scrollbar_is_needed,
    scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

pub(crate) fn paint(_state: &mut FlipPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(FlipPanel::ID) {
        // Stale-rect cleanup so `panel_at` stops returning FLIP_PANEL once the
        // tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::FLIP_PANEL);
        set_last_content_h(0.0);
        set_last_visible_h(0.0);
        return;
    }

    let rect: Rect = ctx.layout.inspector;
    let theme = ctx.host.theme();
    let snap = state::current_style();
    let layers = state::current_layers();

    ctx.host.store_mut().set_panel_rect(ids::FLIP_PANEL, rect);

    // Dark-glass surface + corner accents — identical chrome to the Inspector.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared Inspector ids — the right dock
    // slot is shared, so the resize delta persists across tool switches).
    {
        let drag_rect =
            panel_drag_handle_rect(rect, PANEL_HEADER_H_DEFAULT, PANEL_HEADER_CLOSE_RESERVE);
        let resize_rect = panel_resize_handle_rect(rect);
        let resize_bl_rect = panel_resize_handle_rect_bl(rect);
        let hit_index = ctx.host.hit_index_mut();
        hit_index.register(ph2d_editor_core::ids::INSP_DRAG_HANDLE, drag_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE, resize_rect);
        hit_index.register(ph2d_editor_core::ids::INSP_RESIZE_HANDLE_BL, resize_bl_rect);
    }

    let title_size = paint_panel_title(
        rect,
        "Flip",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    paint_panel_close_button(
        rect,
        ids::FLIP_CLOSE,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let scrollbar_reserve = SCROLLBAR_W + Spacing::Sm.px();
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0 - scrollbar_reserve).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let chip_w = NUMBER_INPUT_MIN_W_PX;
    let font = TypeToken::Base.px();

    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);

    // Clip the body region + shift content up by `scroll_y`.
    let scroll_y = ctx.host.store().panel_scroll(ids::FLIP_PANEL).max(0.0);
    let clip = rect_to_vello(Rect::new(rect.x, body_top, rect.w, body_h));
    ctx.scene.push_clip(&clip);
    let body_top_y = body_top - scroll_y;
    let mut y = body_top_y;
    let mut pending_blend: Option<PendingBlend> = None;

    // ── Phase A: fixed sections via the `BodyCtx` bundle (immutable store). ──
    {
        let scene = &mut *ctx.scene;
        let text_system = &mut *ctx.text_system;
        let (store, hit_index) = ctx.host.store_and_hit_index_mut();
        let mut b = BodyCtx {
            scene,
            text_system,
            store,
            hit_index,
            theme,
            inner_x,
            inner_w,
            row_h,
            row_gap,
            chip_w,
            font,
        };
        y = b.mode_row(&snap, y);
        y = b.shape_row(&snap, y);
        y = b.brush(&snap, y);
        y = b.color(&snap, y);
        y = b.erase_row(&snap, y);
        y = b.fill_section(&snap, y);
        y = b.reshape_section(&snap, y);
        y = b.edit_section(&snap, y);
    }

    // ── Phase B: Layers section (dynamic per-row widgets → mutable store). ──
    let m = LayerMetrics {
        inner_x,
        inner_w,
        row_h,
        row_gap,
        font,
    };
    y = paint_layers::layers_section(ctx, theme, &m, &layers, y, &mut pending_blend);

    // Total painted height (independent of scroll — both ends shift with it).
    let content_h = (y - body_top_y + PANEL_HEAD_PAD).max(0.0);

    // Body scrollbar (inside the clip, right rail) + its drag hit target.
    if scrollbar_is_needed(content_h, body_h) {
        let body = Rect::new(rect.x, body_top, rect.w, body_h);
        let thumb = scrollbar_thumb_rect(scrollbar_track_rect(body), scroll_y, content_h, body_h);
        let is_active =
            matches!(ctx.host.store().scrollbar_drag(), Some(d) if d.panel == ids::FLIP_PANEL);
        paint_scrollbar(
            body, scroll_y, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host.hit_index_mut().register(FLIP_SCROLLBAR_ID, thumb);
    }
    ctx.scene.pop_layer();

    // Deferred blend popover — OUTSIDE the body clip so it isn't cut off.
    if let Some(p) = &pending_blend {
        paint_layers::paint_blend_popover(ctx, theme, p);
    }

    // Mark the Stroke swatch so a Down opens the shared OKLCH picker + publish
    // the body heights so wheel/thumb scroll works.
    {
        let store = ctx.host.store_mut();
        store.register_picker_swatch(ids::FLIP_STROKE_SWATCH);
        store.register_picker_swatch(ids::FLIP_FILL_SWATCH);
        store.set_panel_content_h(ids::FLIP_PANEL, content_h);
        store.set_panel_visible_h(ids::FLIP_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(ids::FLIP_PANEL) > max_scroll {
            store.set_panel_scroll(ids::FLIP_PANEL, max_scroll);
        }
    }
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    // Re-register close chrome after the body (last-registered-wins).
    ctx.host
        .hit_index_mut()
        .register(ids::FLIP_CLOSE, panel_close_button_rect(rect));
}
