//! Behavioral SEAM test for the Color Equalization panel ↔ tool — the test
//! class the 2026-06-20 diagnosis found missing, applied to the panel where
//! the §12.1 dead-dropdown/slider bug actually happened.
//!
//! Unit tests in `ph2d-tool-color-equalization` exercise `apply_ui_edit`
//! directly, and `populate.rs` asserts every widget is registered — but
//! NEITHER proves the multi-site wire between them is intact. A forgotten
//! `event.rs` arm (the `slider_for_widget` table), a missing
//! `handle_panel_event` branch, or a wrong projection all leave a slider
//! painted, draggable and SILENTLY DEAD while every unit test + the
//! `*_contract_surface` gates stay green. CEQ has the most controls of any
//! panel, so the seam is the easiest place to drop one.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set slider value → apply_event → bus → handle_panel_event
//!   → assert the tool's OBSERVABLE params actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_color_equalization::{ColorEqualizationPanel, ColorEqualizationPanelState, ids};
use ph2d_tool_color_equalization::ColorEqualizationTool;
use ph2d_tool_color_equalization::params::{BRIGHTNESS_MAX, BRIGHTNESS_MIN};
use ph2d_ui_testkit::MockPanelHost;

/// Drive the BRIGHTNESS slider to its full-positive end through the real
/// panel event router and prove the value lands in the tool's params —
/// exercising every site from `populate` → `event::apply_event`'s
/// `slider_for_widget` table → the bus → `handle_panel_event`'s
/// `BrightnessSlider` arm → `apply_ui_edit`'s `slider_to_brightness`
/// projection. Track 1.0 unprojects to `BRIGHTNESS_MAX`.
#[test]
fn brightness_slider_drag_reaches_tool_params() {
    let mut host = MockPanelHost::with_panel::<ColorEqualizationPanel>();
    let mut panel_state = ColorEqualizationPanelState;
    let mut tool = ColorEqualizationTool::default();

    // Default must NOT already equal the post-edit value, or the assertion
    // below could pass on a dead seam by coincidence.
    assert_ne!(
        tool.params.brightness, BRIGHTNESS_MAX,
        "test precondition: default brightness must differ from the edit target"
    );

    // A pointer drag writes the slider's stored value, then the dispatch
    // emits ValueChanged(id). Simulate both.
    host.set_slider_value(ids::CEQ_BRIGHTNESS, 1.0);
    let outcome = host.apply_panel_event::<ColorEqualizationPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::CEQ_BRIGHTNESS),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm/`slider_for_widget` entry for CEQ_BRIGHTNESS is missing"
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

    // End-to-end post-condition: the tool's OBSERVABLE param changed to the
    // projected natural-unit value. Track 1.0 → unproject01(1.0, -1, 1) = +1.
    // This is precisely what unit tests + contract gates miss.
    assert_eq!(
        tool.params.brightness, BRIGHTNESS_MAX,
        "slider→tool seam delivered the wrong brightness for track 1.0"
    );
    // Cross-check via the snapshot the panel actually paints (forward
    // projection): full-positive natural unit must round-trip to track 1.0.
    assert_eq!(
        tool.ui_snapshot().brightness01,
        1.0,
        "ui_snapshot disagrees with the committed param — projection is desynced"
    );
}

/// The opposite slider end must project to the opposite param extreme,
/// proving the seam carries the real value and is not a constant/no-op:
/// track 0.0 → unproject01(0.0, -1, 1) = BRIGHTNESS_MIN (-1).
#[test]
fn brightness_slider_low_end_projects_to_min() {
    let mut host = MockPanelHost::with_panel::<ColorEqualizationPanel>();
    let mut panel_state = ColorEqualizationPanelState;
    let mut tool = ColorEqualizationTool::default();

    host.set_slider_value(ids::CEQ_BRIGHTNESS, 0.0);
    let outcome = host.apply_panel_event::<ColorEqualizationPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::CEQ_BRIGHTNESS),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "low-end slider edit ignored by panel"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }
    assert_eq!(
        tool.params.brightness, BRIGHTNESS_MIN,
        "slider→tool seam did not carry the low-end value — projection is a no-op/constant"
    );
}
