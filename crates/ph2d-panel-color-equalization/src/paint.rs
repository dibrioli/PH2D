//! Color Equalization panel paint.
//!
//! Per-frame logic (mirrors `ph2d-panel-padding`):
//! - Visibility gate via [`PanelHostInternal::panel_visible`] +
//!   stale-rect cleanup on hide.
//! - Right-dock rect from `ctx.layout.padding` (Inspector slot — reused
//!   for any Image-Tools docked panel, which are all mutually exclusive).
//! - Chrome publish (`set_panel_rect`) so dispatch can hit-test it.
//! - Canonical chrome: dark-glass surface + corner dot,
//!   [`paint_panel_title`], five labeled slider+chip rows (clip limit /
//!   tile grid / brightness / contrast / saturation) via
//!   [`paint_slider_with_chip_layout`], an Auto-WB toggle, then Cancel +
//!   Apply buttons.
//!
//! Each row's slider track reads the live stored value (so a drag is
//! smooth), falling back to the per-frame snapshot. The chip displays
//! the live stored natural-unit value, falling back to the snapshot.
//! [`crate::event`] keeps the two widgets in lock-step so dragging the
//! slider moves the chip and typing in the chip moves the slider — both
//! in real time.
//!
//! No on-canvas live preview painter here — the canvas overlay is the
//! shell's job (mirrors Padding / BgRemoval).

use crate::state::{self, set_last_content_h, set_last_visible_h};
use crate::{ColorEqualizationPanel, ColorEqualizationPanelState, ids};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, paint_button, paint_slider_with_chip_layout,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};
use ph2d_tool_color_equalization::params::{
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, saturation_to_slider,
    tile_grid_to_slider,
};

/// Label column width for slider rows. // LITERAL-PX-OK: panel grid metric
const LABEL_COL_W: f32 = 84.0;

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

    let inner_x = rect.x + PANEL_HEAD_PAD;
    let inner_w = (rect.w - PANEL_HEAD_PAD * 2.0).max(0.0);
    let row_h = ROW_H_PX;
    let row_gap = Spacing::Sm.px();
    let chip_w = Spacing::Xl.px() * 2.0;

    let title_size = paint_panel_title(
        rect,
        "Color Equalization",
        0.0,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    let mut y = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();

    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // ── Five slider+chip rows ──────────────────────────────────────
    // Each row: live track ?? snapshot-derived track; chip displays
    // live number ?? snapshot natural unit.
    struct Row {
        label: &'static str,
        slider_id: ph2d_a11y::NodeId,
        chip_id: ph2d_a11y::NodeId,
        snap_track: f32,
        snap_chip: f64,
        chip_display: String,
    }
    let rows = [
        Row {
            label: "Clip",
            slider_id: ids::CEQ_CLIP_LIMIT,
            chip_id: ids::CEQ_CLIP_LIMIT_NUM,
            snap_track: clip_limit_to_slider(snapshot.clip_limit),
            snap_chip: snapshot.clip_limit as f64,
            chip_display: format!("{:.2}", snapshot.clip_limit),
        },
        Row {
            label: "Tile Grid",
            slider_id: ids::CEQ_TILE_GRID,
            chip_id: ids::CEQ_TILE_GRID_NUM,
            snap_track: tile_grid_to_slider(snapshot.tile_grid_size),
            snap_chip: snapshot.tile_grid_size as f64,
            chip_display: snapshot.tile_grid_size.to_string(),
        },
        Row {
            label: "Brightness",
            slider_id: ids::CEQ_BRIGHTNESS,
            chip_id: ids::CEQ_BRIGHTNESS_NUM,
            snap_track: brightness_to_slider(snapshot.brightness),
            snap_chip: snapshot.brightness as f64,
            chip_display: format!("{:+.2}", snapshot.brightness),
        },
        Row {
            label: "Contrast",
            slider_id: ids::CEQ_CONTRAST,
            chip_id: ids::CEQ_CONTRAST_NUM,
            snap_track: contrast_to_slider(snapshot.contrast),
            snap_chip: snapshot.contrast as f64,
            chip_display: format!("{:.2}", snapshot.contrast),
        },
        Row {
            label: "Saturation",
            slider_id: ids::CEQ_SATURATION,
            chip_id: ids::CEQ_SATURATION_NUM,
            snap_track: saturation_to_slider(snapshot.saturation),
            snap_chip: snapshot.saturation as f64,
            chip_display: format!("{:+.2}", snapshot.saturation),
        },
    ];
    for row in &rows {
        let track = store
            .slider(row.slider_id)
            .map(|(_, v)| v)
            .unwrap_or(row.snap_track);
        let chip_value = store.number_value(row.chip_id).unwrap_or(row.snap_chip);
        paint_slider_with_chip_layout(
            Rect::new(inner_x, y, inner_w, row_h),
            row.label,
            track,
            chip_value,
            Some(&row.chip_display),
            row.slider_id,
            row.chip_id,
            LABEL_COL_W,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += row_h + row_gap;
    }

    y += row_gap;

    // ── Auto-WB toggle ─────────────────────────────────────────────
    // Accent (pressed) = on (apply Gray-World gain); Default (idle) =
    // off. Snapshot drives the active look (mirrors Padding's pivot
    // toggle / BgRemoval's Show-Mask).
    let wb_on = snapshot.auto_wb;
    let wb_state = if wb_on {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::CEQ_AUTO_WB)
            .unwrap_or(ButtonState::Normal)
    };
    let wb_kind = if wb_on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let wb_label = if wb_on { "Auto WB: On" } else { "Auto WB: Off" };
    let wb_rect = Rect::new(inner_x, y, inner_w, row_h);
    let wb = Button::new(ids::CEQ_AUTO_WB, wb_label)
        .kind(wb_kind)
        .state(wb_state);
    paint_button(&wb, wb_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_AUTO_WB, wb_rect);
    y += row_h + row_gap;

    y += row_gap;

    // ── Cancel (ghost) + Apply (accent CTA) row ────────────────────
    let btn_gap = Spacing::Sm.px();
    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::CEQ_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::CEQ_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::CEQ_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::CEQ_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_APPLY, apply_rect);
    y += row_h;

    let used_h = (y - rect.y + PANEL_HEAD_PAD).min(rect.h);
    set_last_content_h(used_h);
    set_last_visible_h(rect.h);
}
