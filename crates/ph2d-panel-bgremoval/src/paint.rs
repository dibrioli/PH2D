//! Background-Removal panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect taken from `ctx.layout.bgremoval` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface, [`paint_panel_title`],
//!   [`paint_segmented_button`] (Mode), [`paint_slider_with_chip_layout`]
//!   (Tolerance / Feather / Refine), Cancel + Apply buttons. Every
//!   painter is the SHARED source-of-truth from `panel_chrome` /
//!   `widget` — no panel-local widget look.
//! - `content_h` / `visible_h` publish for scroll bounds.
//!
//! Wave 11 §2.2 (ADR-0042 §6 #3) — `paint()` is now a thin orchestrator
//! (≤ 80 LOC). Section-painters live in
//! [`crate::paint_sections`] and follow the y-cursor convention
//! (each helper takes `y_in: f32` and returns `y_out: f32`).

use crate::paint_sections::{
    paint_apply_cta, paint_eyedropper_swatches, paint_grow_shrink, paint_islands,
    paint_protect_brush, paint_slider_rows,
};
use crate::state::{self, BgRemovalPanelState, set_last_content_h, set_last_visible_h};
use crate::{BgRemovalPanel, ids};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_surface, paint_panel_title,
    panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

pub(crate) fn paint(_state: &mut BgRemovalPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(BgRemovalPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // BGR_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::BGR_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.bgremoval;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::BGR_PANEL, rect);

    // Dark-glass surface + corner accent.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared with Inspector — same
    // right-dock slot; persistence is shared).
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

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    // Canonical chip width — matches Widget Gallery + Inspector
    // (NUMBER_INPUT_MIN_W_PX = 72 px ≈ 7 digits + stepper). The old
    // `Spacing::Xl * 2 = 32 px` was too narrow (user 2026-05-24).
    let chip_w = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;

    // Canonical panel title — reserve room for the X close button.
    let title_size = paint_panel_title(
        rect,
        "Bg Removal",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // Disjoint borrows: store + hit_index from host; scene + text_system
    // are sibling fields on ctx; theme is a Copy.
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // X close button — routes to BGR_CANCEL (same handler as the
    // bottom Cancel button: pushes CancelActiveTool).
    paint_panel_close_button(rect, ids::BGR_CANCEL, hit_index, scene, theme);
    // Color dot + notes intentionally NOT broadcast to image-tool panels.

    y = paint_slider_rows(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        chip_w,
        y,
    );
    y = paint_grow_shrink(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        chip_w,
        y,
    );
    y = paint_islands(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        chip_w,
        y,
    );
    y = paint_eyedropper_swatches(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        y,
    );
    y = paint_protect_brush(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        chip_w,
        y,
    );
    y = paint_apply_cta(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        y,
    );

    // Body fits without scroll; publish height as both content + visible
    // so the orchestrator's scroll clamp is a no-op.
    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);

    // Re-register close at end-of-frame (canon — vide panel_chrome doc).
    hit_index.register(
        ids::BGR_CANCEL,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}
