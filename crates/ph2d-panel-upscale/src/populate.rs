//! Upscale panel `populate` — pre-registers the panel's widget slots
//! in the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Widgets:
//! - 3 segmented buttons for the algorithm selector
//!   (Lanczos3 / Nearest / xBR).
//! - 1 slider + 1 NumberInput chip for the scale factor (linked via
//!   manual mirroring in [`crate::event`] — like the Padding panel —
//!   because the slider track is normalized `0..1` whereas the chip
//!   shows the raw factor `1.0..16.0`).
//! - Cancel + Apply buttons.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
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
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }

    // Scale slider — seeded to the default factor's normalized track.
    store.register(
        ids::UPS_SCALE,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: scale_to_slider(DEFAULT_SCALE_FACTOR),
            orientation: SliderOrientation::Horizontal,
        },
    );
    // Scale number chip — raw scale factor as f64; seeded to default.
    store.register(
        ids::UPS_SCALE_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: DEFAULT_SCALE_FACTOR as f64,
            buffer: format!("{DEFAULT_SCALE_FACTOR:.1}"),
            caret: 0,
            last_committed: DEFAULT_SCALE_FACTOR as f64,
            selection_anchor: None,
        },
    );
    // Pill chip via `paint_number_chip` (no stepper arrows) — kill the
    // dispatch's default phantom stepper carve. See DIRETRIZ §4.2.
    store.mark_chip_no_stepper(ids::UPS_SCALE_NUM);
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
    fn scale_slider_seeded_to_default_factor() {
        let mut store = WidgetStore::with_capacity(8);
        populate(&mut store);
        let (_, v) = store.slider(ids::UPS_SCALE).unwrap();
        let expected = scale_to_slider(DEFAULT_SCALE_FACTOR);
        assert!((v - expected).abs() < f32::EPSILON);
    }
}
