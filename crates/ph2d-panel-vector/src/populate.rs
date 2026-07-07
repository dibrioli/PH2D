//! Vector Style panel widget registration (called once at panel install).
//!
//! Registers the Width slider (seeded at the tool's default so the knob renders
//! correctly before the first drag — the slider is absolute-position, so the
//! initial store value drives both the render AND the drag baseline), its px
//! chip (linked via the shared affine mapping), the draw-mode buttons
//! (Pen / Rectangle / Ellipse / Polygon), the Polygon Sides slider + chip, the
//! three Boolean buttons, the Fill "None" button, and the Close (X) button. The
//! two colour swatches need NO store entry — their Down is handled by the
//! generic `is_picker_swatch` dispatch (pointer.rs), which short-circuits before
//! the normal widget-event path.

use crate::ids;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{ButtonState, SliderOrientation, SliderState, TextInputState};
use ph2d_tool_vector::params::{
    SIDES_SLIDER_OFFSET, SIDES_SLIDER_SCALE, WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE,
    sides_to_slider,
};
use ph2d_tool_vector::{DEFAULT_POLYGON_SIDES, DEFAULT_STROKE_WIDTH_PX, px_to_slider};

/// Register a plain action Button in the Normal state.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

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

    // Draw-mode segmented buttons (Pen / Rectangle / Ellipse / Polygon).
    button(store, ids::VECTOR_MODE_PEN);
    button(store, ids::VECTOR_MODE_RECT);
    button(store, ids::VECTOR_MODE_ELLIPSE);
    button(store, ids::VECTOR_MODE_POLYGON);

    // Polygon Sides slider — seeded at the tool's default (`sides_to_slider(5)`).
    // Registered unconditionally (the store is mode-agnostic); the panel only
    // paints/hit-registers it in Polygon mode.
    store.register(
        ids::VECTOR_SIDES,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: sides_to_slider(DEFAULT_POLYGON_SIDES),
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        ids::VECTOR_SIDES_NUM,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(DEFAULT_POLYGON_SIDES),
            buffer: format!("{DEFAULT_POLYGON_SIDES}"),
            caret: 0,
            last_committed: f64::from(DEFAULT_POLYGON_SIDES),
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(
        ids::VECTOR_SIDES,
        ids::VECTOR_SIDES_NUM,
        SIDES_SLIDER_SCALE,
        SIDES_SLIDER_OFFSET,
    );

    // Vertex-type buttons (retype the selected vertex; shown only when a vertex
    // is selected, but registered unconditionally — the store is mode-agnostic)
    // + the Delete-node button.
    button(store, ids::VECTOR_VERT_CORNER);
    button(store, ids::VECTOR_VERT_SMOOTH);
    button(store, ids::VECTOR_VERT_SYMMETRIC);
    button(store, ids::VECTOR_VERT_DELETE);

    // Boolean op buttons (act on the two last closed regions of the document).
    button(store, ids::VECTOR_BOOL_UNION);
    button(store, ids::VECTOR_BOOL_SUBTRACT);
    button(store, ids::VECTOR_BOOL_INTERSECT);

    // Fill "None" button (clears the fill on the selected closed path).
    button(store, ids::VECTOR_FILL_NONE);
    // Close (X) button.
    button(store, ids::VECTOR_CLOSE);
}
