//! Equalize Sizes panel `populate` — pre-registers every widget slot in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Most controls are **buttons**: the 3 target-mode radio buttons, the 3
//! upscale-algorithm radio buttons, the 2 boolean toggles
//! (upscale-if-smaller, rasterize-after), and Cancel / Apply. The
//! fixed-mode W/H + grid-unit chip are **NumberInput** widgets. The
//! grid-unit slider is a unipolar `Slider` (track `0..1`). The slider ↔
//! chip mirror for grid-unit is kept in sync manually in [`crate::event`]
//! (slider drag → chip px; chip type → slider track) — they are NOT
//! `link_slider_number`-coupled because the chip carries pixels while the
//! slider is normalized.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_equalize_sizes::params::{
    EqualizeSizesParams, EqualizeSizesUiSnapshot, grid_unit_to_slider,
};

pub fn populate(store: &mut WidgetStore) {
    // Every plain button (modes, algorithm, toggles, Cancel/Apply).
    for id in [
        ids::EQS_MODE_MAX,
        ids::EQS_MODE_FIXED,
        ids::EQS_MODE_GRID,
        ids::EQS_UPSCALE_IF_SMALLER,
        ids::EQS_RASTERIZE_AFTER,
        ids::EQS_ALG_LANCZOS,
        ids::EQS_ALG_NEAREST,
        ids::EQS_ALG_XBR,
        ids::EQS_CANCEL,
        ids::EQS_APPLY,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Seed defaults so the painted UI matches `EqualizeSizesParams::default`
    // before the first snapshot push from the host (no flicker).
    let defaults = EqualizeSizesUiSnapshot::default();
    let params_default = EqualizeSizesParams::default();

    for (chip_id, default_px) in [
        (ids::EQS_FIXED_W, defaults.fixed_w),
        (ids::EQS_FIXED_H, defaults.fixed_h),
        (ids::EQS_GRID_UNIT_NUM, defaults.grid_unit),
    ] {
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: default_px as f64,
                buffer: default_px.to_string(),
                caret: 0,
                last_committed: default_px as f64,
                selection_anchor: None,
            },
        );
        // Pill chip via `paint_number_chip` (no stepper arrows) — kill
        // the dispatch's default phantom stepper carve. See DIRETRIZ §4.2.
        store.mark_chip_no_stepper(chip_id);
    }

    // Grid-unit slider: unipolar, seeded to the default grid_unit's
    // normalized position.
    store.register(
        ids::EQS_GRID_UNIT,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: grid_unit_to_slider(params_default.grid_unit),
            orientation: SliderOrientation::Horizontal,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        for id in [
            ids::EQS_MODE_MAX,
            ids::EQS_MODE_FIXED,
            ids::EQS_MODE_GRID,
            ids::EQS_UPSCALE_IF_SMALLER,
            ids::EQS_RASTERIZE_AFTER,
            ids::EQS_ALG_LANCZOS,
            ids::EQS_ALG_NEAREST,
            ids::EQS_ALG_XBR,
            ids::EQS_CANCEL,
            ids::EQS_APPLY,
        ] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [ids::EQS_FIXED_W, ids::EQS_FIXED_H, ids::EQS_GRID_UNIT_NUM] {
            assert!(
                store.number_value(id).is_some(),
                "number chip {id:?} missing"
            );
        }
        assert!(store.slider(ids::EQS_GRID_UNIT).is_some());
    }

    #[test]
    fn default_seeds_match_params_default() {
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        let defaults = EqualizeSizesUiSnapshot::default();
        assert_eq!(
            store.number_value(ids::EQS_FIXED_W),
            Some(defaults.fixed_w as f64)
        );
        assert_eq!(
            store.number_value(ids::EQS_FIXED_H),
            Some(defaults.fixed_h as f64)
        );
        assert_eq!(
            store.number_value(ids::EQS_GRID_UNIT_NUM),
            Some(defaults.grid_unit as f64)
        );
    }
}
