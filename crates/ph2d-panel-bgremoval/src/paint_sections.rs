//! Per-section paint helpers for the BgRemoval docked panel.
//!
//! Wave 11 §2.2 (ADR-0042 §6 #3) — split of the 400-LOC `paint()`
//! orchestrator into 6 section helpers so each section reads
//! end-to-end in a single screenful. Same recipe as the CEQ split
//! (Wave 10 Etapa 5.2): each helper takes the per-frame mutables +
//! a `y_in: f32` cursor and returns `y_out: f32`.
//!
//! The sections (in paint order):
//!
//! 1. Tolerance / Feather / Refine sliders ([`paint_slider_rows`])
//! 2. Grow / Shrink bipolar chip ([`paint_grow_shrink`])
//! 3. Separate Islands toggle + Min Px slider ([`paint_islands`])
//! 4. Eyedropper toggle + extra-colour swatches ([`paint_eyedropper_swatches`])
//! 5. Protection brush toggle + size/falloff/show-mask/clear
//!    ([`paint_protect_brush`])
//! 6. Reset row + Cancel/Apply CTA row ([`paint_apply_cta`])

use crate::ids;
use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::widget::panel_chrome::paint_segmented_group_adaptive;
use ph2d_editor_core::widget::{
    Button, ButtonKind, ButtonState, ColorSwatch, SwatchSize, paint_button, paint_color_swatch,
    paint_slider_with_chip_layout_adaptive,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Spacing, Theme};
use ph2d_tool_bgremoval::params::{
    BgRemovalUiSnapshot, BrushFalloff, MIN_ISLAND_PIXELS_FULL_SCALE,
};
use ph2d_vector::VectorScene;

const LABEL_COL_W: f32 = 76.0; // LITERAL-PX-OK: panel grid metric (per-panel label gutter width)

/// Tolerance / Feather / Refine — three canonical slider+chip rows.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_slider_rows(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    chip_w: f32,
    mut y: f32,
) -> f32 {
    for (label, id, chip_id, fallback) in [
        (
            "Tolerance",
            ids::BGR_TOLERANCE,
            ids::BGR_TOLERANCE_NUM,
            snapshot.tolerance01,
        ),
        (
            "Feather",
            ids::BGR_FEATHER,
            ids::BGR_FEATHER_NUM,
            snapshot.feather01,
        ),
        (
            "Refine",
            ids::BGR_REFINE,
            ids::BGR_REFINE_NUM,
            snapshot.refine01,
        ),
    ] {
        let value = store.slider(id).map(|(_, v)| v).unwrap_or(fallback);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            label,
            value,
            value as f64,
            None,
            id,
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
    y
}

/// Grow / Shrink — bipolar slider+chip; chip display remapped to
/// signed `-1..+1` readout (0 at the 0.5 track centre).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_grow_shrink(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    chip_w: f32,
    mut y: f32,
) -> f32 {
    let grow_v = store
        .slider(ids::BGR_GROW)
        .map(|(_, v)| v)
        .unwrap_or(snapshot.grow01);
    let signed = (grow_v - 0.5) * 2.0;
    let grow_eps = 0.005_f32; // LITERAL-PX-OK: display rounding epsilon for −1..+1 grow readout
    let grow_display = if signed.abs() < grow_eps {
        "0.00".to_string() // LITERAL-PX-OK: zero-centred grow display string (numeric value, not a UI metric)
    } else {
        format!("{signed:+.2}")
    };
    let used = paint_slider_with_chip_layout_adaptive(
        Rect::new(inner_x, y, inner_w, row_h),
        "Grow",
        grow_v,
        signed as f64,
        Some(&grow_display),
        ids::BGR_GROW,
        ids::BGR_GROW_NUM,
        LABEL_COL_W,
        chip_w,
        store,
        hit_index,
        scene,
        text_system,
        theme,
    );
    y += used + row_gap;
    y += row_gap; // extra spacing after grow
    y
}

