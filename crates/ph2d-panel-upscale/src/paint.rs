//! Upscale panel paint.
//!
//! Per-frame logic (mirrors the other typed panels):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot — same
//!   alias the Bg Removal / Padding / Color EQ / Equalize Sizes panels
//!   use; all image-tool docked panels are mutually exclusive and
//!   reuse the slot).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot,
//!   [`paint_panel_title`], algorithm segmented group, scale
//!   slider+chip row, "Output: W × H px" readout, Cancel + Apply.
//!   Every painter is the SHARED source-of-truth from `panel_chrome` /
//!   `widget` — no panel-local widget look.
//!
//! No on-canvas RGBA preview inside the panel — mirrors the Bg Removal
//! panel, which keeps the preview thumbnail buffer in the tool crate
//! but renders no image in the panel (the live preview is the shell's
//! on-canvas overlay).

use crate::state::{self, UpscalePanelState, set_last_content_h, set_last_visible_h};
use crate::{UpscalePanel, ids};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_corner_dot, paint_panel_surface, paint_panel_title, paint_segmented_group_adaptive,
    panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, paint_button, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_upscale::params::{UpscaleAlgorithm, scale_to_slider, slider_to_scale};

/// Label column width for the slider row.
const LABEL_COL_W: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

pub(crate) fn paint(_state: &mut UpscalePanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(UpscalePanel::ID) {
        // Symmetric stale-rect cleanup so `panel_at` stops returning
        // UPS_PANEL once the tool is deactivated.
        ctx.host.store_mut().clear_panel_rect(ids::UPS_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    // Publish the rect so wheel/click dispatch can route to this panel.
    ctx.host.store_mut().set_panel_rect(ids::UPS_PANEL, rect);

    // Dark-glass surface + corner accent — identical chrome to the
    // Inspector / Bg Removal / Padding panels.
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);

    // Dock-slot drag + resize handles (shared with Inspector).
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
    // Canonical chip width = NUMBER_INPUT_MIN_W_PX (72 px); user
    // 2026-05-24 reported old 32 px (`Spacing::Xl * 2`) was too narrow.
    let chip_w = ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX;

    // Canonical panel title — reserve room for the X close button.
    let title_size = paint_panel_title(
        rect,
        "Upscale",
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    // Disjoint borrows: store + hit_index from host; scene + text_system
    // are sibling fields on ctx; theme is Copy.
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // X close button → UPS_CANCEL (routes to same handler as Cancel).
    ph2d_editor_core::widget::panel_chrome::paint_panel_close_button(
        rect,
        ids::UPS_CANCEL,
        hit_index,
        scene,
        theme,
    );
    // Color dot + notes intentionally NOT broadcast to image-tool panels.

    // ── Algorithm segmented selector ───────────────────────────────
    // 3 buttons in a row; the active one is the snapshot's algorithm.
    let segs: [(&str, bool, ph2d_a11y::NodeId); 3] = [
        (
            "Lanczos3",
            snapshot.algorithm == UpscaleAlgorithm::Lanczos3,
            ids::UPS_ALGO_LANCZOS3,
        ),
        (
            "Nearest",
            snapshot.algorithm == UpscaleAlgorithm::Nearest,
            ids::UPS_ALGO_NEAREST,
        ),
        (
            "xBR",
            snapshot.algorithm == UpscaleAlgorithm::Xbr,
            ids::UPS_ALGO_XBR,
        ),
    ];
    let seg_rect = Rect::new(inner_x, y, inner_w, row_h);
    let alg_h =
        paint_segmented_group_adaptive(seg_rect, &segs, scene, text_system, theme, hit_index);
    y += alg_h + row_gap;
    y += row_gap;

    // ── Scale slider + chip row ────────────────────────────────────
    // Widget Gallery convention §4.2: chip and slider share `0..1`
    // storage (`link_slider_number`). The chip's natural unit
    // ("2.00×") is paint-only via `display_override`, projected from
    // the live slider track — NOT consulted from the chip's stored
    // value (which lives in track space too).
    let track = store
        .slider(ids::UPS_SCALE)
        .map(|(_, v)| v)
        .unwrap_or_else(|| scale_to_slider(snapshot.scale_factor));
    let factor_display = format!("{:.2}×", slider_to_scale(track));
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(inner_x, y, inner_w, row_h),
        "Scale",
        track,
        track as f64,
        Some(&factor_display),
        ids::UPS_SCALE,
        ids::UPS_SCALE_NUM,
        LABEL_COL_W,
        chip_w,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += used + row_gap;

    // ── Reset (ghost, full width) row ──────────────────────────────
    let btn_gap = Spacing::Sm.px();
    let reset_rect = Rect::new(inner_x, y, inner_w, row_h);
    let reset_state = store
        .button_state(ids::UPS_RESET)
        .unwrap_or(ButtonState::Normal);
    let reset = Button::new(ids::UPS_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::UPS_RESET, reset_rect);
    y += row_h + row_gap;

    // ── Cancel (ghost) + Apply (accent CTA) row ────────────────────
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::UPS_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::UPS_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::UPS_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::UPS_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::UPS_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::UPS_APPLY, apply_rect);
    y += row_h;

    // Body fits without scroll; publish height as both content + visible
    // so the orchestrator's scroll clamp is a no-op.
    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);

    hit_index.register(
        ids::UPS_CANCEL,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}
