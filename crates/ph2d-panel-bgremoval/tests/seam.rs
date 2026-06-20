//! Behavioral SEAM test for the Background-Removal panel ↔ tool — the test
//! class the 2026-06-20 diagnosis found missing (blindagem Fase 0.1).
//!
//! Unit tests in `ph2d-tool-bgremoval` exercise `apply_ui_edit` (and even
//! `handle_panel_event`) directly, and `populate.rs` asserts the widgets are
//! registered — but NEITHER proves the hand-wired chain BETWEEN them is
//! intact. A forgotten arm in `event.rs` (the `is_bgr_slider` / `is_bgr_click`
//! allowlists), a missing `handle_panel_event` branch, or a wrong projection
//! all leave the slider/button painted, draggable and SILENTLY DEAD while
//! every unit test + the `*_contract_surface` gates stay green.
//!
//! These tests run the full path the desktop shell runs, headless:
//!   populate → set widget value → apply_event → bus drain →
//!   handle_panel_event → assert the TOOL'S OBSERVABLE state changed.
//!
//! The bgremoval panel is a pure widget bag (`event.rs` only CLASSIFIES the
//! `WidgetEvent` into a tool-agnostic `PanelEvent` and pushes it on the bus;
//! the semantic mapping lives in `BgRemovalTool::handle_panel_event` →
//! `apply_ui_edit`, ADR-0040 TG-B). So the only end-to-end proof is the
//! tool's observable state — here `ui_snapshot()` (the normalized projection
//! the panel itself paints) and `take_pending_apply()` (the bake latch).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::Tool; // brings `handle_panel_event` into scope
use ph2d_panel_bgremoval::state::BgRemovalPanelState;
use ph2d_panel_bgremoval::{BgRemovalPanel, ids};
use ph2d_tool_bgremoval::BgRemovalTool;
use ph2d_ui_testkit::MockPanelHost;

/// Drive the TOLERANCE slider to a known value and prove it lands in the
/// tool's params, observed through `ui_snapshot().tolerance01` — the exact
/// normalized value the panel paints back. Tolerance is identity-mapped
/// (slider 0..1 ↔ snapshot 0..1), so the assertion is a tight equality, not
/// a "something changed" smell test.
///
/// Exercises every site from `populate` (slider registered) through
/// `event.rs` (`is_bgr_slider` allowlist + bus push) to
/// `handle_panel_event` (`BGR_TOLERANCE` arm → `apply_ui_edit::Tolerance`).
#[test]
fn tolerance_slider_drag_reaches_tool_snapshot() {
    let mut host = MockPanelHost::with_panel::<BgRemovalPanel>();
    let mut panel_state = BgRemovalPanelState::default();
    let mut tool = BgRemovalTool::default();

    // Sanity: the default snapshot does NOT already sit at our target, so a
    // green assertion can only mean the edit actually propagated (the tuned
    // default is tolerance01 = 0.6).
    const TARGET: f32 = 0.25;
    assert!(
        (tool.ui_snapshot().tolerance01 - TARGET).abs() > 1e-3,
        "test setup invalid: default tolerance01 already equals the target {TARGET}"
    );

    // A pointer drag writes the slider's stored value, then dispatch emits
    // ValueChanged. Simulate both — `event.rs` reads the stored value back
    // off the store when it forwards.
    host.set_slider_value(ids::BGR_TOLERANCE, TARGET);
    let outcome = host.apply_panel_event::<BgRemovalPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::BGR_TOLERANCE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real slider edit — the `is_bgr_slider` allowlist/arm \
         for BGR_TOLERANCE in event.rs is missing"
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
        "slider edit never reached the bus as a ToolPanelEvent — the \
         panel→shell seam is dead"
    );

    // END-TO-END: the tool's OBSERVABLE state changed. `ui_snapshot()` is the
    // normalized projection the panel paints; tolerance is identity-mapped so
    // the value must land exactly. This is precisely what the tool's own unit
    // tests (which call apply_ui_edit directly) + the contract gates miss.
    assert!(
        (tool.ui_snapshot().tolerance01 - TARGET).abs() < 1e-5,
        "slider→tool seam dead: expected tolerance01 {TARGET}, got {} — the \
         edit was emitted but never landed in the tool",
        tool.ui_snapshot().tolerance01
    );
}

/// The Apply button must arm the tool's one-shot bake latch through the seam.
/// `event.rs` forwards it as `PanelEvent::Click(BGR_APPLY)`;
/// `handle_panel_event` maps that to `apply_ui_edit::Apply`, which sets
/// `pending_apply`. The observable post-condition is `take_pending_apply()`.
#[test]
fn apply_button_arms_the_bake() {
    let mut host = MockPanelHost::with_panel::<BgRemovalPanel>();
    let mut panel_state = BgRemovalPanelState::default();
    let mut tool = BgRemovalTool::default();

    // Nothing pending before the click.
    assert!(
        !tool.take_pending_apply(),
        "test setup invalid: bake latch already armed before the click"
    );

    let outcome = host.apply_panel_event::<BgRemovalPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::BGR_APPLY),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Apply click was ignored — the `is_bgr_click` allowlist/arm for \
         BGR_APPLY in event.rs is missing"
    );

    for action in host.drained_actions() {
        if let EditorAction::ToolPanelEvent(pe) = action {
            tool.handle_panel_event(pe);
        }
    }

    // END-TO-END: the bake latch is armed in the tool. A test that only
    // checked "an action was pushed" would pass even if the
    // handle_panel_event arm were missing — this proves the effect LANDED.
    assert!(
        tool.take_pending_apply(),
        "Apply click never armed the tool's bake latch through the seam"
    );
}
