//! Sculpt-section widget registration (`docs/Painter/18…`, Waves 1-2) — split from [`crate::populate`] for
//! the panel file-LOC cap. Registers the Radius and Offset sliders (each with its linked editable chip) and
//! the five sub-mode segments, so panel-wiring-parity, the dispatcher AND the pointer see them.
//!
//! **Both sliders are registered even though the card only ever PAINTS one of them** (the row swaps with the
//! verb's family). Registration is wiring, and wiring that is reachable in only one mode is still wiring
//! that can be dead — so the seam sweep drives them both.
//!
//! The segments are registered as **`Button`**, not `Checkbox`, and that is not a stylistic choice: a
//! `Checkbox` emits `Toggled`, which `event.rs` does not forward — the widget would be registered and
//! still dead, which is the second-order version of the same bug (mirror of `populate_deform`).

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};

fn register_slider_with_chip(store: &mut WidgetStore, slider: NodeId, chip: NodeId) {
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
}

pub(crate) fn register_sculpt_widgets(store: &mut WidgetStore) {
    let radius = ph2d_editor_core::ids::PAINTER_SCULPT_RADIUS_SLIDER;
    let radius_chip = ph2d_editor_core::ids::PAINTER_SCULPT_RADIUS_CHIP;
    register_slider_with_chip(store, radius, radius_chip);
    // The chip speaks **px**, not the raw track: `track·15 + 1` is exactly `SculptState::radius_px`, so
    // typing `12` into the chip lands the slider where 12 px is, and the number on screen can never drift
    // from the number in the blur. (`_integer` because a blur radius is a whole number of texels.)
    store.link_slider_number_mapped_integer(radius, radius_chip, 15.0, 1.0); // LITERAL-PX-OK: 0..1 → 1..16 px
    store.set_number_range(radius_chip, 1.0, 16.0, 1.0); // LITERAL-PX-OK: the kernel's radius range, in px

    let offset = ph2d_editor_core::ids::PAINTER_SCULPT_OFFSET_SLIDER;
    let offset_chip = ph2d_editor_core::ids::PAINTER_SCULPT_OFFSET_CHIP;
    register_slider_with_chip(store, offset, offset_chip);
    // …and this chip speaks **paint-loads**: `track·2 − 1` is exactly `SculptState::plane_offset` (with
    // `PLANE_OFFSET_MAX = 1`). NOT `_integer` — the plane slides continuously, and rounding it to whole
    // loads would leave the slider with three usable positions.
    store.link_slider_number_mapped(offset, offset_chip, 2.0, -1.0); // LITERAL-PX-OK: 0..1 → −1..+1 loads
    store.set_number_range(offset_chip, -1.0, 1.0, 0.05); // LITERAL-PX-OK: the plane's travel, in loads

    for id in ph2d_editor_core::ids::PAINTER_SCULPT_MODE_IDS {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}
