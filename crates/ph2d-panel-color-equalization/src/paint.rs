//! Color Equalization panel paint.
//!
//! Per-frame logic (mirrors `ph2d-panel-padding`):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot — reused
//!   for any Image-Tools docked panel, which are all mutually exclusive).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot, panel title.
//!
//! After the panel title, the body paints (in order, via the helpers
//! in [`crate::paint_sections`]): a Phase 2 histogram overlay strip
//! (R/G/B bars from the live preview, drawn with the `Danger` /
//! `Success` / `Info` semantic tokens so the channel colours follow
//! the active theme); thirteen labeled slider + chip rows for the
//! Phase 1 stages (clip, tile grid, exposure, temperature, tint,
//! brightness, contrast, vibrance, saturation) and Phase 2 effects
//! (sharpen amount, sharpen radius) and Phase 3 LUT intensity/mix;
//! LUT-slot dropdowns; Posterize + Quantize dropdowns; a 2×2 auto-*
//! toggle grid; and the Reset / Cancel + Apply CTA rows.

use crate::paint_histogram::paint_histogram_overlay;
use crate::paint_sections::{
    SectionLayout, lut_options_for_slot, paint_apply_cta_section, paint_auto_buttons_section,
    paint_lut_section, paint_posterize_quantize_section, paint_slider_rows_section,
    posterize_options, quantize_options,
};
use crate::state::{self, set_last_content_h, set_last_visible_h};
use crate::{ColorEqualizationPanel, ColorEqualizationPanelState, ids};
use ph2d_editor_core::paint::rect_to_vello;
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_HEADER_CLOSE_RESERVE, PANEL_HEADER_H_DEFAULT, PANEL_TITLE_BASELINE,
    paint_panel_corner_dot, paint_panel_corner_dot_bl, paint_panel_surface, paint_panel_title,
    panel_drag_handle_rect, panel_resize_handle_rect, panel_resize_handle_rect_bl,
};
use ph2d_editor_core::widget::{
    COLOR_EQUALIZATION_SCROLLBAR_ID, Dropdown, DropdownState, paint_dropdown_popover_in_viewport,
    paint_scrollbar, scrollbar_is_needed, scrollbar_thumb_rect, scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, Theme};
use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot;

/// Label column width for slider rows.
const LABEL_COL_W: f32 = 84.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

/// Height of the histogram overlay strip.
const HISTOGRAM_H: f32 = 64.0; // LITERAL-PX-OK: panel grid metric (histogram strip height)

