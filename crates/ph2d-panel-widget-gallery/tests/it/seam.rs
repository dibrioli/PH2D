//! Behavioral SEAM test for the Widget Gallery panel's `apply_event` — the
//! test class the 2026-06-20 diagnosis found missing (blindagem Fase 0.1).
//!
//! `populate.rs` proves the close (X) button is registered and the
//! `*_contract_surface` gates count symbols, but NEITHER proves the
//! `apply_event` → `host.set_panel_visible` wire is intact. A forgotten
//! `GAL_CLOSE`/`TOPBAR_WIDGET_GALLERY` arm, a wrong panel id, or a dropped
//! `set_panel_visible` call all leave the button painted, clickable and
//! SILENTLY DEAD while every unit test + the contract gates stay green.
//!
//! This test runs the full path the desktop shell runs, headless:
//!   with_panel (populate) → set visible=true → apply_event(Click(GAL_CLOSE))
//!   → assert Consumed AND that `panel_visible(widget_gallery)` flipped off.
//!
//! NOTE: `event.rs` TOGGLES visibility (`next = !panel_visible(..)`), so the
//! observable effect of clicking close while visible is a flip to `false`.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::panel::Panel; // brings the `ID` const into scope
use ph2d_editor_core::panel::PanelHostInternal; // brings panel_visible / set_panel_visible into scope
use ph2d_panel_widget_gallery::WidgetGalleryPanel;
use ph2d_panel_widget_gallery::state::WidgetGalleryState;
use ph2d_ui_testkit::MockPanelHost;

/// Clicking the close (X) button while the gallery is visible must flip the
/// canonical visibility flag off through the panel's `apply_event` seam.
#[test]
fn gal_close_click_flips_visibility_off() {
    let mut host = MockPanelHost::with_panel::<WidgetGalleryPanel>();
    let mut state = WidgetGalleryState::default();

    // Precondition: panel is open. Without this the assertion below would be
    // vacuously satisfied by the default-false visibility map.
    host.set_panel_visible(WidgetGalleryPanel::ID, true);
    assert!(
        host.panel_visible(WidgetGalleryPanel::ID),
        "precondition: gallery must be visible before the close click"
    );

    // Drive the exact event the close (X) button emits.
    let outcome = host
        .apply_panel_event::<WidgetGalleryPanel>(&mut state, WidgetEvent::Click(ids::GAL_CLOSE));

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the GAL_CLOSE click — `event.rs` arm for GAL_CLOSE is missing"
    );

    // End-to-end proof: the canonical visibility flag actually flipped off.
    // This is precisely what populate + the contract gates cannot catch.
    assert!(
        !host.panel_visible(WidgetGalleryPanel::ID),
        "GAL_CLOSE was consumed but visibility never flipped — the apply_event \
         → set_panel_visible seam is dead"
    );
}

/// The TopBar palette pill toggles the gallery: from hidden, the same
/// `TOPBAR_WIDGET_GALLERY` click that opens it must drive visibility on
/// through the seam, proving the second id arm is wired too.
#[test]
fn topbar_pill_click_toggles_visibility_on() {
    let mut host = MockPanelHost::with_panel::<WidgetGalleryPanel>();
    let mut state = WidgetGalleryState::default();

    // Precondition: panel starts hidden (default), so the toggle opens it.
    assert!(
        !host.panel_visible(WidgetGalleryPanel::ID),
        "precondition: gallery must be hidden before the pill click"
    );

    let outcome = host.apply_panel_event::<WidgetGalleryPanel>(
        &mut state,
        WidgetEvent::Click(ids::TOPBAR_WIDGET_GALLERY),
    );

    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the TOPBAR_WIDGET_GALLERY click — `event.rs` arm for it is missing"
    );

    assert!(
        host.panel_visible(WidgetGalleryPanel::ID),
        "pill click was consumed but visibility never flipped on — the \
         apply_event → set_panel_visible seam is dead"
    );
}
