//! Padding panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot,
//!   [`paint_panel_title`], four labeled slider+chip rows (Top / Right /
//!   Bottom / Left) via [`paint_slider_with_chip_layout`], a pivot-mode
//!   toggle, then Cancel + Apply buttons. Every painter is the SHARED
//!   source-of-truth from `panel_chrome` / `widget` — no panel-local
//!   widget look.
//!
//! Each edge row pairs a bipolar slider (track `0.5` = 0 px) with a
//! px-valued chip. The slider track reads the live stored value (so a
//! drag is smooth), falling back to the per-frame snapshot; the chip
//! DISPLAYS the live stored px value (kept in sync by [`crate::seam`]),
//! falling back to the snapshot. [`crate::seam`] keeps the two widgets
//! in lock-step so dragging the slider moves the chip and typing in the
//! chip moves the slider — both in real time.
//!
//! No on-canvas live preview in v1 — Apply bakes the resized canvas
//! shell-side.

use crate::state::{self, PaddingPanelState, set_last_content_h, set_last_visible_h};
use crate::{PaddingPanel, ids};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_close_button, paint_panel_corner_dot, paint_panel_corner_dot_bl,
    paint_panel_surface, paint_panel_title, panel_drag_handle_rect, panel_resize_handle_rect,
    panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, PADDING_SCROLLBAR_ID, paint_button, paint_scrollbar,
    paint_slider_with_chip_layout_adaptive, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_padding::params::px_to_slider;

/// Label column width for slider rows.
const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

pub(crate) fn paint(_state: &mut PaddingPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(PaddingPanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // PAD_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::PAD_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::PAD_PANEL, rect);

    // Dark-glass surface + corner accents — identical chrome to the
    // Inspector / Bg Removal panels. BL gripper paintado pra dar
    // affordance ao BL resize handle (Enio 2026-05-26).
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles. Reuse Inspector IDs because
    // image-tool panels share the right dock slot — the resize delta
    // persists when the user switches between Inspector / image tool.
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
    // Canonical chip width = NUMBER_INPUT_MIN_W_PX (72 px) — was 32
    // (`Spacing::Xl * 2`); user 2026-05-24: too narrow.
    let chip_w = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;

    // Canonical panel title — reserve room on the right for the X
    // close button (UI canon post-2026-05-24).
    let title_size = paint_panel_title(
        rect,
        "Padding",
        PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );

    // Canonical X close button (painted before the clip so it sits on
    // the chrome, not inside the scrollable body).
    paint_panel_close_button(
        rect,
        ids::PAD_CANCEL,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );
    // Color dot + notes intentionally NOT broadcast to image-tool
    // panels (user 2026-05-24).

    // Body region (everything below the title, above the corner dot) is
    // clipped + scrolled — Enio 2026-05-26 "padrão central do app é
    // painel com scroll. corrija todos". Wheel + scrollbar route through
    // `PADDING_SCROLLBAR_ID` → `PAD_PANEL`.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::PAD_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = paint_body_sections(
        ctx,
        &snapshot,
        inner_x,
        inner_w,
        row_h,
        row_gap,
        chip_w,
        body_top - scroll,
    );
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);

    ctx.host.hit_index_mut().register(
        ids::PAD_CANCEL,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}

#[allow(clippy::too_many_arguments)]
fn paint_body_sections(
    ctx: &mut PaintCtx,
    snapshot: &ph2d_tool_padding::params::PaddingUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    chip_w: f32,
    y_in: f32,
) -> f32 {
    let theme = ctx.host.theme();
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut y = y_in;

    // ── Four signed per-edge slider+chip rows ──────────────────────
    // Slider track = live stored value (smooth drag) ?? normalized
    // snapshot; chip shows the live stored px ?? snapshot px. Positive =
    // transparent expand, negative = crop.
    for (label, slider_id, chip_id, snap_px) in [
        ("Top", ids::PAD_TOP, ids::PAD_TOP_NUM, snapshot.top),
        ("Right", ids::PAD_RIGHT, ids::PAD_RIGHT_NUM, snapshot.right),
        (
            "Bottom",
            ids::PAD_BOTTOM,
            ids::PAD_BOTTOM_NUM,
            snapshot.bottom,
        ),
        ("Left", ids::PAD_LEFT, ids::PAD_LEFT_NUM, snapshot.left),
    ] {
        let track = store
            .slider(slider_id)
            .map(|(_, v)| v)
            .unwrap_or_else(|| px_to_slider(snap_px));
        let px = store
            .number_value(chip_id)
            .unwrap_or(snap_px as f64)
            .round() as i64;
        let px_display = px.to_string();
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            label,
            track,
            px as f64,
            Some(&px_display),
            slider_id,
            chip_id,
            LABEL_COL_W,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + row_gap;
    }

    y += row_gap;

    // ── Pivot-mode toggle ──────────────────────────────────────────
    // Accent (pressed) = Recenter: the bake recalculates the sprite
    // translation so the original content stays world-fixed. Ghost
    // (idle) = Keep: the translation is left unchanged (canvas resizes
    // around the current pivot). Snapshot-driven look, like Bg Removal
    // Show-Mask toggle.
    let pivot_on = snapshot.recenter_pivot;
    let pivot_state = if pivot_on {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::PAD_PIVOT_RECENTER)
            .unwrap_or(ButtonState::Normal)
    };
    let pivot_kind = if pivot_on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let pivot_label = if pivot_on {
        "Pivot: Recenter"
    } else {
        "Pivot: Keep"
    };
    let pivot_rect = Rect::new(inner_x, y, inner_w, row_h);
    let pivot = Button::new(ids::PAD_PIVOT_RECENTER, pivot_label)
        .kind(pivot_kind)
        .state(pivot_state);
    paint_button(&pivot, pivot_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_PIVOT_RECENTER, pivot_rect);
    y += row_h + row_gap;

    y += row_gap;

    // ── Reset (ghost, full width) row ──────────────────────────────
    let btn_gap = Spacing::Sm.px();
    let reset_rect = Rect::new(inner_x, y, inner_w, row_h);
    let reset_state = store
        .button_state(ids::PAD_RESET)
        .unwrap_or(ButtonState::Normal);
    let reset = Button::new(ids::PAD_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_RESET, reset_rect);
    y += row_h + row_gap;

    // ── Cancel (ghost) + Apply (accent CTA) row ────────────────────
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::PAD_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::PAD_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::PAD_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::PAD_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::PAD_APPLY, apply_rect);
    y += row_h;
    y
}

fn paint_scrollbar_and_publish(
    ctx: &mut PaintCtx,
    body_rect: Rect,
    content_h: f32,
    body_h: f32,
    scroll: f32,
    theme: Theme,
) {
    if scrollbar_is_needed(content_h, body_h) {
        let track = scrollbar_track_rect(body_rect);
        let thumb = scrollbar_thumb_rect(track, scroll, content_h, body_h);
        let is_active = matches!(
            ctx.host.store().scrollbar_drag(),
            Some(d) if d.panel == ids::PAD_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(PADDING_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::PAD_PANEL, content_h);
    store.set_panel_visible_h(ids::PAD_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::PAD_PANEL) > max_scroll {
        store.set_panel_scroll(ids::PAD_PANEL, max_scroll);
    }
}
