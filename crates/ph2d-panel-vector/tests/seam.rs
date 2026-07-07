//! Behavioral SEAM test for the Vector Style panel ↔ tool (blindagem Fase 1.2).
//!
//! Unit tests in `ph2d-tool-vector` exercise `handle_panel_event` directly, and
//! `populate.rs` asserts widgets are registered — but NEITHER proves the wire
//! between them is intact. A forgotten `event.rs` arm or a wrong projection
//! leaves the control painted, draggable and SILENTLY DEAD while every unit test
//! + the `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set widget value → apply_event → bus → handle_panel_event
//!   → assert the tool's Style actually changed.
//!
//! (The Stroke / Fill colour swatches go through the OKLCH picker read-back in
//! the shell's `vector_bridge`, not through `apply_event`, so they are covered
//! by the tool's `set_stroke_rgba` / `set_fill_rgba` unit tests + the bridge —
//! not this panel seam.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::{PanelEvent, Tool}; // brings `handle_panel_event` into scope
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::params::{SIDES_MAX, slider_to_px};
use ph2d_tool_vector::{DrawMode, VectorTool};
use ph2d_ui_testkit::MockPanelHost;

/// Forward every drained `ToolPanelEvent` into the tool (what the shell does
/// each frame). Returns whether at least one was forwarded.
fn drain_into_tool(host: &mut MockPanelHost, tool: &mut VectorTool) -> bool {
    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    forwarded
}

/// Drag the Width slider to its full end and prove the width lands in the tool
/// — exercising every site from `populate` to `stroke_width_px()`.
#[test]
fn width_slider_drag_reaches_tool_style() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    // A drag writes the slider's stored value, then the dispatch emits
    // ValueChanged. Simulate both.
    host.set_slider_value(ids::VECTOR_WIDTH, 1.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_WIDTH),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for VECTOR_WIDTH is missing"
    );

    assert!(
        drain_into_tool(&mut host, &mut tool),
        "width edit never reached the bus as a ToolPanelEvent — the panel→shell seam is dead"
    );

    // End-to-end proof: the tool's width changed to the slider's px.
    assert_eq!(
        tool.stroke_width_px(),
        slider_to_px(1.0),
        "slider→tool seam delivered the wrong px for Width"
    );
}

/// The Fill Opacity slider owns the fill alpha (replaces the old "None" button):
/// dragging it to 0 makes the fill invisible, through the seam.
#[test]
fn fill_opacity_slider_sets_alpha_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(tool.fill_rgba()[3], 0, "precondition: default fill opaque");

    host.set_slider_value(ids::VECTOR_FILL_OPACITY, 0.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_FILL_OPACITY),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Fill Opacity edit ignored — `event.rs` arm for VECTOR_FILL_OPACITY is missing"
    );

    drain_into_tool(&mut host, &mut tool);
    assert_eq!(
        tool.fill_rgba()[3],
        0,
        "Fill Opacity → 0 never cleared the fill alpha through the seam"
    );
    assert!(
        tool.take_apply_to_selected(),
        "Opacity change must flag the selected path for recolour"
    );
}

/// A draw-mode button (Rectangle) must switch the tool's mode through the seam
/// — exercising the mode arm in `event.rs` + `handle_panel_event`.
#[test]
fn mode_button_click_switches_tool_mode_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_eq!(tool.mode(), DrawMode::Pen, "precondition: default is Pen");

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MODE_RECT),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "mode button ignored — `event.rs` arm for VECTOR_MODE_* is missing"
    );

    drain_into_tool(&mut host, &mut tool);
    assert_eq!(
        tool.mode(),
        DrawMode::Rectangle,
        "mode click never reached the tool through the seam"
    );
}

/// A Cap button + the Dash and Gap sliders reach the tool through the seam —
/// the stroke-detail controls.
#[test]
fn stroke_cap_dash_and_gap_reach_the_tool() {
    use ph2d_tool_vector::StrokeCap;
    use ph2d_tool_vector::params::{DASH_MAX, GAP_MAX};
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    let c = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_CAP_ROUND),
    );
    assert_eq!(c, EventOutcome::Consumed, "Cap button not wired");
    drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.cap(), StrokeCap::Round);

    host.set_slider_value(ids::VECTOR_DASH, 1.0);
    let d = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_DASH),
    );
    assert_eq!(d, EventOutcome::Consumed, "Dash slider not wired");
    drain_into_tool(&mut host, &mut tool);
    assert!((tool.dash() - DASH_MAX).abs() < 1e-6);

    host.set_slider_value(ids::VECTOR_GAP, 1.0);
    let g = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_GAP),
    );
    assert_eq!(g, EventOutcome::Consumed, "Gap slider not wired");
    drain_into_tool(&mut host, &mut tool);
    assert!((tool.gap() - GAP_MAX).abs() < 1e-6);
}

