//! Sculpt-section widget registration (`docs/Painter/18…`, Wave 1) — split from [`crate::populate`] for
//! the panel file-LOC cap. Registers the Radius slider (with its linked editable chip) and the
//! Smooth / Sharpen sub-mode segments, so panel-wiring-parity, the dispatcher AND the pointer see them.
//!
//! The segments are registered as **`Button`**, not `Checkbox`, and that is not a stylistic choice: a
//! `Checkbox` emits `Toggled`, which `event.rs` does not forward — the widget would be registered and
//! still dead, which is the second-order version of the same bug (mirror of `populate_deform`).

use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

pub(crate) fn register_sculpt_widgets(store: &mut WidgetStore) {
    let slider = ph2d_editor_core::ids::PAINTER_SCULPT_RADIUS_SLIDER;
    let chip = ph2d_editor_core::ids::PAINTER_SCULPT_RADIUS_CHIP;
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: 0.0,
            buffer: String::new(),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
    // The chip speaks **px**, not the raw track: `track·15 + 1` is exactly `SculptState::radius_px`, so
    // typing `12` into the chip lands the slider where 12 px is, and the number on screen can never drift
    // from the number in the blur. (`_integer` because a blur radius is a whole number of texels.)
    store.link_slider_number_mapped_integer(slider, chip, 15.0, 1.0); // LITERAL-PX-OK: 0..1 → 1..16 px
    store.set_number_range(chip, 1.0, 16.0, 1.0); // LITERAL-PX-OK: the kernel's radius range, in px

    for id in ph2d_editor_core::ids::PAINTER_SCULPT_MODE_IDS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
