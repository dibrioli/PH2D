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
    DASH_SLIDER_OFFSET, DASH_SLIDER_SCALE, GAP_DEFAULT, GAP_SLIDER_OFFSET, GAP_SLIDER_SCALE,
    OPACITY_SLIDER_OFFSET, OPACITY_SLIDER_SCALE, RADIUS_SLIDER_OFFSET, RADIUS_SLIDER_SCALE,
    SIDES_SLIDER_OFFSET, SIDES_SLIDER_SCALE, STAR_INNER_SLIDER_OFFSET, STAR_INNER_SLIDER_SCALE,
    STAR_POINTS_SLIDER_OFFSET, STAR_POINTS_SLIDER_SCALE, WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE,
    gap_to_slider, radius_to_slider, sides_to_slider, star_inner_to_slider, star_points_to_slider,
};
use ph2d_tool_vector::{
    DEFAULT_CORNER_RADIUS_PX, DEFAULT_POLYGON_SIDES, DEFAULT_STAR_INNER, DEFAULT_STAR_POINTS,
    DEFAULT_STROKE_WIDTH_PX, px_to_slider,
};

/// Register a plain action Button in the Normal state.
fn button(store: &mut WidgetStore, id: ph2d_a11y::NodeId) {
    store.register(
        id,
        InteractiveState::Button {
            state: ButtonState::Normal,
        },
    );
}

/// Register a slider + its linked value chip, seeded at `track` / `display`.
fn slider_chip(
    store: &mut WidgetStore,
    slider: ph2d_a11y::NodeId,
    chip: ph2d_a11y::NodeId,
    track: f32,
    display: f64,
    scale: f32,
    offset: f32,
) {
    store.register(
        slider,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: track,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        chip,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: display,
            buffer: format!("{display}"),
            caret: 0,
            last_committed: display,
            selection_anchor: None,
        },
    );
    store.link_slider_number_mapped(slider, chip, scale, offset);
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

    // Draw-mode segmented buttons (Pen / Rect / Oval / Poly / Star / Round).
    button(store, ids::VECTOR_MODE_PEN);
    button(store, ids::VECTOR_MODE_RECT);
    button(store, ids::VECTOR_MODE_ELLIPSE);
    button(store, ids::VECTOR_MODE_POLYGON);
    button(store, ids::VECTOR_MODE_STAR);
    button(store, ids::VECTOR_MODE_RRECT);

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

    // Star Points + Inner sliders (shown in Star mode) and RoundRect Radius
    // slider (shown in RoundRect mode) — seeded at the tool defaults.
    slider_chip(
        store,
        ids::VECTOR_STAR_POINTS,
        ids::VECTOR_STAR_POINTS_NUM,
        star_points_to_slider(DEFAULT_STAR_POINTS),
        f64::from(DEFAULT_STAR_POINTS),
        STAR_POINTS_SLIDER_SCALE,
        STAR_POINTS_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_STAR_INNER,
        ids::VECTOR_STAR_INNER_NUM,
        star_inner_to_slider(DEFAULT_STAR_INNER),
        DEFAULT_STAR_INNER,
        STAR_INNER_SLIDER_SCALE,
        STAR_INNER_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_RRECT_RADIUS,
        ids::VECTOR_RRECT_RADIUS_NUM,
        radius_to_slider(DEFAULT_CORNER_RADIUS_PX),
        DEFAULT_CORNER_RADIUS_PX,
        RADIUS_SLIDER_SCALE,
        RADIUS_SLIDER_OFFSET,
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

    // Stroke / Fill Opacity sliders (0..100 %) — seeded at full opacity, matching
    // the tool's default opaque stroke/fill.
    slider_chip(
        store,
        ids::VECTOR_STROKE_OPACITY,
        ids::VECTOR_STROKE_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_FILL_OPACITY,
        ids::VECTOR_FILL_OPACITY_NUM,
        1.0,
        100.0, // LITERAL-PX-OK: initial opacity display = 100 %
        OPACITY_SLIDER_SCALE,
        OPACITY_SLIDER_OFFSET,
    );

    // Stroke Cap / Join segmented buttons + Dash length slider (0 px = solid).
    button(store, ids::VECTOR_CAP_BUTT);
    button(store, ids::VECTOR_CAP_ROUND);
    button(store, ids::VECTOR_CAP_SQUARE);
    button(store, ids::VECTOR_JOIN_MITER);
    button(store, ids::VECTOR_JOIN_ROUND);
    button(store, ids::VECTOR_JOIN_BEVEL);
    slider_chip(
        store,
        ids::VECTOR_DASH,
        ids::VECTOR_DASH_NUM,
        0.0,
        0.0,
        DASH_SLIDER_SCALE,
        DASH_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_GAP,
        ids::VECTOR_GAP_NUM,
        gap_to_slider(GAP_DEFAULT),
        GAP_DEFAULT,
        GAP_SLIDER_SCALE,
        GAP_SLIDER_OFFSET,
    );

    // Arrange: Duplicate + z-order restack + Flip buttons (act on the selected path).
    button(store, ids::VECTOR_ARRANGE_DUPLICATE);
    button(store, ids::VECTOR_ARRANGE_TO_BACK);
    button(store, ids::VECTOR_ARRANGE_BACKWARD);
    button(store, ids::VECTOR_ARRANGE_FORWARD);
    button(store, ids::VECTOR_ARRANGE_TO_FRONT);
    button(store, ids::VECTOR_ARRANGE_FLIP_H);
    button(store, ids::VECTOR_ARRANGE_FLIP_V);
    button(store, ids::VECTOR_ARRANGE_ROTATE_CW);
    button(store, ids::VECTOR_ARRANGE_ROTATE_CCW);

    // Close (X) button.
    button(store, ids::VECTOR_CLOSE);
}
