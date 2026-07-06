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
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_vector::state::VectorPanelState;
use ph2d_panel_vector::{VectorPanel, ids};
use ph2d_tool_vector::VectorTool;
use ph2d_tool_vector::params::slider_to_px;
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

/// The Fill "None" button must clear the tool's fill + flag the selected path
/// for recolour, through the seam.
#[test]
fn fill_none_click_clears_fill_through_seam() {
    let mut host = MockPanelHost::with_panel::<VectorPanel>();
    let mut panel_state = VectorPanelState;
    let mut tool = VectorTool::default();
    assert_ne!(tool.fill_rgba()[3], 0, "precondition: default fill opaque");

    let outcome = host.apply_panel_event::<VectorPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::VECTOR_FILL_NONE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Fill-None click was ignored — `event.rs` arm for VECTOR_FILL_NONE is missing"
    );

    drain_into_tool(&mut host, &mut tool);
    assert_eq!(
        tool.fill_rgba()[3],
        0,
        "Fill-None click never cleared the fill through the seam"
    );
    assert!(
        tool.take_apply_to_selected(),
        "Fill-None must flag the selected path for recolour"
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
