//! **A live preview never outlives the card that asked for it** (FASE 0.3 do plano 12).
//!
//! The live preview is a formula the expression pass runs *as if* it were authored, so the
//! artist can watch the real object while tuning. It is installed by the shell every frame
//! from `expr_live_target()` and cleared by the card's own painter — which is the LAST call
//! of the panel's visible paint path.
//!
//! ⚠️ **So hiding the panel left it installed, forever.** Measured (auditoria 2026-07-29,
//! §4 D-K): with the channel standing, `x` walked 100 → 110 → 120 → 130 → 140 → 150 → 160,
//! **animating**, and `has_pending_restore()` was **false** the whole time — so the pose
//! was never owed back, and there was no UI on screen to stop it. The card's painter
//! names the failure it cannot prevent from where it stands: *"the panel can stop painting
//! the card by routes that never run `cancel` (**the panel hidden**, the timeline
//! closed)"*.
//!
//! ⚠️ **And no gate in this repo could reach that path**: every paint helper in
//! `ph2d-ui-testkit` forces the panel VISIBLE (it exists to read what the paint drew). The
//! hidden branch — where a panel drops its stale rects, its gestures, its published flags
//! — had no harness at all. `MockPanelHost::paint_hidden` is that harness, and it was born
//! with this gate.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

fn open_modal(host: &mut MockPanelHost, state: &mut TimelinePanelState, target: u64) {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR));
}

/// **Hiding the panel stops the preview.**
///
/// The premise is asserted first — with the card open and painting, something IS driving
/// the scene — because "nothing is installed" is trivially true of a channel that was
/// never filled, and a gate that cannot tell those apart proves nothing.
#[test]
fn hiding_the_panel_stops_the_live_preview() {
    let target = publish_one_track(21, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);

    // Put a row in the sheet so the projected formula is not the empty `value`.
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let driving = ph2d_panel_timeline::expr_live_target()
        .expect("PREMISE: an open card installs a live preview — otherwise this gate is vacuous");
    assert_eq!(
        driving.0, target,
        "it drives the binding the card is open on"
    );

    // Now hide the panel. The card is still in `state` — the artist did not cancel it,
    // they closed the timeline — which is precisely the route that skipped the clear.
    host.paint_hidden::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        ph2d_panel_timeline::expr_live_target().is_none(),
        "a hidden panel left the live formula installed: it would drive the object \
         forever, with nothing on screen able to stop it"
    );
}

/// **And it comes back when the panel does** — the other half, without which "clear it"
/// could be satisfied by never installing anything again.
#[test]
fn showing_the_panel_again_resumes_the_preview() {
    let target = publish_one_track(22, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("sway")),
    );

    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let before = ph2d_panel_timeline::expr_live_target().expect("visible: installed");
    host.paint_hidden::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        ph2d_panel_timeline::expr_live_target().is_none(),
        "hidden: cleared"
    );
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    let after = ph2d_panel_timeline::expr_live_target().expect("visible again: re-installed");

    assert_eq!(
        before, after,
        "re-showing the panel must resume the SAME preview — the card was never \
         cancelled, only hidden"
    );
}

/// **Cancelling the card clears it too**, so the two ways out of a preview agree.
///
/// ⚠️ This one already worked; it is here because it is the CONTROL for the gate above. If
/// the fix had been "clear the channel from somewhere that runs every frame regardless",
/// this would still pass while the panel-owns-the-card structure was gone.
#[test]
fn cancelling_the_card_clears_the_preview_as_well() {
    let target = publish_one_track(23, ph2d_timeline::PropKind::Rotation);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    open_modal(&mut host, &mut state, target);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::expr_gallery_id("shake")),
    );
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(ph2d_panel_timeline::expr_live_target().is_some());

    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::EXPR_MODAL_CANCEL));
    assert!(state.expr_modal.is_none(), "Cancel dismisses the card");
    host.paint::<TimelinePanel>(&mut state, VIEWPORT);
    assert!(
        ph2d_panel_timeline::expr_live_target().is_none(),
        "a dismissed card leaves nothing driving the scene"
    );
}
