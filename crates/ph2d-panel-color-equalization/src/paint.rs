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
//! After the panel title, the body paints (in order): a Phase 2
//! histogram overlay strip (R/G/B bars from the live preview, drawn
//! with the `Danger` / `Success` / `Info` semantic tokens so the
//! channel colours follow the active theme); twelve labeled slider +
//! chip rows for the Phase 1 stages (clip, tile grid, exposure,
//! temperature, tint, brightness, contrast, vibrance, saturation) and
//! Phase 2 effects (sharpen amount, sharpen radius, denoise strength);
//! a 2×2 auto-* toggle grid (Auto Levels, Auto Contrast, Auto Colors,
//! Auto WB); and finally the Cancel + Apply CTA row.

use crate::state::{self, PendingDropdownPopover, set_last_content_h, set_last_visible_h};
use crate::{ColorEqualizationPanel, ColorEqualizationPanelState, ids};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::paint::{fill_rounded_rect, rect_to_vello, resolve, stroke_rounded_rect};
use ph2d_editor_core::panel::{PaintCtx, Panel};
use ph2d_editor_core::widget::panel_chrome::{
    PANEL_HEAD_PAD, PANEL_TITLE_BASELINE, paint_panel_corner_dot, paint_panel_surface,
    paint_panel_title,
};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, COLOR_EQUALIZATION_SCROLLBAR_ID, Dropdown, DropdownOption,
    DropdownState, paint_button, paint_dropdown_chip, paint_dropdown_popover_in_viewport,
    paint_scrollbar, paint_slider_with_chip_layout, scrollbar_is_needed, scrollbar_thumb_rect,
    scrollbar_track_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, StrokeToken, Theme};
use ph2d_tool_color_equalization::algorithm::HistogramData;
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_color_equalization::params::{
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, denoise_strength_to_slider,
    exposure_to_slider, lut_intensity_to_slider, lut_mix_to_slider, saturation_to_slider,
    sharpen_amount_to_slider, sharpen_radius_to_slider, temperature_to_slider, tile_grid_to_slider,
    tint_to_slider, vibrance_to_slider,
};
use ph2d_vector::VectorScene;

/// Label column width for slider rows. // LITERAL-PX-OK: panel grid metric
const LABEL_COL_W: f32 = 84.0;

/// Height of the histogram overlay strip. // LITERAL-PX-OK: panel grid metric
const HISTOGRAM_H: f32 = 64.0;

