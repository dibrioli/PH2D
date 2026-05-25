//! Color Equalization panel — per-section paint helpers.
//!
//! Split out from `paint.rs` so the orchestrator fits the Wave 10
//! `panel-*` LOC cap (600 LOC/file, 200 LOC/fn). Each `paint_*_section`
//! takes the per-frame mutables `paint()` already destructured (scene,
//! text_system, store, hit_index, theme, snapshot) plus the running
//! layout cursor `y_in: f32` and returns the next `y` after the block.
//!
//! No rendering changes vs. the pre-split mega-fn — each block paints
//! the same widgets in the same order with the same arguments.
//!
//! Also hosts the small construction helpers that `paint()` no longer
//! needs to see (`with_alpha`, `paint_histogram_overlay`,
//! `posterize_options`, `quantize_options`, `lut_options_for_slot`) +
//! the histogram constant `HISTOGRAM_BARS`.

use crate::ids;
use crate::state::{self, PendingDropdownPopover};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Dropdown, DropdownOption, DropdownState, paint_button,
    paint_dropdown_chip, paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_color_equalization::params::{
    ColorEqualizationUiSnapshot, brightness_to_slider, clip_limit_to_slider, contrast_to_slider,
    denoise_strength_to_slider, exposure_to_slider, lut_intensity_to_slider, lut_mix_to_slider,
    saturation_to_slider, sharpen_amount_to_slider, sharpen_radius_to_slider,
    temperature_to_slider, tile_grid_to_slider, tint_to_slider, vibrance_to_slider,
};
use ph2d_vector::VectorScene;

/// Shared layout metrics for every section helper, mirroring the locals
/// `paint()` computes once after deriving the inner rect.
#[derive(Clone, Copy)]
pub(crate) struct SectionLayout {
    pub inner_x: f32,
    pub inner_w: f32,
    pub row_h: f32,
    pub row_gap: f32,
    pub chip_w: f32,
    pub label_col_w: f32,
}

// ── Slider rows (14 rows) ─────────────────────────────────────────────

struct SliderRow {
    label: &'static str,
    slider_id: NodeId,
    chip_id: NodeId,
    snap_track: f32,
    snap_chip: f64,
    chip_display: String,
}

