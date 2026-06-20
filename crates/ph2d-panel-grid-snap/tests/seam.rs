//! Behavioral SEAM test for the Grid Snap panel — proves the panel's
//! `apply_event` seam is alive end-to-end, headless (blindagem Fase 0.1).
//!
//! UNLIKE the image tools (Padding et al.), grid-snap has **no paired
//! `Tool`**: its `apply_event` mutates the canvas-renderer state directly
//! via `host.grid_snap_state_mut()`. So the seam to prove is NOT
//! "an action was pushed onto the bus" — it is the real post-condition:
//!
//!   populate(store) → drive a real WidgetEvent through apply_event
//!     → `host.grid_snap_state()` actually changed.
//!
//! A forgotten `apply_click` arm, a missing `apply_value_changed` branch,
//! a wrong id, or a broken split-borrow in `apply_event` all leave the
//! widget painted, clickable/draggable and SILENTLY DEAD while every unit
//! test + the `*_contract_surface` gates stay green. These tests run the
//! exact path the desktop shell runs, with no shell and no GPU.

use ph2d_editor_core::grid_snap::GridKind;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_grid_snap::state::GridSnapPanelState;
use ph2d_panel_grid_snap::{GridSnapPanel, ids};
use ph2d_ui_testkit::MockPanelHost;

/// Clicking the Hex grid-KIND option must flip the canvas-renderer
/// `GridSnapState::kind` to `Hex`. This is the cleanest driver: the click
/// is resolved directly in `apply_click` (no value-set, no bus).
#[test]
fn kind_hex_click_reaches_grid_snap_state() {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    let mut panel_state = GridSnapPanelState;

    // Precondition: GridSnapState::default().kind == Square, so a flip to
    // Hex is a genuine state change (guards against a vacuous assertion).
    assert_eq!(
        host.grid_snap_state().kind,
        GridKind::Square,
        "default grid kind unexpectedly not Square — Hex assertion would be vacuous"
    );

    let outcome = host.apply_panel_event::<GridSnapPanel>(
        &mut panel_state,
        WidgetEvent::Click(ids::GS_KIND_OPT_HEX),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Hex kind click — `event.rs` apply_click arm for GS_KIND_OPT_HEX is missing"
    );

    // End-to-end proof: the canvas-renderer state actually changed.
    assert_eq!(
        host.grid_snap_state().kind,
        GridKind::Hex,
        "Hex click never reached GridSnapState.kind — the apply_event → grid_snap_state_mut seam is dead"
    );
}

/// Dragging the opacity slider must land in `GridSnapState::opacity`.
/// Exercises the `ValueChanged` arm + the store→state slider read.
#[test]
fn opacity_slider_reaches_grid_snap_state() {
    let mut host = MockPanelHost::with_panel::<GridSnapPanel>();
    let mut panel_state = GridSnapPanelState;

    // Precondition: default opacity is 0.75, so 0.5 is a genuine change.
    assert!(
        (host.grid_snap_state().opacity - 0.75).abs() < 1e-6,
        "default opacity unexpectedly not 0.75 — pick a target distinct from the default"
    );

    // A drag writes the slider's stored value first, then dispatch emits
    // ValueChanged. Simulate both.
    host.set_slider_value(ids::GS_OPACITY_SLIDER, 0.5);
    let outcome = host.apply_panel_event::<GridSnapPanel>(
        &mut panel_state,
        WidgetEvent::ValueChanged(ids::GS_OPACITY_SLIDER),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored a real opacity edit — `event.rs` apply_value_changed arm for GS_OPACITY_SLIDER is missing"
    );

    // End-to-end proof: the canvas-renderer opacity actually changed.
    assert!(
        (host.grid_snap_state().opacity - 0.5).abs() < 1e-6,
        "opacity edit never reached GridSnapState.opacity (got {}) — slider→state seam is dead",
        host.grid_snap_state().opacity
    );
}
