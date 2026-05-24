//! Upscale panel `populate` — pre-registers the panel's widget slots
//! in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Layout (Widget Gallery convention, DIRETRIZ §4.2):
//! - 3 segmented buttons for the algorithm selector
//!   (Lanczos3 / Nearest / xBR).
//! - 1 slider + 1 NumberInput chip for the scale factor, **paired
//!   via `link_slider_number`**. Both widgets store the normalized
//!   track in `0..1`; the chip's natural-unit display ("2.00×") is
//!   paint-only via `display_override`. The dispatch handles
//!   drag / clamp / chip↔slider mirror for free — no manual mirror.
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

    // Scale slider + chip — both stored in the SAME `0..1` track space
    // (Widget Gallery convention §4.2). The chip's natural-unit display
    // ("2.00×") is computed in paint via `slider_to_scale(track)`.
    let track = scale_to_slider(DEFAULT_SCALE_FACTOR);
    store.register(
        ids::UPS_SCALE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::UPS_SCALE_NUM,
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
    // its doc) so the dispatch's default stepper carve on the chip's
    // right edge doesn't arm a phantom continuous-hold.
    store.link_slider_number(ids::UPS_SCALE, ids::UPS_SCALE_NUM);

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
    fn scale_slider_and_chip_seeded_to_default_track() {
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        let expected = scale_to_slider(DEFAULT_SCALE_FACTOR);
        let (_, v) = store.slider(ids::UPS_SCALE).unwrap();
        assert!((v - expected).abs() < f32::EPSILON);
        // Chip storage is ALSO in track space (Widget Gallery §4.2).
        let chip_v = store.number_value(ids::UPS_SCALE_NUM).unwrap() as f32;
        assert!((chip_v - expected).abs() < f32::EPSILON);
    }

    #[test]
    fn scale_pair_is_linked() {
        // `link_slider_number` engages the dispatch's bidirectional
        // mirror + clamp + auto no-stepper marking — without it the
        // panel re-creates the slot 1 (Color Equalization) bugs.
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        assert_eq!(
            store.linked_number(ids::UPS_SCALE),
            Some(ids::UPS_SCALE_NUM)
        );
        assert_eq!(
            store.linked_slider(ids::UPS_SCALE_NUM),
            Some(ids::UPS_SCALE)
        );
    }
}