/// Separate Islands toggle + Min Px slider (shown only when armed).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_islands(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    chip_w: f32,
    mut y: f32,
) -> f32 {
    let islands_state = if snapshot.separate_islands {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::BGR_SEPARATE_ISLANDS)
            .unwrap_or(ButtonState::Normal)
    };
    let islands_kind = if snapshot.separate_islands {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let islands_rect = Rect::new(inner_x, y, inner_w, row_h);
    let islands_btn = Button::new(ids::BGR_SEPARATE_ISLANDS, "Separate islands")
        .kind(islands_kind)
        .state(islands_state);
    paint_button(&islands_btn, islands_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_SEPARATE_ISLANDS, islands_rect);
    y += row_h + row_gap;

    if snapshot.separate_islands {
        let min_v = store
            .slider(ids::BGR_MIN_ISLAND_PX)
            .map(|(_, v)| v)
            .unwrap_or(snapshot.min_island_pixels01);
        let min_count =
            ((min_v.clamp(0.0, 1.0) * (MIN_ISLAND_PIXELS_FULL_SCALE - 1.0)).round() as u32 + 1)
                .max(1);
        let min_display = format!("{min_count}");
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            "Min px",
            min_v,
            min_count as f64,
            Some(&min_display),
            ids::BGR_MIN_ISLAND_PX,
            ids::BGR_MIN_ISLAND_PX_NUM,
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
    y += row_gap; // extra spacing after islands block
    y
}

/// Eyedropper toggle + extra-colour swatch row(s).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_eyedropper_swatches(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    mut y: f32,
) -> f32 {
    // Lens-F follow-up (Enio 2026-05-26): Pick Colors doesn't apply
    // when Detect Subject is on — the silhouette path bypasses the
    // chroma backend entirely. Repurpose this slot as the "Add area"
    // toggle that arms the automatic flood-fill selector (single click
    // on canvas → algorithm picks the connected same-colour region
    // into the force-remove mask). When Detect Subject is off, paint
    // the original eyedropper toggle.
    if snapshot.auto_protect_subject {
        let add_area_state = if snapshot.add_area_armed {
            ButtonState::Pressed
        } else {
            store
                .button_state(ids::BGR_ADD_AREA)
                .unwrap_or(ButtonState::Normal)
        };
        let add_area_kind = if snapshot.add_area_armed {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let add_area_rect = Rect::new(inner_x, y, inner_w, row_h);
        let add_area = Button::new(ids::BGR_ADD_AREA, "Add area")
            .kind(add_area_kind)
            .state(add_area_state);
        paint_button(&add_area, add_area_rect, scene, text_system, theme);
        hit_index.register(ids::BGR_ADD_AREA, add_area_rect);
        y += row_h + row_gap;

        // Clear button — visible only when the user has filled any
        // force-remove pixels (mirror of `BGR_PROTECT_CLEAR`).
        if snapshot.has_force_remove_mask {
            let clear_state = store
                .button_state(ids::BGR_ADD_AREA_CLEAR)
                .unwrap_or(ButtonState::Normal);
            let clear_rect = Rect::new(inner_x, y, inner_w, row_h);
            let clear = Button::new(ids::BGR_ADD_AREA_CLEAR, "Clear added areas")
                .kind(ButtonKind::Default)
                .state(clear_state);
            paint_button(&clear, clear_rect, scene, text_system, theme);
            hit_index.register(ids::BGR_ADD_AREA_CLEAR, clear_rect);
            y += row_h + row_gap;
        }
        return y;
    }

    let eyedropper_state = if snapshot.eyedropper_armed {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::BGR_EYEDROPPER)
            .unwrap_or(ButtonState::Normal)
    };
    let eyedropper_kind = if snapshot.eyedropper_armed {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let eyedropper_rect = Rect::new(inner_x, y, inner_w, row_h);
    let eyedropper = Button::new(ids::BGR_EYEDROPPER, "Pick colors")
        .kind(eyedropper_kind)
        .state(eyedropper_state);
    paint_button(&eyedropper, eyedropper_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_EYEDROPPER, eyedropper_rect);
    y += row_h + row_gap;

    let swatch_px = SwatchSize::Sm.px();
    let swatch_gap = Spacing::Xs.px();
    if snapshot.extra_colors.is_empty() {
        y += swatch_px;
    } else {
        let per_row = (((inner_w + swatch_gap) / (swatch_px + swatch_gap)).floor() as usize).max(1);
        let mut col = 0usize;
        let mut sx = inner_x;
        for (i, color) in snapshot.extra_colors.iter().enumerate() {
            if col == per_row {
                col = 0;
                sx = inner_x;
                y += swatch_px + swatch_gap;
            }
            let rect = Rect::new(sx, y, swatch_px, swatch_px);
            let rgba = [color[0], color[1], color[2], 255];
            let sw = ColorSwatch::new(ids::BGR_SWATCHES[i], "Extra bg colour", rgba)
                .size(SwatchSize::Sm);
            paint_color_swatch(&sw, rect, scene, theme);
            hit_index.register(ids::BGR_SWATCHES[i], rect);
            sx += swatch_px + swatch_gap;
            col += 1;
        }
        y += swatch_px;
    }
    y += row_gap;
    y
}

/// "Detect subject" toggle — edge-aware silhouette upgrade (Enio
/// 2026-05-26). Single accent-when-on button; the tool runs the
/// silhouette detector on every pipeline tick while it's on,
/// force-keeping every pixel inside the detected subject contour.
/// Sits between the eyedropper swatches and the protection brush.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_auto_protect_subject(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    mut y: f32,
) -> f32 {
    let on = snapshot.auto_protect_subject;
    let btn_state = if on {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::BGR_AUTO_PROTECT_SUBJECT)
            .unwrap_or(ButtonState::Normal)
    };
    let btn_kind = if on {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let rect = Rect::new(inner_x, y, inner_w, row_h);
    let btn = Button::new(ids::BGR_AUTO_PROTECT_SUBJECT, "Detect subject")
        .kind(btn_kind)
        .state(btn_state);
    paint_button(&btn, rect, scene, text_system, theme);
    hit_index.register(ids::BGR_AUTO_PROTECT_SUBJECT, rect);
    y += row_h + row_gap;
    y
}