/// The Star mode button switches the mode, and the Star "Points" slider reaches
/// the tool's `star_points` through the seam — proving the new shape controls.
#[test]
fn star_mode_and_points_slider_reach_the_tool() {
    use ph2d_tool_vector::params::STAR_POINTS_MAX;
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    let m = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_MODE_STAR),
    );
    assert_eq!(m, EventOutcome::Consumed, "Star mode button not wired");
    drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.mode(), DrawMode::Star);

    host.set_slider_value(ids::VECTOR_STAR_POINTS, 1.0);
    let s = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_STAR_POINTS),
    );
    assert_eq!(s, EventOutcome::Consumed, "Star Points slider not wired");
    drain_into_tool(&mut host, &mut tool);
    assert_eq!(tool.draw_config().star_points, STAR_POINTS_MAX);
}

/// The Polygon Sides slider must reach the tool's `polygon_sides` through the
/// seam (same shape as the Width slider).
#[test]
fn sides_slider_drag_reaches_tool_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();

    host.set_slider_value(ids::VECTOR_SIDES, 1.0);
    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::VECTOR_SIDES),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Sides slider edit ignored — `event.rs` arm for VECTOR_SIDES is missing"
    );

    drain_into_tool(&mut host, &mut tool);
    assert_eq!(
        tool.polygon_sides(),
        SIDES_MAX,
        "Sides slider→tool seam delivered the wrong side count"
    );
}

/// A Boolean button (Union) is a DOCUMENT command, not a Style edit — the tool
/// ignores it, so the seam proof is that the panel forwards the `Click` onto the
/// bus as a `ToolPanelEvent` for the shell drain to apply.
#[test]
fn boolean_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_BOOL_UNION),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Boolean button ignored — `event.rs` arm for VECTOR_BOOL_* is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_BOOL_UNION
        )
    });
    assert!(
        forwarded,
        "Boolean click never reached the bus as a ToolPanelEvent — the shell can't apply the op"
    );
}

/// The Arrange buttons (Duplicate + z-order) are DOCUMENT commands acting on the
/// selected path — the tool ignores them, so the seam proof is that each `Click`
/// reaches the bus as a `ToolPanelEvent` for the shell drain to apply.
#[test]
fn arrange_buttons_forward_to_the_bus_for_the_shell() {
    for id in [
        ids::VECTOR_ARRANGE_DUPLICATE,
        ids::VECTOR_ARRANGE_TO_BACK,
        ids::VECTOR_ARRANGE_BACKWARD,
        ids::VECTOR_ARRANGE_FORWARD,
        ids::VECTOR_ARRANGE_TO_FRONT,
        ids::VECTOR_ARRANGE_FLIP_H,
        ids::VECTOR_ARRANGE_FLIP_V,
        ids::VECTOR_ARRANGE_ROTATE_CW,
        ids::VECTOR_ARRANGE_ROTATE_CCW,
    ] {
        let mut host = MockPanelHost::with_panel::<VectorPanel>();
        let mut panel_state = VectorPanelState;

        let outcome =
            host.apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "Arrange button ignored — `event.rs` arm for VECTOR_ARRANGE_* is missing"
        );

        let forwarded = host.drained_actions().iter().any(|a| {
            matches!(
                a,
                EditorAction::ToolPanelEvent(PanelEvent::Click(fid)) if *fid == id
            )
        });
        assert!(
            forwarded,
            "Arrange click never reached the bus as a ToolPanelEvent — the shell can't apply it"
        );
    }
}

/// A Vertex-type button (Smooth) is a DOCUMENT command (retypes the selected
/// vertex via the shell-side Pen), so — like the Boolean buttons — the seam
/// proof is that the panel forwards the `Click` onto the bus for the shell drain.
#[test]
fn vertex_type_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_VERT_SMOOTH),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Vertex-type button ignored — `event.rs` arm for VECTOR_VERT_* is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_VERT_SMOOTH
        )
    });
    assert!(
        forwarded,
        "Vertex-type click never reached the bus as a ToolPanelEvent — the shell can't retype it"
    );
}

/// The "Delete Node" button is a DOCUMENT command (removes the selected vertex
/// via the shell Pen), so — like Boolean/Vertex-type — the seam proof is that the
/// panel forwards the `Click` onto the bus for the shell drain.
#[test]
fn delete_node_button_click_forwards_to_the_bus_for_the_shell() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_VERT_DELETE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Delete-Node click ignored — `event.rs` arm for VECTOR_VERT_DELETE is missing"
    );

    let forwarded = host.drained_actions().iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == ids::VECTOR_VERT_DELETE
        )
    });
    assert!(
        forwarded,
        "Delete-Node click never reached the bus as a ToolPanelEvent — the shell can't delete it"
    );
}

/// The Close (X) button must emit `CancelActiveTool` (deactivates the tool),
/// mirror of the Padding panel's Cancel.
#[test]
fn close_button_cancels_active_tool() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;

    let outcome = host
        .apply_panel_event::<VectorPanel>(&mut panel_state, WidgetEvent::Click(ids::VECTOR_CLOSE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close click was ignored — `event.rs` arm for VECTOR_CLOSE is missing"
    );

    let cancelled = host
        .drained_actions()
        .iter()
        .any(|a| matches!(a, EditorAction::CancelActiveTool));
    assert!(
        cancelled,
        "Close click never emitted CancelActiveTool through the seam"
    );
}
