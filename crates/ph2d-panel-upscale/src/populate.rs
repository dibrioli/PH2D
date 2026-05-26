//! Upscale panel `populate` — pre-registers the panel's widget slots
//! in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout:
//! - 3 segmented buttons for the algorithm selector
//!   (Lanczos3 / Nearest / xBR).
//! - 1 slider + 1 NumberInput chip for the scale factor. Slider stores
//!   the normalized track in `0..1`; chip stores the **natural scale
//!   factor** in `[1.0, SCALE_FULL_SCALE]`. The pair is NOT wired via
//!   `link_slider_number` — that helper couples both widgets in the
//!   same `0..1` space, but the user must be able to type "2" and get
//!   2× (not track=2.0 → clamp 1.0 → 16×). Mirror is manual in
//!   [`crate::event`] (slider drag → chip factor; chip commit → slider
//!   track). Mirrors the Padding panel's chip-in-natural-unit pattern.
//! - Cancel + Apply buttons.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_upscale::params::{DEFAULT_SCALE_FACTOR, scale_to_slider};

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

    // Scale slider in track space (0..1) + scale chip in natural unit
    // (factor in [1.0, SCALE_FULL_SCALE]). The pair is NOT linked via
    // `link_slider_number`; `crate::event` mirrors them manually so the
    // user can type "2" → 2× (not "0.067" for track).
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
    fn scale_pair_is_not_linked() {
        // Manual mirror in `crate::event` — see populate module doc.
        // Linking would force chip and slider into the same 0..1 space
        // and break "type 2 → 2×".
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        assert_eq!(store.linked_number(ids::UPS_SCALE), None);
        assert_eq!(store.linked_slider(ids::UPS_SCALE_NUM), None);
    }
}
