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
    ARC_DEGREES_SLIDER_OFFSET, ARC_DEGREES_SLIDER_SCALE, DASH_SLIDER_OFFSET, DASH_SLIDER_SCALE,
    GAP_DEFAULT, GAP_SLIDER_OFFSET, GAP_SLIDER_SCALE, OPACITY_SLIDER_OFFSET, OPACITY_SLIDER_SCALE,
    RADIUS_SLIDER_OFFSET, RADIUS_SLIDER_SCALE, SIDES_SLIDER_OFFSET, SIDES_SLIDER_SCALE,
    SPIRAL_TURNS_SLIDER_OFFSET, SPIRAL_TURNS_SLIDER_SCALE, STAR_INNER_SLIDER_OFFSET,
    STAR_INNER_SLIDER_SCALE, STAR_POINTS_SLIDER_OFFSET, STAR_POINTS_SLIDER_SCALE,
    WIDTH_SLIDER_OFFSET, WIDTH_SLIDER_SCALE, arc_degrees_to_slider, gap_to_slider,
    radius_to_slider, sides_to_slider, spiral_turns_to_slider, star_inner_to_slider,
    star_points_to_slider,
};
use ph2d_tool_vector::{
    DEFAULT_ARC_DEGREES, DEFAULT_CORNER_RADIUS_PX, DEFAULT_POLYGON_SIDES, DEFAULT_SPIRAL_TURNS,
    DEFAULT_STAR_INNER, DEFAULT_STAR_POINTS, DEFAULT_STROKE_WIDTH_PX, px_to_slider,
};

/// Linear-gradient Angle slider mapping: track `0..1` → `0..360` degrees.
const GRAD_ANGLE_SLIDER_SCALE: f32 = 360.0; // LITERAL-PX-OK: degrees in a full turn (math constant)
/// Multi-point Influence slider mapping: track `0..1` → `0..4` (IDW strength).
const GRAD_INFLUENCE_SLIDER_SCALE: f32 = 4.0; // LITERAL-PX-OK: max IDW influence (domain constant)
/// Multi-point Jitter slider mapping: track `0..1` → `0..1` (per-texel grain).
const GRAD_JITTER_SLIDER_SCALE: f32 = 1.0; // LITERAL-PX-OK: jitter is already a 0..1 fraction

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

/// Registra os widgets do painel Vector. Delegado a helpers por seção — o corpo
/// plano estourava o teto de 200 LOC por função dos painéis.
pub fn populate(store: &mut WidgetStore) {
    populate_shape(store);
    populate_ops(store);
    populate_style(store);
    populate_arrange(store);
}

/// Width + modos de desenho + os sliders per-forma (Sides / Star / RoundRect / Spiral).
fn populate_shape(store: &mut WidgetStore) {
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
    button(store, ids::VECTOR_MODE_SELECT);
    button(store, ids::VECTOR_MODE_NODE);
    button(store, ids::VECTOR_MODE_PEN);
    button(store, ids::VECTOR_MODE_RECT);
    button(store, ids::VECTOR_MODE_ELLIPSE);
    button(store, ids::VECTOR_MODE_POLYGON);
    button(store, ids::VECTOR_MODE_STAR);
    button(store, ids::VECTOR_MODE_RRECT);
    button(store, ids::VECTOR_MODE_SPIRAL);
    button(store, ids::VECTOR_MODE_LINE);
    button(store, ids::VECTOR_MODE_ARC);

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
    slider_chip(
        store,
        ids::VECTOR_SPIRAL_TURNS,
        ids::VECTOR_SPIRAL_TURNS_NUM,
        spiral_turns_to_slider(DEFAULT_SPIRAL_TURNS),
        f64::from(DEFAULT_SPIRAL_TURNS),
        SPIRAL_TURNS_SLIDER_SCALE,
        SPIRAL_TURNS_SLIDER_OFFSET,
    );
    slider_chip(
        store,
        ids::VECTOR_ARC_DEGREES,
        ids::VECTOR_ARC_DEGREES_NUM,
        arc_degrees_to_slider(DEFAULT_ARC_DEGREES),
        DEFAULT_ARC_DEGREES,
        ARC_DEGREES_SLIDER_SCALE,
        ARC_DEGREES_SLIDER_OFFSET,
    );

    populate_transform_fields(store);
}

