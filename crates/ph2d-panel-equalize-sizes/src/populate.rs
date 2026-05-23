//! Equalize Sizes panel `populate` — pre-registers every widget slot in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout (Widget Gallery convention, DIRETRIZ §4.2):
//! - **3 mode buttons** (Max / Fixed / Grid), **3 algorithm buttons**
//!   (Lanczos / Nearest / xBR), **2 toggle buttons**
//!   (upscale-if-smaller / rasterize-after), **Cancel / Apply** — all
//!   plain buttons.
//! - **Fixed-mode W/H chips** (`EQS_FIXED_W`, `EQS_FIXED_H`) — standalone
//!   `NumberInput` widgets storing pixels (natural unit, no slider
//!   pairing) and explicitly marked `mark_chip_no_stepper` so the
//!   dispatch's default phantom-stepper carve doesn't fire on their
//!   right edge.
//! - **Grid-unit slider + chip** (`EQS_GRID_UNIT` + `EQS_GRID_UNIT_NUM`)
//!   — paired via `link_slider_number`. Storage for BOTH widgets is the
//!   normalized track in `0..1` (the slider's native space); the chip's
//!   natural-unit "px" label is paint-only via `display_override`.
//!   `link_slider_number` auto-marks the chip as no-stepper too — no
//!   need to call `mark_chip_no_stepper` for `EQS_GRID_UNIT_NUM`
//!   separately.
//!
//! This mirrors the canonical "Speed slider" setup from the widget
//! gallery (`pre_populate.rs`) — chip and slider share storage so the
//! dispatch handles drag / clamp / mirror automatically, and
//! [`crate::event`] stays a thin forwarder. See DIRETRIZ §4.2 + the
//! Color Equalization slot 1 incident report for why divergence here
//! re-creates the 4 bugs in `docs/UI_Bugs/README.md §11`.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
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

    // Fixed-mode W/H chips — standalone (no slider pair). Storage in
    // pixels (natural unit). Explicit `mark_chip_no_stepper` because
    // there's no `link_slider_number` here to auto-mark.
    for (chip_id, default_px) in [
        (ids::EQS_FIXED_W, defaults.fixed_w),
        (ids::EQS_FIXED_H, defaults.fixed_h),
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
        store.mark_chip_no_stepper(chip_id);
    }

    // Grid-unit slider + chip — paired via `link_slider_number`. Storage
    // for both widgets is the normalized track in 0..1 (mirror of the
    // Speed slider in the widget gallery). The chip's displayed "px"
    // label comes from `display_override` in paint.rs, computed live
    // from the slider track via `slider_to_grid_unit`.
    let track = grid_unit_to_slider(params_default.grid_unit);
    store.register(
        ids::EQS_GRID_UNIT,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::EQS_GRID_UNIT_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: track as f64,
            buffer: format_number(track as f64),
            caret: 0,
            last_committed: track as f64,
            selection_anchor: None,
        },
    );
    // `link_slider_number` auto-marks the chip as no-stepper too (see
    // its doc) so the dispatch's default 16-22 px stepper carve on the
    // chip's right edge doesn't arm a phantom continuous-hold.
    store.link_slider_number(ids::EQS_GRID_UNIT, ids::EQS_GRID_UNIT_NUM);
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
    fn fixed_chips_seed_in_pixels() {
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
    }

    #[test]
    fn grid_unit_pair_seeds_in_track_space() {
        // Both slider and chip storage live in 0..1 (track), not in
        // pixels — Widget Gallery convention §4.2.
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        let params_default = EqualizeSizesParams::default();
        let expected_track = grid_unit_to_slider(params_default.grid_unit);
        let (_, slider_val) = store.slider(ids::EQS_GRID_UNIT).unwrap();
        assert!((slider_val - expected_track).abs() < 1e-6);
        let chip_val = store.number_value(ids::EQS_GRID_UNIT_NUM).unwrap();
        assert!((chip_val as f32 - expected_track).abs() < 1e-6);
    }

    #[test]
    fn grid_unit_pair_is_linked() {
        // `link_slider_number` engages the dispatch's bidirectional
        // mirror + clamp + auto no-stepper marking. Without it the
        // panel re-creates the 4 bugs from slot 1 (Color Equalization).
        let mut store = WidgetStore::with_capacity(32);
        populate(&mut store);
        assert_eq!(
            store.linked_number(ids::EQS_GRID_UNIT),
            Some(ids::EQS_GRID_UNIT_NUM)
        );
        assert_eq!(
            store.linked_slider(ids::EQS_GRID_UNIT_NUM),
            Some(ids::EQS_GRID_UNIT)
        );
    }
}