fn build_slider_rows(snapshot: &ColorEqualizationUiSnapshot) -> [SliderRow; 14] {
    [
        SliderRow {
            label: "Clip",
            slider_id: ids::CEQ_CLIP_LIMIT,
            chip_id: ids::CEQ_CLIP_LIMIT_NUM,
            snap_track: clip_limit_to_slider(snapshot.clip_limit),
            snap_chip: snapshot.clip_limit as f64,
            chip_display: format!("{:.2}", snapshot.clip_limit),
        },
        SliderRow {
            label: "Tile Grid",
            slider_id: ids::CEQ_TILE_GRID,
            chip_id: ids::CEQ_TILE_GRID_NUM,
            snap_track: tile_grid_to_slider(snapshot.tile_grid_size),
            snap_chip: snapshot.tile_grid_size as f64,
            chip_display: snapshot.tile_grid_size.to_string(),
        },
        SliderRow {
            label: "Exposure",
            slider_id: ids::CEQ_EXPOSURE,
            chip_id: ids::CEQ_EXPOSURE_NUM,
            snap_track: exposure_to_slider(snapshot.exposure),
            snap_chip: snapshot.exposure as f64,
            chip_display: format!("{:+.2} EV", snapshot.exposure),
        },
        SliderRow {
            label: "Temperature",
            slider_id: ids::CEQ_TEMPERATURE,
            chip_id: ids::CEQ_TEMPERATURE_NUM,
            snap_track: temperature_to_slider(snapshot.temperature),
            snap_chip: snapshot.temperature as f64,
            chip_display: format!("{:+.2}", snapshot.temperature),
        },
        SliderRow {
            label: "Tint",
            slider_id: ids::CEQ_TINT,
            chip_id: ids::CEQ_TINT_NUM,
            snap_track: tint_to_slider(snapshot.tint),
            snap_chip: snapshot.tint as f64,
            chip_display: format!("{:+.2}", snapshot.tint),
        },
        SliderRow {
            label: "Brightness",
            slider_id: ids::CEQ_BRIGHTNESS,
            chip_id: ids::CEQ_BRIGHTNESS_NUM,
            snap_track: brightness_to_slider(snapshot.brightness),
            snap_chip: snapshot.brightness as f64,
            chip_display: format!("{:+.2}", snapshot.brightness),
        },
        SliderRow {
            label: "Contrast",
            slider_id: ids::CEQ_CONTRAST,
            chip_id: ids::CEQ_CONTRAST_NUM,
            snap_track: contrast_to_slider(snapshot.contrast),
            snap_chip: snapshot.contrast as f64,
            chip_display: format!("{:.2}", snapshot.contrast),
        },
        SliderRow {
            label: "Vibrance",
            slider_id: ids::CEQ_VIBRANCE,
            chip_id: ids::CEQ_VIBRANCE_NUM,
            snap_track: vibrance_to_slider(snapshot.vibrance),
            snap_chip: snapshot.vibrance as f64,
            chip_display: format!("{:+.2}", snapshot.vibrance),
        },
        SliderRow {
            label: "Saturation",
            slider_id: ids::CEQ_SATURATION,
            chip_id: ids::CEQ_SATURATION_NUM,
            snap_track: saturation_to_slider(snapshot.saturation),
            snap_chip: snapshot.saturation as f64,
            chip_display: format!("{:+.2}", snapshot.saturation),
        },
        SliderRow {
            label: "Sharpen",
            slider_id: ids::CEQ_SHARPEN_AMOUNT,
            chip_id: ids::CEQ_SHARPEN_AMOUNT_NUM,
            snap_track: sharpen_amount_to_slider(snapshot.sharpen_amount),
            snap_chip: snapshot.sharpen_amount as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_amount),
        },
        SliderRow {
            label: "Radius",
            slider_id: ids::CEQ_SHARPEN_RADIUS,
            chip_id: ids::CEQ_SHARPEN_RADIUS_NUM,
            snap_track: sharpen_radius_to_slider(snapshot.sharpen_radius),
            snap_chip: snapshot.sharpen_radius as f64,
            chip_display: format!("{:.2}", snapshot.sharpen_radius),
        },
        SliderRow {
            label: "Denoise",
            slider_id: ids::CEQ_DENOISE_STRENGTH,
            chip_id: ids::CEQ_DENOISE_STRENGTH_NUM,
            snap_track: denoise_strength_to_slider(snapshot.denoise_strength),
            snap_chip: snapshot.denoise_strength as f64,
            chip_display: format!("{:.2}", snapshot.denoise_strength),
        },
        SliderRow {
            label: "LUT Intensity",
            slider_id: ids::CEQ_LUT_INTENSITY,
            chip_id: ids::CEQ_LUT_INTENSITY_NUM,
            snap_track: lut_intensity_to_slider(snapshot.lut_intensity),
            snap_chip: snapshot.lut_intensity as f64,
            chip_display: format!("{:.2}", snapshot.lut_intensity),
        },
        SliderRow {
            label: "LUT Mix",
            slider_id: ids::CEQ_LUT_MIX,
            chip_id: ids::CEQ_LUT_MIX_NUM,
            snap_track: lut_mix_to_slider(snapshot.lut_mix),
            snap_chip: snapshot.lut_mix as f64,
            chip_display: format!("{:.2}", snapshot.lut_mix),
        },
    ]
}

/// Twelve+two labeled slider+chip rows (Phase 1/2 stages — clip, tile
/// grid, exposure, temperature, tint, brightness, contrast, vibrance,
/// saturation, sharpen amount + radius, denoise, LUT intensity + mix).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_slider_rows_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &ColorEqualizationUiSnapshot,
    layout: SectionLayout,
    y_in: f32,
) -> f32 {
    let rows = build_slider_rows(snapshot);
    let mut y = y_in;
    for row in &rows {
        let track = store
            .slider(row.slider_id)
            .map(|(_, v)| v)
            .unwrap_or(row.snap_track);
        let chip_value = store.number_value(row.chip_id).unwrap_or(row.snap_chip);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(layout.inner_x, y, layout.inner_w, layout.row_h),
            row.label,
            track,
            chip_value,
            Some(&row.chip_display),
            row.slider_id,
            row.chip_id,
            layout.label_col_w,
            layout.chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + layout.row_gap;
    }
    y
}

// ── LUT slot dropdowns ───────────────────────────────────────────────