pub(crate) fn paint(_state: &mut ColorEqualizationPanelState, ctx: &mut PaintCtx) {
    if !ctx.host.panel_visible(ColorEqualizationPanel::ID) {
        ctx.host.store_mut().clear_panel_rect(ids::CEQ_PANEL);
        return;
    }

    let rect: Rect = ctx.layout.padding;
    let theme = ctx.host.theme();
    let snapshot = state::current_snapshot();

    ctx.host.store_mut().set_panel_rect(ids::CEQ_PANEL, rect);
    paint_panel_surface(rect, ctx.scene, theme);
    paint_panel_corner_dot(rect, ctx.scene, theme);
    // BL resize gripper dot — sem isso o usuário não vê affordance
    // do BL handle (que já é hit-registrado abaixo). Enio 2026-05-26.
    paint_panel_corner_dot_bl(rect, ctx.scene, theme);

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

    let layout = SectionLayout {
        inner_x: rect.x + PANEL_HEAD_PAD,
        inner_w: (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0),
        row_h: ROW_H_PX,
        row_gap: Spacing::Sm.px(),
        // Canonical chip width — 72 px (was 32, user 2026-05-24).
        chip_w: ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX,
        label_col_w: LABEL_COL_W,
    };

    // Título curto pra caber em 1 linha mesmo com painel estreito
    // (Enio 2026-05-26: "nome do painel fica cortado abaixo do
    // monitor. Corrija isso.").
    let title_size = paint_panel_title(
        rect,
        "Color EQ",
        ph2d_editor_core::widget::panel_chrome::PANEL_HEADER_CLOSE_RESERVE,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    // X close button → CEQ_CANCEL (same handler as bottom Cancel).
    ph2d_editor_core::widget::panel_chrome::paint_panel_close_button(
        rect,
        ids::CEQ_CANCEL,
        ctx.host.hit_index_mut(),
        ctx.scene,
        theme,
    );
    // Color dot + notes intentionally NOT broadcast to image-tool panels.
    // Body region (everything below the title, above the corner dot) is
    // clipped + scrolled. CEQ grew past the dock height once Phase 2/3
    // landed (~810 px against a ~600 px panel). Wheel + scrollbar route
    // through `COLOR_EQUALIZATION_SCROLLBAR_ID` → `CEQ_PANEL` so the
    // bottom controls stay reachable.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::CEQ_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let y_after = paint_body_sections(ctx, &snapshot, layout, theme, body_top - scroll);

    // `content_h` is the painted body height in body-local coords (undo
    // the scroll subtraction we applied when laying out from `body_top
    // - scroll`). The scrollbar uses this against `body_h` to decide
    // whether it's needed and how tall the thumb is.
    let content_h = (y_after + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);
    ctx.scene.pop_layer();

    // Painted AFTER `pop_layer` so the option list floats over every
    // later section / panel.
    paint_pending_popovers(ctx);
    paint_scrollbar_and_publish(ctx, body_rect, content_h, body_h, scroll, theme);

    // Re-register close at end-of-frame so scrolled body widgets
    // behind the title can't shadow it (canon — vide panel_chrome doc).
    ctx.host.hit_index_mut().register(
        ids::CEQ_CANCEL,
        ph2d_editor_core::widget::panel_chrome::panel_close_button_rect(rect),
    );
}

/// Paint every body section inside the active scroll clip. Returns the
/// final `y` after the Apply CTA row. Takes `&mut PaintCtx` because the
/// section helpers need shared `&WidgetStore` + `&mut HitIndex` borrows
/// that must be held across the whole cascade (one destructure here, no
/// borrow gymnastics for the caller).
fn paint_body_sections(
    ctx: &mut PaintCtx,
    snapshot: &ColorEqualizationUiSnapshot,
    layout: SectionLayout,
    theme: Theme,
    y_in: f32,
) -> f32 {
    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut y = y_in;

    // ── Phase 2 histogram overlay ──────────────────────────────────
    let hist_rect = Rect::new(layout.inner_x, y, layout.inner_w, HISTOGRAM_H);
    state::with_current_histogram(|h| {
        paint_histogram_overlay(hist_rect, h, scene, theme);
    });
    y += HISTOGRAM_H + layout.row_gap;

    // ── 13 slider+chip rows ────────────────────────────────────────
    y = paint_slider_rows_section(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        snapshot,
        layout,
        y,
    );
    y += layout.row_gap;

    // ── Phase 3 LUT grouped-select dropdowns ──────────────────────
    y = paint_lut_section(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        snapshot,
        layout,
        y,
    );

    // ── Phase 5 Posterize + Quantize dropdowns ────────────────────
    y = paint_posterize_quantize_section(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        snapshot,
        layout,
        y,
    );

    // ── Auto-* toggle 2×2 grid ─────────────────────────────────────
    y = paint_auto_buttons_section(
        scene,
        text_system,
        store,
        hit_index,
        theme,
        snapshot,
        layout,
        y,
    );

    // ── Reset + Cancel/Apply CTA rows ──────────────────────────────
    paint_apply_cta_section(scene, text_system, store, hit_index, theme, layout, y)
}

/// Paint the vertical scrollbar (only if needed) and publish
/// `content_h` / `visible_h` to the store so wheel dispatch can bound
/// the offset against painter-known metrics (avoids a 1-frame jump
/// when wheeling past the end). Also clamps any over-scroll left over
/// from a content shrink in the previous frame.
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
            Some(d) if d.panel == ids::CEQ_PANEL
        );
        paint_scrollbar(
            body_rect, scroll, content_h, body_h, is_active, ctx.scene, theme,
        );
        ctx.host
            .hit_index_mut()
            .register(COLOR_EQUALIZATION_SCROLLBAR_ID, thumb);
    }
    let store = ctx.host.store_mut();
    store.set_panel_content_h(ids::CEQ_PANEL, content_h);
    store.set_panel_visible_h(ids::CEQ_PANEL, body_h);
    let max_scroll = (content_h - body_h).max(0.0);
    if store.panel_scroll(ids::CEQ_PANEL) > max_scroll {
        store.set_panel_scroll(ids::CEQ_PANEL, max_scroll);
    }
}