/// Number of bars per RGB channel in the histogram overlay. 256 bins
/// down-sampled to 64 bars (4 bins per bar) keeps the painter snappy
/// while still showing meaningful distribution shape.
// LITERAL-PX-OK: visualization grid count, not a UI metric.
const HISTOGRAM_BARS: usize = 64;

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
    // Body region (everything below the title, above the corner dot) is
    // clipped + scrolled. CEQ grew past the dock height once Phase 2/3
    // landed (histogram + 14 slider rows + LUT cycles + 4 auto buttons +
    // CTA = ~810 px against a ~600 px panel). Wheel + scrollbar route
    // through `COLOR_EQUALIZATION_SCROLLBAR_ID` → `CEQ_PANEL` so the
    // bottom controls stay reachable.
    let body_top = rect.y + PANEL_TITLE_BASELINE + title_size + Spacing::Md.px();
    let body_h = (rect.y + rect.h - body_top - PANEL_HEAD_PAD).max(0.0);
    let body_rect = Rect::new(rect.x, body_top, rect.w, body_h);
    let scroll = ctx.host.store().panel_scroll(ids::CEQ_PANEL);

    ctx.scene.push_clip(&rect_to_vello(body_rect));
    let mut y = body_top - scroll;

    let scene = &mut *ctx.scene;
    let text_system = &mut *ctx.text_system;
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();

    // ── Phase 2 histogram overlay ──────────────────────────────────
    let hist_rect = Rect::new(inner_x, y, inner_w, HISTOGRAM_H);
    state::with_current_histogram(|h| {
        paint_histogram_overlay(hist_rect, h, scene, theme);
    });
    y += HISTOGRAM_H + row_gap;

    // ── Twelve slider+chip rows ────────────────────────────────────
    // Each row: live track ?? snapshot-derived track; chip displays
    // live number ?? snapshot natural unit.
    struct Row {
        label: &'static str,
        slider_id: NodeId,
        chip_id: NodeId,
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
            label: "Exposure",
            slider_id: ids::CEQ_EXPOSURE,
            chip_id: ids::CEQ_EXPOSURE_NUM,
            snap_track: exposure_to_slider(snapshot.exposure),
            snap_chip: snapshot.exposure as f64,
            chip_display: format!("{:+.2} EV", snapshot.exposure),
        },
        Row {
            label: "Temperature",
            slider_id: ids::CEQ_TEMPERATURE,
            chip_id: ids::CEQ_TEMPERATURE_NUM,
            snap_track: temperature_to_slider(snapshot.temperature),
            snap_chip: snapshot.temperature as f64,
            chip_display: format!("{:+.2}", snapshot.temperature),
        },
        Row {
            label: "Tint",
            slider_id: ids::CEQ_TINT,
            chip_id: ids::CEQ_TINT_NUM,
            snap_track: tint_to_slider(snapshot.tint),
            snap_chip: snapshot.tint as f64,
            chip_display: format!("{:+.2}", snapshot.tint),
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
            label: "Vibrance",
            slider_id: ids::CEQ_VIBRANCE,
            chip_id: ids::CEQ_VIBRANCE_NUM,
            snap_track: vibrance_to_slider(snapshot.vibrance),
            snap_chip: snapshot.vibrance as f64,
            chip_display: format!("{:+.2}", snapshot.vibrance),
        },
        Row {
            label: "Saturation",
            slider_id: ids::CEQ_SATURATION,
            chip_id: ids::CEQ_SATURATION_NUM,
            snap_track: saturation_to_slider(snapshot.saturation),
            snap_chip: snapshot.saturation as f64,
            chip_display: format!("{:+.2}", snapshot.saturation),
        },
        Row {
            label: "Sharpen",
            slider_id: ids::CEQ_SHARPEN_AMOUNT,
            chip_id: ids::CEQ_SHARPEN_AMOUNT_NUM,
            snap_track: sharpen_amount_to_slider(snapshot.sharpen_amount),
            snap_chip: snapshot.sharpen_amount as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_amount),
        },
        Row {
            label: "Radius",
            slider_id: ids::CEQ_SHARPEN_RADIUS,
            chip_id: ids::CEQ_SHARPEN_RADIUS_NUM,
            snap_track: sharpen_radius_to_slider(snapshot.sharpen_radius),
            snap_chip: snapshot.sharpen_radius as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_radius),
        },
        Row {
            label: "Denoise",
            slider_id: ids::CEQ_DENOISE_STRENGTH,
            chip_id: ids::CEQ_DENOISE_STRENGTH_NUM,
            snap_track: denoise_strength_to_slider(snapshot.denoise_strength),
            snap_chip: snapshot.denoise_strength as f64,
            chip_display: format!("{:.2}", snapshot.denoise_strength),
        },
        Row {
            label: "LUT Intensity",
            slider_id: ids::CEQ_LUT_INTENSITY,
            chip_id: ids::CEQ_LUT_INTENSITY_NUM,
            snap_track: lut_intensity_to_slider(snapshot.lut_intensity),
            snap_chip: snapshot.lut_intensity as f64,
            chip_display: format!("{:.2}", snapshot.lut_intensity),
        },
        Row {
            label: "LUT Mix",
            slider_id: ids::CEQ_LUT_MIX,
            chip_id: ids::CEQ_LUT_MIX_NUM,
            snap_track: lut_mix_to_slider(snapshot.lut_mix),
            snap_chip: snapshot.lut_mix as f64,
            chip_display: format!("{:.2}", snapshot.lut_mix),
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

    // ── Phase 3 LUT grouped-select dropdowns ──────────────────────
    // Two side-by-side dropdown chips (closed state painted here; the
    // popover paints AFTER the scroll clip pops via
    // `state::push_pending_lut_popover`, so the options float over
    // every later section). Click on chip → dispatch auto-toggles
    // `open`; click on option → `tool::handle_panel_event` routes
    // back to `SetLutPreset{1,2}` and stages a one-shot close.
    let lut_gap = Spacing::Sm.px();
    let lut_half = ((inner_w - lut_gap) * 0.5).max(0.0);
    let lut_slots = [
        (
            ids::CEQ_LUT_1_DROPDOWN,
            snapshot.lut_preset_1,
            "LUT 1",
            1_u8,
        ),
        (
            ids::CEQ_LUT_2_DROPDOWN,
            snapshot.lut_preset_2,
            "LUT 2",
            2_u8,
        ),
    ];
    for (i, (chip_id, preset, slot_label, slot)) in lut_slots.iter().enumerate() {
        let bx = inner_x + (i as f32) * (lut_half + lut_gap);
        let chip_rect = Rect::new(bx, y, lut_half, row_h);
        let open = matches!(
            store.get(*chip_id),
            Some(InteractiveState::Dropdown { open: true, .. })
        );
        let state = if open {
            DropdownState::Focused
        } else {
            DropdownState::Normal
        };
        let dd = Dropdown::new(
            *chip_id,
            slot_label.to_string(),
            lut_options_for_slot(*slot),
        )
        .selected(*preset)
        .state(state)
        .open(open);
        paint_dropdown_chip(&dd, chip_rect, scene, text_system, theme);
        hit_index.register(*chip_id, chip_rect);
        if open {
            state::push_pending_popover(PendingDropdownPopover {
                slot: *slot,
                chip: chip_rect,
            });
        }
    }
    y += row_h + row_gap;
    y += row_gap;

    // ── Phase 5 Posterize + Quantize dropdowns ────────────────────
    // Posterize: discrete level count (Off / 2 / 3 / 4 / 6 / 8 / 16)
    // + a paired "Dither" toggle (Floyd-Steinberg on/off). Quantize:
    // discrete K-Means++ colour count (Off / 4 / 8 / 16 / 32 / 64 /
    // 128 / 256). Same chip + deferred popover pattern as LUT; the
    // chip's `selected` value is rendered as the natural unit string
    // by `Dropdown::selected_label`.
    let post_open = matches!(
        store.get(ids::CEQ_POSTERIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let post_state = if post_open {
        DropdownState::Focused
    } else {
        DropdownState::Normal
    };
    let post_chip_rect = Rect::new(inner_x, y, lut_half, row_h);
    let post_dd = Dropdown::new(
        ids::CEQ_POSTERIZE_DROPDOWN,
        "Posterize".to_string(),
        posterize_options(),
    )
    .selected(snapshot.posterize_levels)
    .state(post_state)
    .open(post_open);
    paint_dropdown_chip(&post_dd, post_chip_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_POSTERIZE_DROPDOWN, post_chip_rect);
    if post_open {
        state::push_pending_popover(PendingDropdownPopover {
            slot: 3,
            chip: post_chip_rect,
        });
    }
    // Dithering toggle in the right half-slot — only meaningful when
    // posterize is active, but always paintable for predictability.
    let dith_rect = Rect::new(inner_x + lut_half + lut_gap, y, lut_half, row_h);
    let dith_active = snapshot.posterize_dithering;
    let dith_kind = if dith_active {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let dith_btn_state = if dith_active {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::CEQ_POSTERIZE_DITHERING)
            .unwrap_or(ButtonState::Normal)
    };
    let dith_label = if dith_active { "Dither: On" } else { "Dither" };
    let dith_button = Button::new(ids::CEQ_POSTERIZE_DITHERING, dith_label)
        .kind(dith_kind)
        .state(dith_btn_state);
    paint_button(&dith_button, dith_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_POSTERIZE_DITHERING, dith_rect);
    y += row_h + row_gap;

    // Quantize dropdown — full width row (it's the heaviest stage and
    // benefits from the wider chip label, e.g. "Quantize: 256").
    let quant_open = matches!(
        store.get(ids::CEQ_QUANTIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let quant_state = if quant_open {
        DropdownState::Focused
    } else {
        DropdownState::Normal
    };
    let quant_chip_rect = Rect::new(inner_x, y, inner_w, row_h);
    let quant_dd = Dropdown::new(
        ids::CEQ_QUANTIZE_DROPDOWN,
        "Quantize".to_string(),
        quantize_options(),
    )
    .selected(snapshot.quantize_colors)
    .state(quant_state)
    .open(quant_open);
    paint_dropdown_chip(&quant_dd, quant_chip_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_QUANTIZE_DROPDOWN, quant_chip_rect);
    if quant_open {
        state::push_pending_popover(PendingDropdownPopover {
            slot: 4,
            chip: quant_chip_rect,
        });
    }
    y += row_h + row_gap;
    y += row_gap;

    // ── Auto-* toggle 2×2 grid ─────────────────────────────────────
    // Levels / Contrast / Colors / WB — each toggles its own pipeline
    // stage. Accent (pressed) = on. Snapshot drives the active look.
    let auto_gap = Spacing::Sm.px();
    let half = ((inner_w - auto_gap) * 0.5).max(0.0);
    let auto_buttons = [
        (
            ids::CEQ_AUTO_LEVELS,
            snapshot.auto_levels,
            "Auto Levels",
            "Auto Levels: On",
        ),
        (
            ids::CEQ_AUTO_CONTRAST,
            snapshot.auto_contrast,
            "Auto Contrast",
            "Auto Contrast: On",
        ),
        (
            ids::CEQ_AUTO_COLORS,
            snapshot.auto_colors,
            "Auto Colors",
            "Auto Colors: On",
        ),
        (ids::CEQ_AUTO_WB, snapshot.auto_wb, "Auto WB", "Auto WB: On"),
    ];
    for (i, (id, on, off_label, on_label)) in auto_buttons.iter().enumerate() {
        let col = (i % 2) as f32;
        let row = (i / 2) as f32;
        let bx = inner_x + col * (half + auto_gap);
        let by = y + row * (row_h + row_gap);
        let btn_rect = Rect::new(bx, by, half, row_h);
        let state = if *on {
            ButtonState::Pressed
        } else {
            store.button_state(*id).unwrap_or(ButtonState::Normal)
        };
        let kind = if *on {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let label = if *on { *on_label } else { *off_label };
        let button = Button::new(*id, label).kind(kind).state(state);
        paint_button(&button, btn_rect, scene, text_system, theme);
        hit_index.register(*id, btn_rect);
    }
    y += 2.0 * row_h + row_gap;
    y += row_gap;

    // ── Reset (ghost, full width) row ──────────────────────────────
    // Sits ABOVE the Cancel/Apply row so accidental "Reset" doesn't
    // sit next to "Apply" (destructive-adjacent-to-CTA = misclicks).
    let btn_gap = Spacing::Sm.px();
    let reset_rect = Rect::new(inner_x, y, inner_w, row_h);
    let reset_state = store
        .button_state(ids::CEQ_RESET)
        .unwrap_or(ButtonState::Normal);
    let reset = Button::new(ids::CEQ_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_RESET, reset_rect);
    y += row_h + row_gap;

    // ── Cancel (ghost) + Apply (accent CTA) row ────────────────────
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

    // ── Close clip + publish content/visible heights ────────────────
    // `content_h` is the painted body height in body-local coords (undo
    // the scroll subtraction we applied when laying out from `body_top
    // - scroll`). The scrollbar uses this against `body_h` to decide
    // whether it's needed and how tall the thumb is.
    let content_h = (y + scroll) - body_top + PANEL_HEAD_PAD;
    set_last_content_h(content_h);
    set_last_visible_h(body_h);

    ctx.scene.pop_layer();

    // ── Deferred LUT dropdown popovers ──────────────────────────────
    // Painted AFTER `pop_layer` so the option list floats over every
    // later section / panel (mirror of Inspector's
    // `take_pending_dropdown_chip`). Each pending entry was staged by
    // the per-chip block above when its `InteractiveState::Dropdown.
    // open` was `true`.
    // Each pending dropdown gets its popover painted here, on top of
    // every later panel section. Slots 1/2 are LUT presets
    // (`Dropdown<LutPreset>`); slots 3/4 are Posterize / Quantize
    // discrete counts (`Dropdown<u32>`) — `Dropdown<T>` is generic so
    // we paint each branch separately to avoid an enum at the type
    // level.
    // Pass the viewport rect so the popover flips ABOVE the chip when
    // it'd overflow below — the Posterize / Quantize chips sit near
    // the bottom of the panel, where a 16-row LUT list (or even an
    // 8-row Quantize list) extends past the screen bottom.
    let pending = state::take_pending_popovers();
    let snapshot_for_popover = state::current_snapshot();
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
            _ => {
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
        }
    }

    // Vertical scrollbar at the right edge of the body — mirror of
    // GridSnap / Inspector / Widget Gallery. Skips paint + hit when
    // the content fits the visible area.
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

    // Publish content_h / visible_h to the store + clamp scroll so
    // wheel dispatch can bound the offset against painter-known
    // metrics (avoids 1-frame jump when wheeling past the end).
    {
        let store = ctx.host.store_mut();
        store.set_panel_content_h(ids::CEQ_PANEL, content_h);
        store.set_panel_visible_h(ids::CEQ_PANEL, body_h);
        let max_scroll = (content_h - body_h).max(0.0);
        if store.panel_scroll(ids::CEQ_PANEL) > max_scroll {
            store.set_panel_scroll(ids::CEQ_PANEL, max_scroll);
        }
    }
}

/// Paint the RGB histogram overlay inside `rect`. Down-samples each
/// channel's 256 bins to [`HISTOGRAM_BARS`] bars and normalises bar
/// height to the max bin count across all three channels. Semantic
/// tokens proxy channel colours (`Danger` = R, `Success` = G, `Info`
/// = B) so the overlay respects the active theme.
///
/// When `hist` is `None` (host hasn't published one yet) only the empty
/// chrome frame is drawn.
fn paint_histogram_overlay(
    rect: Rect,
    hist: Option<&HistogramData>,
    scene: &mut VectorScene,
    theme: Theme,
) {
    // Backing surface — Bg2 + thin BorderStrong stroke.
    let bg = resolve(ColorToken::Bg2, theme);
    let border = resolve(ColorToken::BorderStrong, theme);
    let radius = Radius::Sm.px();
    fill_rounded_rect(scene, rect, radius, bg);
    stroke_rounded_rect(scene, rect, radius, StrokeToken::Hairline.px(), border);

    let Some(h) = hist else {
        return;
    };
    if h.opaque_count == 0 {
        return;
    }

    // Down-sample each channel's 256 bins into HISTOGRAM_BARS buckets.
    // Each bucket sums `256 / HISTOGRAM_BARS` (= 4) bins.
    let bucket = 256 / HISTOGRAM_BARS;
    let mut r_buckets = [0u32; HISTOGRAM_BARS];
    let mut g_buckets = [0u32; HISTOGRAM_BARS];
    let mut b_buckets = [0u32; HISTOGRAM_BARS];
    for (i, b) in r_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.r[start..start + bucket].iter().sum();
    }
    for (i, b) in g_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.g[start..start + bucket].iter().sum();
    }
    for (i, b) in b_buckets.iter_mut().enumerate() {
        let start = i * bucket;
        *b = h.b[start..start + bucket].iter().sum();
    }
    // Global max across all channels so the bars share a y-scale.
    let max_bucket = r_buckets
        .iter()
        .chain(g_buckets.iter())
        .chain(b_buckets.iter())
        .copied()
        .max()
        .unwrap_or(1)
        .max(1);

    // Geometry: 1-px inset, bars span (HISTOGRAM_BARS) divisions.
    let inset = 2.0_f32;
    let inner = Rect::new(
        rect.x + inset,
        rect.y + inset,
        (rect.w - inset * 2.0).max(0.0),
        (rect.h - inset * 2.0).max(0.0),
    );
    let bar_w = inner.w / HISTOGRAM_BARS as f32;
    let bar_h_max = inner.h;

    let r_color = resolve(ColorToken::Danger, theme);
    let g_color = resolve(ColorToken::Success, theme);
    let b_color = resolve(ColorToken::Info, theme);
    let alpha = 0.6_f32;
    let r_color = with_alpha(r_color, alpha);
    let g_color = with_alpha(g_color, alpha);
    let b_color = with_alpha(b_color, alpha);

    for i in 0..HISTOGRAM_BARS {
        let x = inner.x + i as f32 * bar_w;
        let bar_w_filled = (bar_w - 0.5).max(0.5);
        // Each channel's bar height — overdraw with channel alpha so
        // overlapping regions show their additive sum.
        let hr = (r_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        let hg = (g_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        let hb = (b_buckets[i] as f32 / max_bucket as f32) * bar_h_max;
        if hr > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hr, bar_w_filled, hr),
                0.0,
                r_color,
            );
        }
        if hg > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hg, bar_w_filled, hg),
                0.0,
                g_color,
            );
        }
        if hb > 0.0 {
            fill_rounded_rect(
                scene,
                Rect::new(x, inner.y + bar_h_max - hb, bar_w_filled, hb),
                0.0,
                b_color,
            );
        }
    }
}

