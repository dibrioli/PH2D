//! Behavioral SEAM test for the Equalize Sizes panel ↔ tool — the test
//! class the 2026-06-20 diagnosis found missing (blindagem Fase 0.1).
//!
//! Unit tests in `ph2d-tool-equalize-sizes` exercise `apply_ui_edit` /
//! `handle_panel_event` directly, and `populate.rs` asserts widgets are
//! registered — but NEITHER proves the hand-wired chain BETWEEN them is
//! intact. A forgotten `event.rs` arm, a missing `handle_panel_event`
//! branch, or a wrong `PanelEvent` classification all leave the widget
//! painted, clickable/draggable and SILENTLY DEAD while every unit test +
//! the `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set widget value → apply_event → bus → handle_panel_event
//!   → assert the TOOL'S observable state (`params()`) actually changed.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_equalize_sizes::{EqualizeSizesPanel, ids, state::EqualizeSizesPanelState};
use ph2d_tool_equalize_sizes::params::TargetMode;
use ph2d_tool_equalize_sizes::tool::EqualizeSizesTool;
use ph2d_ui_testkit::MockPanelHost;

/// Type the Fixed-mode WIDTH chip and prove the px value lands in the
/// tool's authoritative params — exercising every site from `populate`
/// (`EQS_FIXED_W` registered as a `NumberInput`) through `event.rs`
/// (`ValueChanged` arm → `PanelEvent::SetValue`) and the bus drain into
/// `handle_panel_event` (`SetValue(EQS_FIXED_W, ..)` → `SetFixedW` →
/// `apply_ui_edit` clamp+commit).
#[test]
fn fixed_w_chip_edit_reaches_tool_params() {
    let mut host = MockPanelHost::with_panel::<EqualizeSizesPanel>();
    let mut panel_state = EqualizeSizesPanelState::default();
    let mut tool = EqualizeSizesTool::default();

    // Sanity: the default width must differ from our target so the
    // assertion below proves a real change, not a coincidence with the
    // default (256).
    assert_eq!(tool.params().fixed_w, 256, "test premise: default fixed_w");

    // A scrub/type commits the chip's stored value, then the dispatch
    // emits `ValueChanged(id)`. Simulate both. 512 is well under the
    // `EQS_MAX_FIXED_DIM` (4096) cap so the post-condition is exact.
    host.set_number_value(ids::EQS_FIXED_W, 512.0);
    let outcome = host.apply_panel_event::<EqualizeSizesPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::EQS_FIXED_W),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real chip edit — `event.rs` ValueChanged arm for EQS_FIXED_W is missing"
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
        "chip edit never reached the bus as a ToolPanelEvent — the panel→shell seam is dead"
    );

    // End-to-end proof: the tool's OBSERVABLE params changed (the
    // `handle_panel_event` arm + `apply_ui_edit` commit). This is exactly
    // what the tool unit tests and the contract gates cannot catch.
    assert_eq!(
        tool.params().fixed_w,
        512,
        "chip→tool seam delivered the wrong fixed_w"
    );
    assert_eq!(
        tool.params().fixed_h,
        256,
        "only width was edited — height must stay at its default"
    );
}

/// Click the "Grid Unit" mode button and prove the tool's `target_mode`
/// actually flips — exercising the `Click` arm in `event.rs`
/// (`PanelEvent::Click`) and the `EQS_MODE_GRID` branch in
/// `handle_panel_event` (`SetMode(TargetMode::GridUnit)`).
#[test]
fn grid_mode_button_click_flips_tool_target_mode() {
    let mut host = MockPanelHost::with_panel::<EqualizeSizesPanel>();
    let mut panel_state = EqualizeSizesPanelState::default();
    let mut tool = EqualizeSizesTool::default();

    assert_eq!(
        tool.params().target_mode,
        TargetMode::MaxOfSelection,
        "test premise: default target_mode is MaxOfSelection"
    );

    let outcome = host.apply_panel_event::<EqualizeSizesPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::EQS_MODE_GRID),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "mode-button Click was ignored — `event.rs` Click arm for EQS_MODE_GRID is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }

    assert_eq!(
        tool.params().target_mode,
        TargetMode::GridUnit,
        "Grid-mode button click never reached the tool's target_mode — the seam is dead"
    );
}
