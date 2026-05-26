//! Color Equalization panel `populate` — pre-registers the panel's widget
//! slots in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Mirrors the canonical "slider Speed" setup from the widget gallery
//! (`pre_populate.rs`): chip + slider share the same `0..1` value space,
//! `link_slider_number` registers the bidirectional mirror, and the
//! dispatch handles drag / clamp / sync automatically. Natural-unit
//! formatting ("2.00" clip limit, "+0.30" brightness etc.) is paint-only
//! via `display_override` and lives outside the storage.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{
    ButtonState, DropdownState, SliderOrientation, SliderState, TextInputState,
};
use ph2d_tool_color_equalization::params::{
    BRIGHTNESS_DEFAULT, CLIP_LIMIT_DEFAULT, CONTRAST_DEFAULT, DENOISE_STRENGTH_DEFAULT,
    EXPOSURE_DEFAULT, LUT_INTENSITY_DEFAULT, LUT_MIX_DEFAULT, POSTERIZE_DITHER_GRAIN_DEFAULT,
    POSTERIZE_DITHER_STRENGTH_DEFAULT, SATURATION_DEFAULT, SHARPEN_AMOUNT_DEFAULT,
    SHARPEN_RADIUS_DEFAULT, TEMPERATURE_DEFAULT, TILE_GRID_DEFAULT, TINT_DEFAULT, VIBRANCE_DEFAULT,
    brightness_to_slider, clip_limit_to_slider, contrast_to_slider, denoise_strength_to_slider,
    exposure_to_slider, lut_intensity_to_slider, lut_mix_to_slider,
    posterize_dither_grain_to_slider, posterize_dither_strength_to_slider, saturation_to_slider,
    sharpen_amount_to_slider, sharpen_radius_to_slider, temperature_to_slider, tile_grid_to_slider,
    tint_to_slider, vibrance_to_slider,
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

    let rows: [(_, _, f32); 16] = [
        (
            ids::CEQ_CLIP_LIMIT,
            ids::CEQ_CLIP_LIMIT_NUM,
            clip_limit_to_slider(CLIP_LIMIT_DEFAULT),
        ),
        (
            ids::CEQ_TILE_GRID,
            ids::CEQ_TILE_GRID_NUM,
            tile_grid_to_slider(TILE_GRID_DEFAULT),
        ),
        (
            ids::CEQ_EXPOSURE,
            ids::CEQ_EXPOSURE_NUM,
            exposure_to_slider(EXPOSURE_DEFAULT),
        ),
        (
            ids::CEQ_TEMPERATURE,
            ids::CEQ_TEMPERATURE_NUM,
            temperature_to_slider(TEMPERATURE_DEFAULT),
        ),
        (
            ids::CEQ_TINT,
            ids::CEQ_TINT_NUM,
            tint_to_slider(TINT_DEFAULT),
        ),
        (
            ids::CEQ_BRIGHTNESS,
            ids::CEQ_BRIGHTNESS_NUM,
            brightness_to_slider(BRIGHTNESS_DEFAULT),
        ),
        (
            ids::CEQ_CONTRAST,
            ids::CEQ_CONTRAST_NUM,
            contrast_to_slider(CONTRAST_DEFAULT),
        ),
        (
            ids::CEQ_VIBRANCE,
            ids::CEQ_VIBRANCE_NUM,
            vibrance_to_slider(VIBRANCE_DEFAULT),
        ),
        (
            ids::CEQ_SATURATION,
            ids::CEQ_SATURATION_NUM,
            saturation_to_slider(SATURATION_DEFAULT),
        ),
        (
            ids::CEQ_SHARPEN_AMOUNT,
            ids::CEQ_SHARPEN_AMOUNT_NUM,
            sharpen_amount_to_slider(SHARPEN_AMOUNT_DEFAULT),
        ),
        (
            ids::CEQ_SHARPEN_RADIUS,
            ids::CEQ_SHARPEN_RADIUS_NUM,
            sharpen_radius_to_slider(SHARPEN_RADIUS_DEFAULT),
        ),
        (
            ids::CEQ_DENOISE_STRENGTH,
            ids::CEQ_DENOISE_STRENGTH_NUM,
            denoise_strength_to_slider(DENOISE_STRENGTH_DEFAULT),
        ),
        (
            ids::CEQ_LUT_INTENSITY,
            ids::CEQ_LUT_INTENSITY_NUM,
            lut_intensity_to_slider(LUT_INTENSITY_DEFAULT),
        ),
        (
            ids::CEQ_LUT_MIX,
            ids::CEQ_LUT_MIX_NUM,
            lut_mix_to_slider(LUT_MIX_DEFAULT),
        ),
        (
            ids::CEQ_POSTERIZE_DITHER_STRENGTH,
            ids::CEQ_POSTERIZE_DITHER_STRENGTH_NUM,
            posterize_dither_strength_to_slider(POSTERIZE_DITHER_STRENGTH_DEFAULT),
        ),
        (
            ids::CEQ_POSTERIZE_DITHER_GRAIN,
            ids::CEQ_POSTERIZE_DITHER_GRAIN_NUM,
            posterize_dither_grain_to_slider(POSTERIZE_DITHER_GRAIN_DEFAULT),
        ),
    ];
    for (slider_id, chip_id, track) in rows {
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
                value: track as f64,
                buffer: format_number(track as f64),
                caret: 0,
                last_committed: track as f64,
                selection_anchor: None,
            },
        );
        // `link_slider_number` auto-marks `chip_id` as no-stepper too
        // (see its doc) so the chip's right edge doesn't arm a phantom
        // continuous-hold from the dispatch's default stepper carve.
        store.link_slider_number(slider_id, chip_id);
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
            ids::CEQ_DENOISE_STRENGTH,
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
            ids::CEQ_DENOISE_STRENGTH_NUM,
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
            (ids::CEQ_DENOISE_STRENGTH, ids::CEQ_DENOISE_STRENGTH_NUM),
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
