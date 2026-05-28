//! Padding panel `populate` — pre-registers the panel's widget slots in
//! the `WidgetStore` at host boot (once, via `Panel::populate`).
//!
//! Each edge is a bipolar **slider** (`PAD_*`, normalized `0..1` with
//! `0.5` = neutral) plus a px-valued **NumberInput chip** (`PAD_*_NUM`).
//! Wired via [`WidgetStore::link_slider_number_mapped`] with the affine
//! projection `px = track * (2 * FULL_SCALE) + (-FULL_SCALE)`, so:
//!
//! - the chip's stored value is **always pixels** (matches the visible
//!   display + what the user types — no surprise transform on Enter),
//! - the slider's stored value stays in `0..1` for compatibility with
//!   the canonical slider widget,
//! - the dispatch handles drag-scrub clamp, slider-drag mirror, and
//!   keyboard commit round-trip automatically (the 2026-05-27 fix
//!   class — `commit_number_buffer` inverse-projects through the
//!   registered mapping).
//!
//! Pre-2026-05-27 the mirror was manual in `event.rs`; with the mapped
//! link, `event.rs` only has to forward `PanelEvent::SetValue` on
//! `ValueChanged(slider)` (the dispatch mirror keeps chip+slider in
//! lockstep on its own).

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_padding::params::PAD_SLIDER_FULL_SCALE;

pub fn populate(store: &mut WidgetStore) {
    // Cancel / Apply + the pivot-mode toggle (a Button painted as an
    // accent toggle, like Bg Removal's Show-Mask).
    for id in [
        ids::PAD_CANCEL,
        ids::PAD_APPLY,
        ids::PAD_RESET,
        ids::PAD_PIVOT_RECENTER,
    ] {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
    // Four bipolar sliders (seeded neutral = 0.5 = 0 px) + their px
    // chips (seeded 0 px). Mapping: `px = track*1024 - 512`, i.e.
    // `(scale = 2*FULL_SCALE, offset = -FULL_SCALE)` — full slider
    // sweep maps to ±FULL_SCALE px symmetrically around the neutral
    // centre. See [`ph2d_tool_padding::params::px_to_slider`] /
    // `slider_to_px` for the canonical projection (rounding is the
    // tool's job; this affine map keeps the round-trip exact for
    // integer-valued px).
    let full_scale = PAD_SLIDER_FULL_SCALE as f32;
    let scale = 2.0 * full_scale;
    let offset = -full_scale;
    for (slider_id, chip_id) in [
        (ids::PAD_TOP, ids::PAD_TOP_NUM),
        (ids::PAD_RIGHT, ids::PAD_RIGHT_NUM),
        (ids::PAD_BOTTOM, ids::PAD_BOTTOM_NUM),
        (ids::PAD_LEFT, ids::PAD_LEFT_NUM),
    ] {
        store.register(
            slider_id,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        store.register(
            chip_id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: "0".to_string(),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        store.link_slider_number_mapped(slider_id, chip_id, scale, offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn populate_registers_all_controls() {
        let mut store = WidgetStore::with_capacity(16);
        populate(&mut store);
        for id in [
            ids::PAD_CANCEL,
            ids::PAD_APPLY,
            ids::PAD_RESET,
            ids::PAD_PIVOT_RECENTER,
        ] {
            assert!(store.button_state(id).is_some(), "button {id:?} missing");
        }
        for id in [ids::PAD_TOP, ids::PAD_RIGHT, ids::PAD_BOTTOM, ids::PAD_LEFT] {
            assert!(store.slider(id).is_some(), "slider {id:?} missing");
        }
        for id in [
            ids::PAD_TOP_NUM,
            ids::PAD_RIGHT_NUM,
            ids::PAD_BOTTOM_NUM,
            ids::PAD_LEFT_NUM,
        ] {
            assert!(store.number_value(id).is_some(), "chip {id:?} missing");
        }
    }
}
