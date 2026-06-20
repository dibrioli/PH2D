//! Behavioral SEAM test for the Upscale panel ↔ tool — the test class
//! the 2026-06-20 diagnosis found missing (blindagem Fase 0.1).
//!
//! Unit tests in `ph2d-tool-upscale` exercise `apply_ui_edit` /
//! `handle_panel_event` directly, and `populate.rs` asserts widgets are
//! registered — but NEITHER proves the hand-wired sites BETWEEN them are
//! intact. A forgotten `event.rs` arm, a wrong `host.store().slider(..)`
//! read, a missing `handle_panel_event` branch, or a bad projection all
//! leave the slider/segment painted, draggable and SILENTLY DEAD while
//! every unit test + the `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set widget value → apply_event → bus drain
//!   → tool.handle_panel_event → assert the tool's OBSERVABLE state
//!   (`tool.params.*` / `take_pending_apply`) actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_upscale::state::UpscalePanelState;
use ph2d_panel_upscale::{UpscalePanel, ids};
use ph2d_tool_upscale::UpscaleTool;
use ph2d_tool_upscale::params::{UpscaleAlgorithm, slider_to_scale};
use ph2d_ui_testkit::MockPanelHost;

/// Drag the scale slider to its full-positive end (track = 1.0) and
/// prove the value lands in the tool's live params — exercising every
/// site from `populate` (slider registered in track space) through
/// `event.rs` (reads the stored track, forwards `SetValue`) to
/// `handle_panel_event` (`slider_to_scale` → `apply_ui_edit::Scale`).
#[test]
fn scale_slider_drag_reaches_tool_params() {
    let mut host = MockPanelHost::with_panel::<UpscalePanel>();
    let mut panel_state = UpscalePanelState::default();
    let mut tool = UpscaleTool::default();

    // Sanity: default tool sits at 2.0× before any seam traffic, so a
    // post-condition of 16.0× is unambiguously the slider's doing.
    assert!(
        (tool.params.scale_factor - 2.0).abs() < f32::EPSILON,
        "default tool should start at 2x"
    );

    // A drag writes the slider's stored track value, then the dispatch
    // emits ValueChanged. Simulate both. track=1.0 → full scale.
    host.set_slider_value(ids::UPS_SCALE, 1.0);
    let outcome = host.apply_panel_event::<UpscalePanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::UPS_SCALE),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — `event.rs` arm for UPS_SCALE is missing"
    );

    // The shell drains `ToolPanelEvent` → `tool.handle_panel_event` each
    // frame. (Cancel/SetValue both flow here; only ToolPanelEvent is the
    // params seam.)
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

    // End-to-end proof: the tool's live params changed (handle_panel_event
    // SetValue arm + slider_to_scale projection + apply_ui_edit::Scale).
    // This is precisely what unit tests + contract gates miss.
    let expected = slider_to_scale(1.0); // = SCALE_FULL_SCALE = 16.0
    assert!(
        (tool.params.scale_factor - expected).abs() < f32::EPSILON,
        "slider→tool seam delivered the wrong scale: got {}, expected {expected}",
        tool.params.scale_factor
    );
    assert!(
        tool.params.scale_factor > 2.0,
        "full-positive slider must raise the scale above the 2x default, got {}",
        tool.params.scale_factor
    );
}

/// Click the Nearest algorithm segment and prove the selection lands in
/// the tool's live params through the seam — covers the segmented-button
/// `Click` arm in `event.rs` and the algorithm branch in
/// `handle_panel_event`. Default algorithm is Lanczos3, so observing
/// Nearest is unambiguously the click's effect.
#[test]
fn algorithm_segment_click_reaches_tool_params() {
    let mut host = MockPanelHost::with_panel::<UpscalePanel>();
    let mut panel_state = UpscalePanelState::default();
    let mut tool = UpscaleTool::default();

    assert_eq!(
        tool.params.algorithm,
        UpscaleAlgorithm::Lanczos3,
        "default algorithm should be Lanczos3 before the seam click"
    );

    let outcome = host.apply_panel_event::<UpscalePanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::UPS_ALGO_NEAREST),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "algorithm-segment click was ignored — `event.rs` arm for UPS_ALGO_NEAREST is missing"
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
        "algorithm click never reached the bus as a ToolPanelEvent — the seam is dead"
    );

    assert_eq!(
        tool.params.algorithm,
        UpscaleAlgorithm::Nearest,
        "algorithm→tool seam delivered the wrong selection — `handle_panel_event` branch dead"
    );
}

/// Click Apply and prove it arms the tool's one-shot bake latch through
/// the seam (`event.rs` Apply arm → `handle_panel_event` → `apply_ui_edit::Apply`).
#[test]
fn apply_button_arms_the_bake() {
    let mut host = MockPanelHost::with_panel::<UpscalePanel>();
    let mut panel_state = UpscalePanelState::default();
    let mut tool = UpscaleTool::default();

    let outcome = host
        .apply_panel_event::<UpscalePanel>(&mut panel_state, WidgetEvent::Click(ids::UPS_APPLY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Apply click was ignored — `event.rs` arm for UPS_APPLY is missing"
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
