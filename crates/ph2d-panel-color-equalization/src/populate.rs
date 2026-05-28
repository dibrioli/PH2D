//! Color Equalization panel `populate` — pre-registers the panel's widget
//! slots in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Each row uses `link_slider_number_mapped(scale, offset)` (the chip's
//! **display-space** = natural unit, the slider's **storage** = `0..1`),
//! so:
//!
//! - the **buffer the user sees on focus** is already in natural units
//!   (e.g. `+0.30` for brightness, `2.00` for clip-limit) — matching
//!   the unfocused `display_override`,
//! - a typed value commits **without** silently being interpreted as
//!   `0..1` (the 2026-05-27 BGRemoval Grow bug class),
//! - drag-scrub clamps at the chip's natural bounds (`[min, max]`) and
//!   the slider drag forward-projects storage → natural so the chip
//!   stored value stays in lockstep.
//!
//! Tool still receives slider `0..1` via `SetValue(slider_id, track)`
//! from `event.rs` (single projection site in `apply_ui_edit`).

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    ButtonState, DropdownState, SliderOrientation, SliderState, TextInputState,
};
use ph2d_tool_color_equalization::params::{
    BRIGHTNESS_DEFAULT, BRIGHTNESS_MAX, BRIGHTNESS_MIN, CLIP_LIMIT_DEFAULT, CLIP_LIMIT_MAX,
    CLIP_LIMIT_MIN, CONTRAST_DEFAULT, CONTRAST_MAX, CONTRAST_MIN, EXPOSURE_DEFAULT, EXPOSURE_MAX,
    EXPOSURE_MIN, LUT_INTENSITY_DEFAULT, LUT_INTENSITY_MAX, LUT_INTENSITY_MIN, LUT_MIX_DEFAULT,
    LUT_MIX_MAX, LUT_MIX_MIN, POSTERIZE_DITHER_GRAIN_DEFAULT, POSTERIZE_DITHER_GRAIN_MAX,
    POSTERIZE_DITHER_GRAIN_MIN, POSTERIZE_DITHER_STRENGTH_DEFAULT,
    POSTERIZE_DITHER_STRENGTH_MAX, POSTERIZE_DITHER_STRENGTH_MIN, SATURATION_DEFAULT,
    SATURATION_MAX, SATURATION_MIN, SHARPEN_AMOUNT_DEFAULT, SHARPEN_AMOUNT_MAX,
    SHARPEN_AMOUNT_MIN, SHARPEN_RADIUS_DEFAULT, SHARPEN_RADIUS_MAX, SHARPEN_RADIUS_MIN,
    TEMPERATURE_DEFAULT, TEMPERATURE_MAX, TEMPERATURE_MIN, TILE_GRID_DEFAULT, TILE_GRID_MAX,
    TILE_GRID_MIN, TINT_DEFAULT, TINT_MAX, TINT_MIN, VIBRANCE_DEFAULT, VIBRANCE_MAX, VIBRANCE_MIN,
};

