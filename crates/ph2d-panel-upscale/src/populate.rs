//! Upscale panel `populate` — pre-registers the panel's widget slots
//! in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout:
//! - 3 segmented buttons for the algorithm selector
//!   (Lanczos3 / Nearest / xBR).
//! - 1 slider + 1 NumberInput chip for the scale factor, wired via
//!   [`WidgetStore::link_slider_number_mapped`] with the affine
//!   projection `factor = track*(MAX-MIN) + MIN`. Chip stores the
//!   **natural factor** in `[MIN_SCALE_FACTOR, SCALE_FULL_SCALE]`,
//!   slider keeps `0..1` for canonical widget compatibility. Dispatch
//!   handles clamp (out-of-range typed factors snap back via
//!   `apply_chip_value_with_mirror`'s re-sync), drag-scrub bounds, and
//!   commit round-trip — no manual mirror in `event.rs` anymore.
//! - Cancel + Apply buttons.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_upscale::params::{
    DEFAULT_SCALE_FACTOR, MIN_SCALE_FACTOR, SCALE_FULL_SCALE, scale_to_slider,
};

pub fn populate(store: &mut WidgetStore) {
    // Algorithm segmented buttons + Cancel + Apply: five Buttons.
    for id in [
        ids::UPS_ALGO_LANCZOS3,
        ids::UPS_ALGO_NEAREST,
        ids::UPS_ALGO_XBR,
        ids::UPS_APPLY,
        ids::UPS_CANCEL,
        ids::UPS_RESET,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Scale slider in track space (0..1) + scale chip in natural
    // factor (in [MIN_SCALE_FACTOR, SCALE_FULL_SCALE]). Mapping:
    // `factor = track * (MAX-MIN) + MIN`, so the dispatch projects
    // typed factors back into `0..1` storage and snaps out-of-range
    // inputs (e.g. "999") to the bound via the canonical re-sync in
    // `apply_chip_value_with_mirror`.
    let track = scale_to_slider(DEFAULT_SCALE_FACTOR);
    store.register(
        ids::UPS_SCALE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let factor = DEFAULT_SCALE_FACTOR as f64;
    store.register(
        ids::UPS_SCALE_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: factor,
            buffer: format_number(factor),
            caret: 0,
            last_committed: factor,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(
        ids::UPS_SCALE,
        ids::UPS_SCALE_NUM,
        SCALE_FULL_SCALE - MIN_SCALE_FACTOR,
        MIN_SCALE_FACTOR,
    );

    // Hover tooltips for the 3 algorithm chips so the user knows which
    // algorithm suits which image kind. Each text fits in a single line
    // at the canonical tooltip width; no out-of-font glyphs (use ASCII
    // and `\u{00b7}` middot only — see no_tofu_glyphs gate).
    store.set_tooltip(
        ids::UPS_ALGO_LANCZOS3,
        "Lanczos3 \u{00b7} smooth gradients \u{00b7} photos / illustrations \u{00b7} default",
    );
    store.set_tooltip(
        ids::UPS_ALGO_NEAREST,
        "Nearest \u{00b7} keeps hard pixel edges \u{00b7} pixel art / tile sprites",
    );
    store.set_tooltip(
        ids::UPS_ALGO_XBR,
        "xBR \u{00b7} edge-aware pixel-art upscale \u{00b7} 2x / 3x / 4x only",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        for id in [
            ids::UPS_ALGO_LANCZOS3,
            ids::UPS_ALGO_NEAREST,
            ids::UPS_ALGO_XBR,
            ids::UPS_APPLY,
            ids::UPS_CANCEL,
        ] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        assert!(store.slider(ids::UPS_SCALE).is_some(), "slider missing");
        assert!(
            store.number_value(ids::UPS_SCALE_NUM).is_some(),
            "scale chip missing"
        );
    }

    #[test]
    fn scale_slider_seeded_in_track_chip_in_natural_factor() {
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        // Slider lives in track space (0..1).
        let expected_track = scale_to_slider(DEFAULT_SCALE_FACTOR);
        let (_, v) = store.slider(ids::UPS_SCALE).unwrap();
        assert!((v - expected_track).abs() < f32::EPSILON);
        // Chip lives in natural unit (factor in [1, 16]).
        let chip_v = store.number_value(ids::UPS_SCALE_NUM).unwrap();
        assert!((chip_v - DEFAULT_SCALE_FACTOR as f64).abs() < f64::EPSILON);
    }

    #[test]
    fn scale_pair_is_mapped_linked() {
        // 2026-05-27: chip↔slider mirror is now in dispatch via
        // `link_slider_number_mapped(slider, chip, MAX-MIN, MIN)`.
        // Asserts BOTH the bidirectional link AND the affine mapping.
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        assert_eq!(store.linked_number(ids::UPS_SCALE), Some(ids::UPS_SCALE_NUM));
        assert_eq!(store.linked_slider(ids::UPS_SCALE_NUM), Some(ids::UPS_SCALE));
        let (scale, offset) = store.linked_slider_mapping(ids::UPS_SCALE_NUM);
        let expected_scale = SCALE_FULL_SCALE - MIN_SCALE_FACTOR;
        let expected_offset = MIN_SCALE_FACTOR;
        assert!(
            (scale - expected_scale).abs() < f32::EPSILON,
            "scale {scale} != {expected_scale}"
        );
        assert!(
            (offset - expected_offset).abs() < f32::EPSILON,
            "offset {offset} != {expected_offset}"
        );
    }
}