/// Protection brush toggle + (armed) brush size & falloff + show-mask
/// + (mask exists) clear button.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_protect_brush(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    snapshot: &BgRemovalUiSnapshot,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    chip_w: f32,
    mut y: f32,
) -> f32 {
    let protect_state = if snapshot.protect_brush_armed {
        ButtonState::Pressed
    } else {
        store
            .button_state(ids::BGR_PROTECT)
            .unwrap_or(ButtonState::Normal)
    };
    let protect_kind = if snapshot.protect_brush_armed {
        ButtonKind::Accent
    } else {
        ButtonKind::Default
    };
    let protect_rect = Rect::new(inner_x, y, inner_w, row_h);
    let protect = Button::new(ids::BGR_PROTECT, "Protect")
        .kind(protect_kind)
        .state(protect_state);
    paint_button(&protect, protect_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_PROTECT, protect_rect);
    y += row_h + row_gap;

    if snapshot.protect_brush_armed {
        let size_v = store
            .slider(ids::BGR_BRUSH_SIZE)
            .map(|(_, v)| v)
            .unwrap_or(snapshot.brush_size01);
        let used = paint_slider_with_chip_layout_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            "Size",
            size_v,
            size_v as f64,
            None,
            ids::BGR_BRUSH_SIZE,
            ids::BGR_BRUSH_SIZE_NUM,
            LABEL_COL_W,
            chip_w,
            store,
            hit_index,
            scene,
            text_system,
            theme,
        );
        y += used + row_gap;

        let falloff_h = paint_segmented_group_adaptive(
            Rect::new(inner_x, y, inner_w, row_h),
            &[
                (
                    BrushFalloff::Smooth.label(),
                    snapshot.falloff == BrushFalloff::Smooth,
                    ids::BGR_FALLOFF_SMOOTH,
                ),
                (
                    BrushFalloff::Sphere.label(),
                    snapshot.falloff == BrushFalloff::Sphere,
                    ids::BGR_FALLOFF_SPHERE,
                ),
                (
                    BrushFalloff::Sharp.label(),
                    snapshot.falloff == BrushFalloff::Sharp,
                    ids::BGR_FALLOFF_SHARP,
                ),
                (
                    BrushFalloff::Constant.label(),
                    snapshot.falloff == BrushFalloff::Constant,
                    ids::BGR_FALLOFF_CONSTANT,
                ),
            ],
            scene,
            text_system,
            theme,
            hit_index,
        );
        y += falloff_h + row_gap;
    }

    if snapshot.protect_brush_armed || snapshot.has_protect_mask {
        let show_state = if snapshot.show_mask {
            ButtonState::Pressed
        } else {
            store
                .button_state(ids::BGR_SHOW_MASK)
                .unwrap_or(ButtonState::Normal)
        };
        let show_kind = if snapshot.show_mask {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        let show_rect = Rect::new(inner_x, y, inner_w, row_h);
        let show = Button::new(ids::BGR_SHOW_MASK, "Show mask")
            .kind(show_kind)
            .state(show_state);
        paint_button(&show, show_rect, scene, text_system, theme);
        hit_index.register(ids::BGR_SHOW_MASK, show_rect);
        y += row_h + row_gap;
    }

    if snapshot.has_protect_mask {
        let clear_rect = Rect::new(inner_x, y, inner_w, row_h);
        let clear_state = store
            .button_state(ids::BGR_PROTECT_CLEAR)
            .unwrap_or(ButtonState::Normal);
        let clear = Button::new(ids::BGR_PROTECT_CLEAR, "Clear protection")
            .kind(ButtonKind::Default)
            .state(clear_state);
        paint_button(&clear, clear_rect, scene, text_system, theme);
        hit_index.register(ids::BGR_PROTECT_CLEAR, clear_rect);
        y += row_h + row_gap;
    }
    y
}

