//! Vector Style panel widget registration (called once at panel install).
//!
//! Registers the Width slider (seeded at the tool's default so the knob renders
//! correctly before the first drag — the slider is absolute-position, so the
//! initial store value drives both the render AND the drag baseline), its px
//! chip (linked via the shared affine mapping), the Fill "None" button, and the
//! Close (X) button. The two colour swatches need NO store entry — their Down is
//! handled by the generic `is_picker_swatch` dispatch (pointer.rs), which
//! short-circuits before the normal widget-event path.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_vector::params::{WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE};
use ph2d_tool_vector::{DEFAULT_STROKE_WIDTH_PX, px_to_slider};

pub fn populate(store: &mut WidgetStore) {
    // Width slider — seeded at the tool's default (`px_to_slider(3px)`).
    store.register(
        ids::VECTOR_WIDTH,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: px_to_slider(DEFAULT_STROKE_WIDTH_PX),
            orientation: SliderOrientation::Horizontal,
        },
    );
    // Px chip paired with the Width slider.
    store.register(
        ids::VECTOR_WIDTH_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: DEFAULT_STROKE_WIDTH_PX,
            buffer: format!("{}", DEFAULT_STROKE_WIDTH_PX as i64),
            caret: 0,
            last_committed: DEFAULT_STROKE_WIDTH_PX,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(
        ids::VECTOR_WIDTH,
        ids::VECTOR_WIDTH_NUM,
        WIDTH_SLIDER_SCALE,
        WIDTH_SLIDER_OFFSET,
    );

    // Fill "None" button (clears the fill on the selected closed path).
    store.register(
        ids::VECTOR_FILL_NONE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
    // Close (X) button.
    store.register(
        ids::VECTOR_CLOSE,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}