pub fn populate(store: &mut WidgetStore) {
    for id in [
        ids::CEQ_CANCEL,
        ids::CEQ_APPLY,
        ids::CEQ_RESET,
        ids::CEQ_AUTO_LEVELS,
        ids::CEQ_AUTO_CONTRAST,
        ids::CEQ_AUTO_COLORS,
        ids::CEQ_AUTO_WB,
        ids::CEQ_POSTERIZE_DITHERING,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Phase 3/5 dropdowns (LUT slots + Posterize levels + Quantize
    // colours). Click on chip auto-toggles `open` in the shared
    // dispatch handler for `InteractiveState::Dropdown`. Option click
    // routes back to the tool's `handle_panel_event` and stages a
    // one-shot close (`pending_close_lut_dropdown` with slot 1/2/3/4).
    for id in [
        ids::CEQ_LUT_1_DROPDOWN,
        ids::CEQ_LUT_2_DROPDOWN,
        ids::CEQ_POSTERIZE_DROPDOWN,
        ids::CEQ_QUANTIZE_DROPDOWN,
    ] {
        store.register(
            id,
            InteractiveState::Dropdown {
                state: DropdownState::Normal,
                open: false,
                selected_index: None,
            },
        );
    }

    // Dropdown option ids MUST be registered too — `dispatch::pointer`
    // only seeds `active`/`active_rect` on Down for ids that pass
    // `is_focusable(store, id)`, which returns `false` for ids absent
    // from the store. Without a Button registration, Up never reaches
    // `apply_click` and no `Click(option_id)` event fires → the tool
    // never sees the selection and the popover never closes. (The
    // Inspector showcase dropdown has the same gap but is sample-only;
    // production dropdowns must register every option.)
    for id in ids::CEQ_LUT_1_OPTS
        .iter()
        .chain(ids::CEQ_LUT_2_OPTS.iter())
        .chain(ids::CEQ_POSTERIZE_OPTS.iter())
        .chain(ids::CEQ_QUANTIZE_OPTS.iter())
    {
        store.register(
            *id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Per-row: (slider_id, chip_id, natural_default, natural_min, natural_max).
    // Slider's 0..1 storage is computed from the natural default via the
    // tool's `*_to_slider` projection (canonical source). Mapping for
    // `link_slider_number_mapped` is the affine inverse:
    //   display = storage * (max - min) + min
    // (see `slider_to_<param>` in `ph2d-tool-color-equalization::params`).
    let rows: [(NodeId, NodeId, f32, f32, f32); 15] = [
        (
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_CLIP_LIMIT_NUM,
            CLIP_LIMIT_DEFAULT,
            CLIP_LIMIT_MIN,
            CLIP_LIMIT_MAX,
        ),
        (
            ids::CEQ_TILE_GRID,
            ids::CEQ_TILE_GRID_NUM,
            TILE_GRID_DEFAULT as f32,
            TILE_GRID_MIN as f32,
            TILE_GRID_MAX as f32,
        ),
        (
            ids::CEQ_EXPOSURE,
            ids::CEQ_EXPOSURE_NUM,
            EXPOSURE_DEFAULT,
            EXPOSURE_MIN,
            EXPOSURE_MAX,
        ),
        (
            ids::CEQ_TEMPERATURE,
            ids::CEQ_TEMPERATURE_NUM,
            TEMPERATURE_DEFAULT,
            TEMPERATURE_MIN,
            TEMPERATURE_MAX,
        ),
        (
            ids::CEQ_TINT,
            ids::CEQ_TINT_NUM,
            TINT_DEFAULT,
            TINT_MIN,
            TINT_MAX,
        ),
        (
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_BRIGHTNESS_NUM,
            BRIGHTNESS_DEFAULT,
            BRIGHTNESS_MIN,
            BRIGHTNESS_MAX,
        ),
        (
            ids::CEQ_CONTRAST,
            ids::CEQ_CONTRAST_NUM,
            CONTRAST_DEFAULT,
            CONTRAST_MIN,
            CONTRAST_MAX,
        ),
        (
            ids::CEQ_VIBRANCE,
            ids::CEQ_VIBRANCE_NUM,
            VIBRANCE_DEFAULT,
            VIBRANCE_MIN,
            VIBRANCE_MAX,
        ),
        (
            ids::CEQ_SATURATION,
            ids::CEQ_SATURATION_NUM,
            SATURATION_DEFAULT,
            SATURATION_MIN,
            SATURATION_MAX,
        ),
        (
            ids::CEQ_SHARPEN_AMOUNT,
            ids::CEQ_SHARPEN_AMOUNT_NUM,
            SHARPEN_AMOUNT_DEFAULT,
            SHARPEN_AMOUNT_MIN,
            SHARPEN_AMOUNT_MAX,
        ),
        (
            ids::CEQ_SHARPEN_RADIUS,
            ids::CEQ_SHARPEN_RADIUS_NUM,
            SHARPEN_RADIUS_DEFAULT,
            SHARPEN_RADIUS_MIN,
            SHARPEN_RADIUS_MAX,
        ),
        (
            ids::CEQ_LUT_INTENSITY,
            ids::CEQ_LUT_INTENSITY_NUM,
            LUT_INTENSITY_DEFAULT,
            LUT_INTENSITY_MIN,
            LUT_INTENSITY_MAX,
        ),
        (
            ids::CEQ_LUT_MIX,
            ids::CEQ_LUT_MIX_NUM,
            LUT_MIX_DEFAULT,
            LUT_MIX_MIN,
            LUT_MIX_MAX,
        ),
        (
            ids::CEQ_POSTERIZE_DITHER_STRENGTH,
            ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            POSTERIZE_DITHER_STRENGTH_DEFAULT,
            POSTERIZE_DITHER_STRENGTH_MIN,
            POSTERIZE_DITHER_STRENGTH_MAX,
        ),
        (
            ids::CEQ_POSTERIZE_DITHER_GRAIN,
            ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            POSTERIZE_DITHER_GRAIN_DEFAULT as f32,
            POSTERIZE_DITHER_GRAIN_MIN as f32,
            POSTERIZE_DITHER_GRAIN_MAX as f32,
        ),
    ];
    for (slider_id, chip_id, natural_default, min, max) in rows {
        register_mapped_pair(store, slider_id, chip_id, natural_default, min, max);
    }
}

fn register_mapped_pair(
    store: &mut WidgetStore,
    slider_id: NodeId,
    chip_id: NodeId,
    natural_default: f32,
    min: f32,
    max: f32,
) {
    let track = project01(natural_default, min, max);
    let display = natural_default as f64;
    store.register(
        slider_id,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip_id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: display,
            buffer: format_number(display),
            caret: 0,
            last_committed: display,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(slider_id, chip_id, max - min, min);
}

/// Identical to `ph2d_tool_color_equalization::params::project01` but
/// inlined here so populate doesn't pull in the whole `params` math
/// surface for one helper. Kept private to this module.
fn project01(v: f32, min: f32, max: f32) -> f32 {
    if (max - min).abs() < f32::EPSILON {
        0.0
    } else {
        ((v - min) / (max - min)).clamp(0.0, 1.0) // CLAMP-OK: literal bounds 0,1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(48);
        populate(&mut store);
        for id in [
            ids::CEQ_CANCEL,
            ids::CEQ_APPLY,
            ids::CEQ_AUTO_LEVELS,
            ids::CEQ_AUTO_CONTRAST,
            ids::CEQ_AUTO_COLORS,
            ids::CEQ_AUTO_WB,
            ids::CEQ_POSTERIZE_DITHERING,
        ] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [
            ids::CEQ_LUT_1_DROPDOWN,
            ids::CEQ_LUT_2_DROPDOWN,
            ids::CEQ_POSTERIZE_DROPDOWN,
            ids::CEQ_QUANTIZE_DROPDOWN,
        ] {
            assert!(
                matches!(
                    store.get(id),
                    Some(ph2d_editor_core::interaction::InteractiveState::Dropdown { .. })
                ),
                "dropdown {id:?} missing"
            );
        }
        for id in [
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_TILE_GRID,
            ids::CEQ_EXPOSURE,
            ids::CEQ_TEMPERATURE,
            ids::CEQ_TINT,
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_CONTRAST,
            ids::CEQ_VIBRANCE,
            ids::CEQ_SATURATION,
            ids::CEQ_SHARPEN_AMOUNT,
            ids::CEQ_SHARPEN_RADIUS,
            ids::CEQ_LUT_INTENSITY,
            ids::CEQ_LUT_MIX,
            ids::CEQ_POSTERIZE_DITHER_STRENGTH,
            ids::CEQ_POSTERIZE_DITHER_GRAIN,
        ] {
            assert!(store.slider(id).is_some(), "slider {id:?} missing");
        }
        for id in [
            ids::CEQ_CLIP_LIMIT_NUM,
            ids::CEQ_TILE_GRID_NUM,
            ids::CEQ_EXPOSURE_NUM,
            ids::CEQ_TEMPERATURE_NUM,
            ids::CEQ_TINT_NUM,
            ids::CEQ_BRIGHTNESS_NUM,
            ids::CEQ_CONTRAST_NUM,
            ids::CEQ_VIBRANCE_NUM,
            ids::CEQ_SATURATION_NUM,
            ids::CEQ_SHARPEN_AMOUNT_NUM,
            ids::CEQ_SHARPEN_RADIUS_NUM,
            ids::CEQ_LUT_INTENSITY_NUM,
            ids::CEQ_LUT_MIX_NUM,
            ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
        ] {
            assert!(store.number_value(id).is_some(), "chip {id:?} missing");
        }
    }

    #[test]
    fn populate_links_each_slider_to_its_chip() {
        let mut store = WidgetStore::with_capacity(48);
        populate(&mut store);
        for (slider, chip) in [
            (ids::CEQ_CLIP_LIMIT, ids::CEQ_CLIP_LIMIT_NUM),
            (ids::CEQ_TILE_GRID, ids::CEQ_TILE_GRID_NUM),
            (ids::CEQ_EXPOSURE, ids::CEQ_EXPOSURE_NUM),
            (ids::CEQ_TEMPERATURE, ids::CEQ_TEMPERATURE_NUM),
            (ids::CEQ_TINT, ids::CEQ_TINT_NUM),
            (ids::CEQ_BRIGHTNESS, ids::CEQ_BRIGHTNESS_NUM),
            (ids::CEQ_CONTRAST, ids::CEQ_CONTRAST_NUM),
            (ids::CEQ_VIBRANCE, ids::CEQ_VIBRANCE_NUM),
            (ids::CEQ_SATURATION, ids::CEQ_SATURATION_NUM),
            (ids::CEQ_SHARPEN_AMOUNT, ids::CEQ_SHARPEN_AMOUNT_NUM),
            (ids::CEQ_SHARPEN_RADIUS, ids::CEQ_SHARPEN_RADIUS_NUM),
            (ids::CEQ_LUT_INTENSITY, ids::CEQ_LUT_INTENSITY_NUM),
            (ids::CEQ_LUT_MIX, ids::CEQ_LUT_MIX_NUM),
            (
                ids::CEQ_POSTERIZE_DITHER_STRENGTH,
                ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            ),
            (
                ids::CEQ_POSTERIZE_DITHER_GRAIN,
                ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            ),
        ] {
            assert_eq!(store.linked_number(slider), Some(chip));
            assert_eq!(store.linked_slider(chip), Some(slider));
        }
    }
}