/// Vértice, Boolean + Compound, Fill Rule, Snap, tipo de Fill, Align/Distribute.
fn populate_ops(store: &mut WidgetStore) {
    // Vertex-type buttons (retype the selected vertex; shown only when a vertex
    // is selected, but registered unconditionally — the store is mode-agnostic)
    // + the Delete-node button.
    button(store, ids::VECTOR_VERT_CORNER);
    button(store, ids::VECTOR_VERT_SMOOTH);
    button(store, ids::VECTOR_VERT_SYMMETRIC);
    button(store, ids::VECTOR_VERT_DELETE);

    // Boolean op buttons (N-ary over the SELECTED closed regions) + compound row.
    button(store, ids::VECTOR_BOOL_UNION);
    button(store, ids::VECTOR_BOOL_SUBTRACT);
    button(store, ids::VECTOR_BOOL_INTERSECT);
    button(store, ids::VECTOR_BOOL_EXCLUDE);
    button(store, ids::VECTOR_COMPOUND_MAKE);
    button(store, ids::VECTOR_COMPOUND_RELEASE);
    // Fill rule (compound paths only — the row hides for a single contour).
    button(store, ids::VECTOR_FILL_RULE_NONZERO);
    button(store, ids::VECTOR_FILL_RULE_EVENODD);
    // Shape snapping (tool setting, always visible). The grid toggle is in the
    // editor's universal Grid Snap panel.
    button(store, ids::VECTOR_SNAP_OFF);
    button(store, ids::VECTOR_SNAP_ON);

    // Fill-type selector (Solid / Linear / Radial) — act on the selected path.
    button(store, ids::VECTOR_FILL_KIND_SOLID);
    button(store, ids::VECTOR_FILL_KIND_LINEAR);
    button(store, ids::VECTOR_FILL_KIND_RADIAL);
    button(store, ids::VECTOR_FILL_KIND_MULTI);
    button(store, ids::VECTOR_GRAD_ADD_POINT);
    button(store, ids::VECTOR_GRAD_REMOVE_POINT);
    button(store, ids::VECTOR_GRAD_ADD_STOP);
    button(store, ids::VECTOR_GRAD_REMOVE_STOP);
    // Align + Distribute (multi-path object selection).
    button(store, ids::VECTOR_PIVOT_EDIT);
    button(store, ids::VECTOR_ALIGN_LEFT);
    button(store, ids::VECTOR_ALIGN_HCENTER);
    button(store, ids::VECTOR_ALIGN_RIGHT);
    button(store, ids::VECTOR_ALIGN_TOP);
    button(store, ids::VECTOR_ALIGN_VCENTER);
    button(store, ids::VECTOR_ALIGN_BOTTOM);
    button(store, ids::VECTOR_DISTRIBUTE_H);
    button(store, ids::VECTOR_DISTRIBUTE_V);
}

/// Gradiente (Angle / Influence / Jitter), opacidades e o traço (cap/join/dash).
fn populate_style(store: &mut WidgetStore) {
    // Linear-gradient Angle slider (track 0..1 → 0..360°) + its chip.
    slider_chip(
        store,
        ids::VECTOR_GRAD_ANGLE,
        ids::VECTOR_GRAD_ANGLE_NUM,
        0.0,
        0.0,
        GRAD_ANGLE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Influence (track 0..1 → 0..4); seeded at the default 1.0.
    slider_chip(
        store,
        ids::VECTOR_GRAD_INFLUENCE,
        ids::VECTOR_GRAD_INFLUENCE_NUM,
        1.0 / GRAD_INFLUENCE_SLIDER_SCALE,
        1.0,
        GRAD_INFLUENCE_SLIDER_SCALE,
        0.0,
    );
    // Multi-point Jitter (track 0..1 → 0..1); seeded at the default 0.0 (smooth).
    slider_chip(
        store,
        ids::VECTOR_GRAD_JITTER,
        ids::VECTOR_GRAD_JITTER_NUM,
        0.0,
        0.0,
        GRAD_JITTER_SLIDER_SCALE,
        0.0,
    );

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
}

/// Arrange (duplicate / z-order / flip / rotate), reshape de path, e o botão Close.
fn populate_arrange(store: &mut WidgetStore) {
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

    // Path: Smooth / Sharpen / Simplify — reshape ALL vertices of the selected path.
    button(store, ids::VECTOR_PATH_SMOOTH);
    button(store, ids::VECTOR_PATH_SHARPEN);
    button(store, ids::VECTOR_PATH_SIMPLIFY);
    button(store, ids::VECTOR_PATH_SUBDIVIDE);
    button(store, ids::VECTOR_PATH_CLOSE);

    // Close (X) button.
    button(store, ids::VECTOR_CLOSE);
}

/// Register the four Transform number fields (X/Y/W/H) — standalone (NOT slider-
/// linked); seeded from the selected path's bbox each frame, edits route a
/// document command through the shell drain.
fn populate_transform_fields(store: &mut WidgetStore) {
    for id in [
        ids::VECTOR_TRANSFORM_X,
        ids::VECTOR_TRANSFORM_Y,
        ids::VECTOR_TRANSFORM_W,
        ids::VECTOR_TRANSFORM_H,
        ids::VECTOR_TRANSFORM_R,
    ] {
        store.register(
            id,
            InteractiveState::NumberInput {
                state: TextInputState::Normal,
                value: 0.0,
                buffer: String::from("0"),
                caret: 0,
                last_committed: 0.0,
                selection_anchor: None,
            },
        );
        // NO `set_number_range` → the field is UNBOUNDED (no clamp, world coords
        // span any magnitude). The shell's `vector_bridge` calibrates the drag
        // scrub live via `set_number_drag_rate(px_to_world)` so a chip drag is 1:1
        // with the shape's on-screen movement at any zoom.
    }
}