/// Reapply `alpha` to a token-resolved Color while keeping its RGB
/// components. Lets us draw semi-transparent histogram bars without
/// editing tokens.json. We deliberately avoid `Color::rgba8(...)` —
/// the channel hue still comes from the active theme via `resolve`.
fn with_alpha(color: ph2d_vector::Color, alpha: f32) -> ph2d_vector::Color {
    color.multiply_alpha(alpha.clamp(0.0, 1.0))
}

/// Build the flat `Vec<DropdownOption<LutPreset>>` for `slot` (1 or 2)
/// in `LutPreset::ALL` order. Preset labels carry the group name as a
/// "Group › Name" prefix when the preset name differs from the group
/// (`Blockbuster`, `Film Noir`, `Warm`, …), giving the visual cue the
/// legacy `createGroupedSelect` got from explicit headers without
/// requiring a new widget feature. The `id` of each option is pulled
/// from `ids::CEQ_LUT_{slot}_OPTS`, aligned to `LutPreset::ALL`, so
/// the tool's `handle_panel_event` can resolve `option_id → preset`
/// by index lookup.
/// Build the Posterize dropdown's flat option list. Index 0 = Off
/// (level value `0`); rest mirror `ids::CEQ_POSTERIZE_LEVELS`. Labels
/// match the legacy panel's "Off", "2 Levels", "3 Levels"… text.
fn posterize_options() -> Vec<DropdownOption<u32>> {
    ids::CEQ_POSTERIZE_OPTS
        .iter()
        .zip(ids::CEQ_POSTERIZE_LEVELS.iter())
        .map(|(id, &level)| {
            let label = if level == 0 {
                "Off".to_string()
            } else {
                format!("{level} Levels")
            };
            DropdownOption::new(*id, level, label)
        })
        .collect()
}