/// Drain `state::take_pending_popovers()` and paint each open chip's
/// option list on top of the panel chrome. The viewport rect is taken
/// from `ctx.viewport` so the popover flips above the chip when it'd
/// overflow below — important because the Posterize / Quantize chips
/// sit near the bottom of the panel, where a 16-row LUT list (or even
/// an 8-row Quantize list) extends past the screen bottom.
fn paint_pending_popovers(ctx: &mut PaintCtx) {
    let pending = state::take_pending_popovers();
    let snapshot_for_popover = state::current_snapshot();
    let theme = ctx.host.theme();
    let viewport = ctx.viewport;
    for p in pending {
        match p.slot {
            1 | 2 => {
                let (chip_id, label, selected) = if p.slot == 1 {
                    (
                        ids::CEQ_LUT_1_DROPDOWN,
                        "LUT 1",
                        snapshot_for_popover.lut_preset_1,
                    )
                } else {
                    (
                        ids::CEQ_LUT_2_DROPDOWN,
                        "LUT 2",
                        snapshot_for_popover.lut_preset_2,
                    )
                };
                let dd = Dropdown::new(chip_id, label.to_string(), lut_options_for_slot(p.slot))
                    .selected(selected)
                    .state(DropdownState::Focused)
                    .open(true);
                paint_dropdown_popover_in_viewport(
                    &dd,
                    p.chip,
                    Some(viewport),
                    ctx.scene,
                    ctx.text_system,
                    theme,
                );
                let panel_rect = dd.popover_rect_clamped(p.chip, viewport);
                for (i, opt) in dd.options.iter().enumerate() {
                    ctx.host
                        .hit_index_mut()
                        .register(opt.id, dd.option_rect_in(p.chip, panel_rect, i));
                }
            }
            3 => {
                let dd = Dropdown::new(
                    ids::CEQ_POSTERIZE_DROPDOWN,
                    "Posterize".to_string(),
                    posterize_options(),
                )
                .selected(snapshot_for_popover.posterize_levels)
                .state(DropdownState::Focused)
                .open(true);
                paint_dropdown_popover_in_viewport(
                    &dd,
                    p.chip,
                    Some(viewport),
                    ctx.scene,
                    ctx.text_system,
                    theme,
                );
                let panel_rect = dd.popover_rect_clamped(p.chip, viewport);
                for (i, opt) in dd.options.iter().enumerate() {
                    ctx.host
                        .hit_index_mut()
                        .register(opt.id, dd.option_rect_in(p.chip, panel_rect, i));
                }
            }
            4 => {
                let dd = Dropdown::new(
                    ids::CEQ_QUANTIZE_DROPDOWN,
                    "Quantize".to_string(),
                    quantize_options(),
                )
                .selected(snapshot_for_popover.quantize_colors)
                .state(DropdownState::Focused)
                .open(true);
                paint_dropdown_popover_in_viewport(
                    &dd,
                    p.chip,
                    Some(viewport),
                    ctx.scene,
                    ctx.text_system,
                    theme,
                );
                let panel_rect = dd.popover_rect_clamped(p.chip, viewport);
                for (i, opt) in dd.options.iter().enumerate() {
                    ctx.host
                        .hit_index_mut()
                        .register(opt.id, dd.option_rect_in(p.chip, panel_rect, i));
                }
            }
            _ => {}
        }
    }
}
