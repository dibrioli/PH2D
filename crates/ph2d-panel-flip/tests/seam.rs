//! Behavioral SEAM test for the Flip panel ↔ tool (blindagem Fase 1.2).
//!
//! Unit tests in `ph2d-tool-flip` exercise `handle_panel_event` directly, and
//! `populate.rs` registers widgets — but NEITHER proves the wire between them is
//! intact. A forgotten `event.rs` arm or a wrong id would leave a slider painted,
//! draggable and SILENTLY DEAD while every unit test + contract gate stays green.
//!
//! These run the full path the desktop shell runs, headless:
//!   populate → set value / click → apply_event → bus → handle_panel_event
//!   → assert the tool's state actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_flip::state::FlipPanelState;
use ph2d_panel_flip::{FlipPanel, ids};
use ph2d_tool_flip::{FlipMode, FlipTool, WIDTH_MAX_PX};
use ph2d_ui_testkit::MockPanelHost;

/// Drag the Size slider to its full end and prove the width reaches the tool —
/// exercising every site from `populate` to `width_px()`.
#[test]
fn size_slider_drag_reaches_tool() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState;
    let mut tool = FlipTool::default();

    host.set_slider_value(ids::FLIP_SIZE, 1.0);
    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut panel_state, WidgetEvent::ValueChanged(ids::FLIP_SIZE));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for FLIP_SIZE is missing"
    );

    let mut forwarded = false;
    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
            forwarded = true;
        }
    }
    assert!(
        forwarded,
        "slider edit never reached the bus as a ToolPanelEvent — the seam is dead"
    );
    assert_eq!(
        tool.width_px(),
        WIDTH_MAX_PX,
        "slider→tool seam delivered the wrong px for Size"
    );
}

/// Clicking the Draw mode button must switch the tool's canvas mode through the
/// seam (Select → Draw).
#[test]
fn draw_mode_button_switches_the_tool_mode() {
    let mut host = MockPanelHost::with_panel::<FlipPanel>();
    let mut panel_state = FlipPanelState;
    let mut tool = FlipTool::default();
    assert_eq!(tool.mode(), FlipMode::Select, "fresh tool starts in Select");

    let outcome = host
        .apply_panel_event::<FlipPanel>(&mut panel_state, WidgetEvent::Click(ids::FLIP_MODE_DRAW));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Draw button click ignored — `event.rs` arm for FLIP_MODE_DRAW is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.mode(),
        FlipMode::Draw,
        "mode button never switched the tool mode through the seam"
    );
}