/// Build the Quantize dropdown's flat option list. Index 0 = Off; rest
/// mirror `ids::CEQ_QUANTIZE_COLORS`. Labels read "256 Colors" etc.
fn quantize_options() -> Vec<DropdownOption<u32>> {
    ids::CEQ_QUANTIZE_OPTS
        .iter()
        .zip(ids::CEQ_QUANTIZE_COLORS.iter())
        .map(|(id, &colors)| {
            let label = if colors == 0 {
                "Off".to_string()
            } else {
                format!("{colors} Colors")
            };
            DropdownOption::new(*id, colors, label)
        })
        .collect()
}

fn lut_options_for_slot(slot: u8) -> Vec<DropdownOption<LutPreset>> {
    let opt_ids = match slot {
        1 => &ids::CEQ_LUT_1_OPTS,
        _ => &ids::CEQ_LUT_2_OPTS,
    };
    // Label is just the preset name (no group prefix). The natural
    // ordering of `LutPreset::ALL` already clusters by group (Cinematic
    // → Atmosphere → Vintage → Stylized), which gives the visual
    // grouping without forcing every row to span two lines on the
    // narrow docked-panel chip. A previous "Group › Name" form ran
    // off the row width and wrapped, getting truncated by the popover
    // row clip.
    LutPreset::ALL
        .iter()
        .enumerate()
        .map(|(i, preset)| DropdownOption::new(opt_ids[i], *preset, preset.label().to_string()))
        .collect()
}
