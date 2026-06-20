//! Behavioral SEAM test for the Padding panel ↔ tool — the test class the
//! 2026-06-20 diagnosis found missing (blindagem Fase 0.1, example).
//!
//! Unit tests in `ph2d-tool-padding` exercise `apply_ui_edit` directly, and
//! `populate.rs` asserts widgets are registered — but NEITHER proves the
//! ~13-site wire between them is intact. A forgotten `event.rs` arm, a
//! missing `handle_panel_event` branch, or a wrong projection all leave the
//! slider painted, draggable and SILENTLY DEAD while every unit test + the
//! `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set slider value → apply_event → bus → handle_panel_event
//!   → assert the tool's spec actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_padding::state::PaddingPanelState;
use ph2d_panel_padding::{PaddingPanel, ids};
use ph2d_tool_padding::PaddingTool;
use ph2d_tool_padding::params::slider_to_px;
use ph2d_ui_testkit::MockPanelHost;

/// Drag the TOP slider to its full-positive end and prove the value lands
/// in the tool's spec — exercising every site from `populate` to `spec()`.
#[test]
fn top_slider_drag_reaches_tool_spec() {
    let mut host = MockPanelHost::with_panel::<PaddingPanel>();
    let mut panel_state = PaddingPanelState;
    let mut tool = PaddingTool::default();

    // A drag writes the slider's stored value, then the dispatch emits
    // ValueChanged. Simulate both.
    host.set_slider_value(ids::PAD_TOP, 1.0);
    let outcome = host.apply_panel_event::<PaddingPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::PAD_TOP),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` allowlist/arm for PAD_TOP is missing"
    );

    // The shell drains `ToolPanelEvent` → `tool.handle_panel_event` each frame.
    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    assert!(
        forwarded,
        "slider edit never reached the bus as a ToolPanelEvent — the panel→shell seam is dead"
    );

    // End-to-end proof: the tool's spec changed (handle_panel_event arm +
    // apply_ui_edit). This is precisely what unit tests + contract gates miss.
    let (top, right, bottom, left) = tool.spec();
    assert_eq!(
        top,
        slider_to_px(1.0),
        "slider→tool seam delivered the wrong px for TOP"
    );
    assert!(
        top > 0,
        "full-positive slider must produce positive padding, got {top}"
    );
    assert_eq!(
        (right, bottom, left),
        (0, 0, 0),
        "only TOP was edited — the other edges must stay 0"
    );
}

/// The Apply button must arm the tool's one-shot bake latch through the seam.
#[test]
fn apply_button_arms_the_bake() {
    let mut host = MockPanelHost::with_panel::<PaddingPanel>();
    let mut panel_state = PaddingPanelState;
    let mut tool = PaddingTool::default();

    let outcome = host
        .apply_panel_event::<PaddingPanel>(&mut panel_state, WidgetEvent::Click(ids::PAD_APPLY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Apply click was ignored — `event.rs` arm for PAD_APPLY is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert!(
        tool.take_pending_apply(),
        "Apply click never armed the tool's bake latch through the seam"
    );
}