/// Two side-by-side LUT slot dropdown chips. Open popovers are deferred
/// via `state::push_pending_popover` and painted after the scroll clip
/// pops in `paint()`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_lut_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &ColorEqualizationUiSnapshot,
    layout: SectionLayout,
    y_in: f32,
) -> f32 {
    let lut_gap = Spacing::Sm.px();
    let lut_half = ((layout.inner_w - lut_gap) * 0.5).max(0.0);
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
    let mut y = y_in;
    for (i, (chip_id, preset, slot_label, slot)) in lut_slots.iter().enumerate() {
        let bx = layout.inner_x + (i as f32) * (lut_half + lut_gap);
        let chip_rect = Rect::new(bx, y, lut_half, layout.row_h);
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
    y += layout.row_h + layout.row_gap;
    y + layout.row_gap
}

// ── Posterize / Quantize / Dither row ────────────────────────────────

/// Posterize dropdown (left half) + Dither toggle (right half) on one
/// row; Quantize dropdown full-width on the row below.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_posterize_quantize_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &ColorEqualizationUiSnapshot,
    layout: SectionLayout,
    y_in: f32,
) -> f32 {
    let gap = Spacing::Sm.px();
    let half = ((layout.inner_w - gap) * 0.5).max(0.0);

    // Posterize chip (left half).
    let post_open = matches!(
        store.get(ids::CEQ_POSTERIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let post_state = if post_open {
        DropdownState::Focused
    } else {
        DropdownState::Normal
    };
    let post_chip_rect = Rect::new(layout.inner_x, y_in, half, layout.row_h);
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

    // Dither toggle (right half).
    let dith_rect = Rect::new(layout.inner_x + half + gap, y_in, half, layout.row_h);
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

    let mut y = y_in + layout.row_h + layout.row_gap;

    // Quantize chip — full width.
    let quant_open = matches!(
        store.get(ids::CEQ_QUANTIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let quant_state = if quant_open {
        DropdownState::Focused
    } else {
        DropdownState::Normal
    };
    let quant_chip_rect = Rect::new(layout.inner_x, y, layout.inner_w, layout.row_h);
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
    y += layout.row_h + layout.row_gap;
    y + layout.row_gap
}

// ── Auto-* 2×2 grid ──────────────────────────────────────────────────

/// Auto Levels / Auto Contrast / Auto Colors / Auto WB in a 2×2 grid.
/// Each toggles its own pipeline stage. Accent (pressed) = on.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_auto_buttons_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &ColorEqualizationUiSnapshot,
    layout: SectionLayout,
    y_in: f32,
) -> f32 {
    let auto_gap = Spacing::Sm.px();
    let half = ((layout.inner_w - auto_gap) * 0.5).max(0.0);
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
        let bx = layout.inner_x + col * (half + auto_gap);
        let by = y_in + row * (layout.row_h + layout.row_gap);
        let btn_rect = Rect::new(bx, by, half, layout.row_h);
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
    let mut y = y_in + 2.0 * layout.row_h + layout.row_gap;
    y += layout.row_gap;
    y
}

// ── Reset + Cancel/Apply CTA ─────────────────────────────────────────

/// Reset (ghost, full width) row above Cancel (ghost) + Apply (accent
/// CTA) — destructive Reset deliberately not adjacent to Apply.
pub(crate) fn paint_apply_cta_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    layout: SectionLayout,
    y_in: f32,
) -> f32 {
    let btn_gap = Spacing::Sm.px();
    let reset_rect = Rect::new(layout.inner_x, y_in, layout.inner_w, layout.row_h);
    let reset_state = store
        .button_state(ids::CEQ_RESET)
        .unwrap_or(ButtonState::Normal);
    let reset = Button::new(ids::CEQ_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_RESET, reset_rect);
    let mut y = y_in + layout.row_h + layout.row_gap;

    let half_btn = ((layout.inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(layout.inner_x, y, half_btn, layout.row_h);
    let cancel_state = store
        .button_state(ids::CEQ_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::CEQ_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_CANCEL, cancel_rect);
    let apply_rect = Rect::new(
        layout.inner_x + half_btn + btn_gap,
        y,
        half_btn,
        layout.row_h,
    );
    let apply_state = store
        .button_state(ids::CEQ_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::CEQ_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_APPLY, apply_rect);
    y += layout.row_h;
    y
}

// ── Dropdown option builders ─────────────────────────────────────────

/// Build the Posterize dropdown's flat option list. Index 0 = Off
/// (level value `0`); rest mirror `ids::CEQ_POSTERIZE_LEVELS`. Labels
/// match the legacy panel's "Off", "2 Levels", "3 Levels"… text.
pub(crate) fn posterize_options() -> Vec<DropdownOption<u32>> {
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
pub(crate) fn quantize_options() -> Vec<DropdownOption<u32>> {
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

/// Build the flat `Vec<DropdownOption<LutPreset>>` for `slot` (1 or 2)
/// in `LutPreset::ALL` order. Label is just the preset name (no group
/// prefix) — the natural ordering of `LutPreset::ALL` already clusters
/// by group (Cinematic → Atmosphere → Vintage → Stylized), which gives
/// the visual grouping without forcing every row to span two lines on
/// the narrow docked-panel chip. A previous "Group › Name" form ran
/// off the row width and wrapped, getting truncated by the popover
/// row clip.
pub(crate) fn lut_options_for_slot(slot: u8) -> Vec<DropdownOption<LutPreset>> {
    let opt_ids = match slot {
        1 => &ids::CEQ_LUT_1_OPTS,
        _ => &ids::CEQ_LUT_2_OPTS,
    };
    LutPreset::ALL
        .iter()
        .enumerate()
        .map(|(i, preset)| DropdownOption::new(opt_ids[i], *preset, preset.label().to_string()))
        .collect()
}