/// Reset row (ghost, full width) + Cancel/Apply row (ghost / accent CTA).
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_apply_cta(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    theme: Theme,
    inner_x: f32,
    inner_w: f32,
    row_h: f32,
    row_gap: f32,
    mut y: f32,
) -> f32 {
    let btn_gap = Spacing::Sm.px();

    let reset_rect = Rect::new(inner_x, y, inner_w, row_h);
    let reset_state = store
        .button_state(ids::BGR_RESET)
        .unwrap_or(ButtonState::Normal);
    let reset = Button::new(ids::BGR_RESET, "Reset to Defaults")
        .kind(ButtonKind::Default)
        .state(reset_state);
    paint_button(&reset, reset_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_RESET, reset_rect);
    y += row_h + row_gap;

    let half_btn = ((inner_w - btn_gap) * 0.5).max(0.0);
    let cancel_rect = Rect::new(inner_x, y, half_btn, row_h);
    let cancel_state = store
        .button_state(ids::BGR_CANCEL)
        .unwrap_or(ButtonState::Normal);
    let cancel = Button::new(ids::BGR_CANCEL, "Cancel")
        .kind(ButtonKind::Default)
        .state(cancel_state);
    paint_button(&cancel, cancel_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_CANCEL, cancel_rect);
    let apply_rect = Rect::new(inner_x + half_btn + btn_gap, y, half_btn, row_h);
    let apply_state = store
        .button_state(ids::BGR_APPLY)
        .unwrap_or(ButtonState::Normal);
    let apply = Button::new(ids::BGR_APPLY, "Apply")
        .kind(ButtonKind::Accent)
        .state(apply_state);
    paint_button(&apply, apply_rect, scene, text_system, theme);
    hit_index.register(ids::BGR_APPLY, apply_rect);
    y += row_h;
    y
}
