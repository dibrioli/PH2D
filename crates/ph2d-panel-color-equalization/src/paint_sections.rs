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
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, Dropdown, DropdownOption, paint_button, paint_dropdown_chip,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme};
use ph2d_tool_color_equalization::lut_presets::LutPreset;
use ph2d_tool_color_equalization::params::ColorEqualizationUiSnapshot;
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

// ── Slider rows (13 rows) — the TABLE lives in `rows.rs` ─────────────
//
// Split out when the `show` gate landed and pushed this file past its
// frozen 660-LOC allowance: an allowance is a debt, not a budget.
// ⚠️ The allowance entry in `architecture_panel_loc_cap.rs` now reads
// 660 for a 540-LOC file and should be lowered — that file is outside
// this line's ownership, so it is reported instead of edited.
use crate::rows::build_slider_rows;

/// Eleven+two labeled slider+chip rows (Phase 1/2 stages — clip, tile
/// grid, exposure, temperature, tint, brightness, contrast, vibrance,
/// saturation, sharpen amount + radius, LUT intensity + mix).
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
        // ⚠️ A row whose stage is not running is not painted AND not
        // hit-indexed: a control the artist can see and move must be a
        // control something reads.
        if !(row.show)(snapshot) {
            continue;
        }
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
        let visual = store.dropdown_visual(*chip_id);
        let dd = Dropdown::new(
            *chip_id,
            slot_label.to_string(),
            lut_options_for_slot(*slot),
        )
        .selected(*preset)
        .visual(visual)
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
    let gap = Spacing::Xs.px();
    // ⚠️ **The Dither toggle shares this row with Posterize, and only
    // Posterize is unconditional** — it is the stage's own on-switch.
    // With Posterize off, `posterize(..)` never runs and the toggle
    // reaches nothing, so the dropdown takes the full width rather than
    // leaving a live-looking button beside an empty half.
    let dither_reachable = snapshot.posterize_stage_runs();
    let half = if dither_reachable {
        ((layout.inner_w - gap) * 0.5).max(0.0)
    } else {
        layout.inner_w
    };
    // Mini-label acima de cada dropdown (Enio 2026-05-26: "Esses
    // dropdown precisam de labels para o usuário saber do que se
    // trata"). Font Xs / cor Text2 / pequeno gap, igual o padrão
    // do topbar rail chips.
    let mini_label_font = ph2d_tokens::TypeToken::Xs.px();
    let mini_label_h = mini_label_font + 2.0; // LITERAL-PX-OK: tight ascent margin
    let label_gap = 2.0_f32; // LITERAL-PX-OK: gap label→dropdown

    // Posterize label + dropdown chip (left half).
    paint_text(
        text_system,
        scene,
        "Posterize",
        layout.inner_x,
        y_in,
        mini_label_font,
        half,
        resolve(ColorToken::Text2, theme),
    );
    let chip_y = y_in + mini_label_h + label_gap;
    let post_open = matches!(
        store.get(ids::CEQ_POSTERIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let post_visual = store.dropdown_visual(ids::CEQ_POSTERIZE_DROPDOWN);
    let post_chip_rect = Rect::new(layout.inner_x, chip_y, half, layout.row_h);
    let post_dd = Dropdown::new(
        ids::CEQ_POSTERIZE_DROPDOWN,
        "Posterize".to_string(),
        posterize_options(),
    )
    .selected(snapshot.posterize_levels)
    .visual(post_visual)
    .open(post_open);
    paint_dropdown_chip(&post_dd, post_chip_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_POSTERIZE_DROPDOWN, post_chip_rect);
    if post_open {
        state::push_pending_popover(PendingDropdownPopover {
            slot: 3,
            chip: post_chip_rect,
        });
    }

    // Dither toggle (right half) — paint label "Dither" acima também
    // pra parity visual com Posterize/Quantize. Only exists once
    // Posterize has a stage for it to modify.
    if dither_reachable {
        paint_text(
            text_system,
            scene,
            "Dither",
            layout.inner_x + half + gap,
            y_in,
            mini_label_font,
            half,
            resolve(ColorToken::Text2, theme),
        );
        let dith_rect = Rect::new(layout.inner_x + half + gap, chip_y, half, layout.row_h);
        let dith_active = snapshot.posterize_dithering;
        let dith_kind = if dith_active {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let dith_btn_state = if dith_active {
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            store.button_visual(ids::CEQ_POSTERIZE_DITHERING)
        };
        let dith_label = if dith_active { "Dither: On" } else { "Dither" };
        let dith_button = Button::new(ids::CEQ_POSTERIZE_DITHERING, dith_label)
            .kind(dith_kind)
            .visual(dith_btn_state);
        paint_button(&dith_button, dith_rect, scene, text_system, theme);
        hit_index.register(ids::CEQ_POSTERIZE_DITHERING, dith_rect);
    }

    let mut y = chip_y + layout.row_h + layout.row_gap;

    // Dither Strength + Grain sliders (Enio 2026-05-26). Pintados como
    // duas linhas slider+chip adaptativos (mesmo padrão de Phase 1/2).
    //
    // ⭐ Both values are handed to `posterize(..)` and read only inside
    // its dither sub-pass. They need TWO facts to matter — Posterize on
    // AND Dither on — and the panel is born with `posterize_levels == 0`,
    // so the artist reaches two live-looking sliders in the first second
    // over a stage that never runs.
    if snapshot.dither_stage_runs() {
        let dither_strength_track = store
            .slider(ids::CEQ_POSTERIZE_DITHER_STRENGTH)
            .map(|(_, v)| v)
            .unwrap_or(snapshot.posterize_dither_strength01);
        let dither_strength_chip = store
            .number_value(ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM)
            .unwrap_or(snapshot.posterize_dither_strength as f64);
        let dither_strength_display = format!("{:.2}", snapshot.posterize_dither_strength);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(layout.inner_x, y, layout.inner_w, layout.row_h),
            "Dither Strength",
            dither_strength_track,
            dither_strength_chip,
            Some(&dither_strength_display),
            ids::CEQ_POSTERIZE_DITHER_STRENGTH,
            ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            layout.label_col_w,
            layout.chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + layout.row_gap;

        let dither_grain_track = store
            .slider(ids::CEQ_POSTERIZE_DITHER_GRAIN)
            .map(|(_, v)| v)
            .unwrap_or(snapshot.posterize_dither_grain01);
        let dither_grain_chip = store
            .number_value(ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM)
            .unwrap_or(snapshot.posterize_dither_grain as f64);
        let dither_grain_display = format!("{}", snapshot.posterize_dither_grain);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(layout.inner_x, y, layout.inner_w, layout.row_h),
            "Dither Grain",
            dither_grain_track,
            dither_grain_chip,
            Some(&dither_grain_display),
            ids::CEQ_POSTERIZE_DITHER_GRAIN,
            ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
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

    // Quantize label + dropdown chip — full width.
    paint_text(
        text_system,
        scene,
        "Quantize",
        layout.inner_x,
        y,
        mini_label_font,
        layout.inner_w,
        resolve(ColorToken::Text2, theme),
    );
    y += mini_label_h + label_gap;
    let quant_open = matches!(
        store.get(ids::CEQ_QUANTIZE_DROPDOWN),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    let quant_visual = store.dropdown_visual(ids::CEQ_QUANTIZE_DROPDOWN);
    let quant_chip_rect = Rect::new(layout.inner_x, y, layout.inner_w, layout.row_h);
    let quant_dd = Dropdown::new(
        ids::CEQ_QUANTIZE_DROPDOWN,
        "Quantize".to_string(),
        quantize_options(),
    )
    .selected(snapshot.quantize_colors)
    .visual(quant_visual)
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
            (ButtonState::Pressed, ph2d_editor_core::motion::SETTLED)
        } else {
            store.button_visual(*id)
        };
        let kind = if *on {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let label = if *on { *on_label } else { *off_label };
        let button = Button::new(*id, label).kind(kind).visual(state);
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
    let reset_state = store.button_visual(ids::CEQ_RESET);
    let reset = Button::new(ids::CEQ_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .visual(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_RESET, reset_rect);
    let mut y = y_in + layout.row_h + layout.row_gap;

    let half_btn = ((layout.inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(layout.inner_x, y, half_btn, layout.row_h);
    let cancel_state = store.button_visual(ids::CEQ_CANCEL);
    let cancel = Button::new(ids::CEQ_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .visual(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::CEQ_CANCEL, cancel_rect);
    let apply_rect = Rect::new(
        layout.inner_x + half_btn + btn_gap,
        y,
        half_btn,
        layout.row_h,
    );
    let apply_state = store.button_visual(ids::CEQ_APPLY);
    let apply = Button::new(ids::CEQ_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .visual(apply_state);
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
