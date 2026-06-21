//! Behavioral SEAM test for the Painter Layers panel ↔ tool — the test class
//! the 2026-06-20 diagnosis found missing (blindagem Fase 1).
//!
//! `PainterLayersPanel` is a thin FORWARDER (ADR-0040 TG-B, mirror of the
//! image panels): its `apply_event` classifies each `WidgetEvent` into a
//! tool-agnostic [`PanelEvent`] and pushes it on the action bus as
//! `EditorAction::ToolPanelEvent(..)` (or `CancelActiveTool` for the X). The
//! shell drains the bus each frame and calls `PainterTool::handle_panel_event`
//! on the active tool.
//!
//! Unit tests in `ph2d-tool-painter` exercise `handle_panel_event` / the
//! `LayerStack` API directly, and `populate.rs` registers the chrome buttons —
//! but NEITHER proves the wire BETWEEN them is intact. A forgotten `event.rs`
//! arm or a wrong `PanelEvent` shape leaves the "+ Layer" button painted,
//! clickable and SILENTLY DEAD while every unit test + `*_contract_surface`
//! gate stays green.
//!
//! These tests prove the PANEL→shell seam: the real click forwards the EXACT
//! `PanelEvent` the tool's matching arm consumes. (The tool-side effect —
//! `add_raster_layer` growing the stack — needs a canvas/source size, which a
//! headless seam test doesn't set up; that path is covered by
//! `ph2d-tool-painter`'s own unit tests. The seam this gate exists for is the
//! forward, which is asserted here on the exact inner id.)

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_painter_layers::PainterLayersPanel;
use ph2d_panel_painter_layers::state::PainterLayersPanelState;
use ph2d_ui_testkit::MockPanelHost;

/// Click the fixed "+ Layer" chrome button and prove it forwards as the
/// SPECIFIC `ToolPanelEvent(PanelEvent::Click(PAINTER_LAYERS_ADD))` — the exact
/// event `PainterTool::handle_panel_event`'s `PAINTER_LAYERS_ADD` arm consumes.
/// Exercises `populate` (button registered) → `event.rs` (chrome allowlist arm).
#[test]
fn add_layer_click_forwards_specific_id() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    // The exact id a real pointer click on the "+ Layer" button carries.
    let clicked = core_ids::PAINTER_LAYERS_ADD;

    let outcome =
        host.apply_panel_event::<PainterLayersPanel>(&mut panel_state, WidgetEvent::Click(clicked));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the '+ Layer' click — the `event.rs` chrome-button allowlist arm for \
         PAINTER_LAYERS_ADD is missing"
    );

    // Assert the bus carries the SPECIFIC inner PanelEvent::Click(clicked), not
    // merely "a ToolPanelEvent exists" — a wrong/forgotten arm changes the id.
    let actions = host.drained_actions();
    let forwarded = actions.iter().any(|a| {
        matches!(
            a,
            EditorAction::ToolPanelEvent(PanelEvent::Click(id)) if *id == clicked
        )
    });
    assert!(
        forwarded,
        "'+ Layer' click never reached the bus as ToolPanelEvent(Click(PAINTER_LAYERS_ADD)) — \
         the panel→shell seam is dead. drained = {actions:?}"
    );
}

/// DEFECT REPRO (Vector falloff-handle menu does nothing): the right-click
/// "handle type" menu item `Click(CTX_MENU_FALLOFF_HANDLE_VECTOR)` is a CHROME
/// id, NOT a panel widget — it must fall through to `chrome::falloff_handle`.
/// The painter panel is in the registry whenever the Painter is active and runs
/// BEFORE chrome in `HeroScreen::apply_event`; if it returns `Consumed` (or
/// `Observed` that the host treats as handled) for this unknown id, the chrome
/// handler never runs and `pending_falloff_point_handle` is never set → the
/// curve stays smooth. This test pins the panel to `Ignored`.
#[test]
fn falloff_handle_menu_click_is_not_consumed_by_panel() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    for id in [
        core_ids::CTX_MENU_FALLOFF_HANDLE_VECTOR,
        core_ids::CTX_MENU_FALLOFF_HANDLE_AUTO,
    ] {
        let outcome =
            host.apply_panel_event::<PainterLayersPanel>(&mut panel_state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Ignored,
            "painter panel ATE the falloff-handle menu click {id:?} — chrome::falloff_handle \
             never runs, so the Vector/Auto choice is silently dropped"
        );
        let actions = host.drained_actions();
        assert!(
            actions.is_empty(),
            "panel forwarded a spurious action for a chrome menu id: {actions:?}"
        );
    }
}

/// The Close (X) button is the OTHER fixed-chrome seam: it must push
/// `CancelActiveTool` (canon BgRemoval/Painter sidebar), not a ToolPanelEvent.
#[test]
fn close_button_forwards_cancel_active_tool() {
    let mut host = MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut panel_state = PainterLayersPanelState;

    let outcome = host.apply_panel_event::<PainterLayersPanel>(
        &mut panel_state,
        WidgetEvent::Click(core_ids::PAINTER_LAYERS_CLOSE),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "Close click was ignored — the `event.rs` arm for PAINTER_LAYERS_CLOSE is missing"
    );

    let actions = host.drained_actions();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EditorAction::CancelActiveTool)),
        "Close click never reached the bus as CancelActiveTool — the panel→shell seam is dead. \
         drained = {actions:?}"
    );
}
