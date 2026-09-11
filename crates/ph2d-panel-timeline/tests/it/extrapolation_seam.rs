//! Per-track EXTRAPOLATION menu, driven through the REAL panel seam (crown-jewels
//! plan §6). A cascade row opens the four-mode submenu; a mode row raises
//! `SetTrackExtrap`. And the Time-Remap track menu does NOT offer the cascades —
//! the "panel does not offer it there" half of the inertness, as a table fact.

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest, WidgetEvent};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

/// Publish one track and return its raw `AnimTarget` (mirrors `seam.rs`).
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

/// Click one menu row through the real seam; returns the outcome.
fn click(
    host: &mut MockPanelHost,
    state: &mut TimelinePanelState,
    id: ph2d_a11y::NodeId,
) -> EventOutcome {
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(id))
}

#[test]
fn a_post_cascade_opens_the_submenu_then_loop_sets_the_post_extrapolation() {
    use ph2d_timeline::{AnimTarget, Extrap, ExtrapSide, PropKind, TimelineIntent};
    let _ = ph2d_panel_timeline::drain_intents();
    let entity = 7;
    let target = publish_one_track(entity, PropKind::Rotation);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // The track menu was opened by a Secondary Down and parked by the next Down.
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 12.0,
        y: 34.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();

    // Click the "Extrapolation Post" cascade: it REPLACES the menu with the
    // four-mode submenu, carrying the target + the Post side.
    assert_eq!(
        click(&mut host, &mut state, ids::CTX_MENU_TL_EXTRAP_POST),
        EventOutcome::Consumed,
    );
    assert_eq!(
        host.store().context_menu().map(|r| r.kind),
        Some(ContextMenuKind::TimelineExtrap {
            target,
            side: ids::TL_EXTRAP_SIDE_POST,
        }),
        "the cascade must open the extrapolation submenu for the Post side"
    );

    // The next Down parks the submenu; the Loop leaf then raises SetTrackExtrap.
    host.store_mut().close_context_menu();
    assert_eq!(
        click(&mut host, &mut state, ids::CTX_MENU_TL_EXTRAP_LOOP),
        EventOutcome::Consumed,
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![TimelineIntent::SetTrackExtrap {
            target: AnimTarget::new(target),
            side: ExtrapSide::Post,
            mode: Extrap::Loop,
        }],
        "the Loop leaf must set the Post extrapolation of the row's track"
    );
    assert!(
        host.store().last_context_menu().is_none(),
        "the submenu request is spent"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn the_pre_cascade_routes_to_the_pre_side() {
    use ph2d_timeline::{AnimTarget, Extrap, ExtrapSide, PropKind, TimelineIntent};
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(3, PropKind::Opacity);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    click(&mut host, &mut state, ids::CTX_MENU_TL_EXTRAP_PRE);
    host.store_mut().close_context_menu();
    click(&mut host, &mut state, ids::CTX_MENU_TL_EXTRAP_CONTINUE);
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![TimelineIntent::SetTrackExtrap {
            target: AnimTarget::new(target),
            side: ExtrapSide::Pre,
            mode: Extrap::Continue,
        }],
        "the Pre cascade must route the mode to the Pre side"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn every_extrap_mode_row_is_handled_by_the_panel() {
    // The anti-dead-item gate for the submenu: a mode row added to
    // TIMELINE_EXTRAP_MENU without an event.rs arm is a painted item that does
    // nothing. Drive each under a parked TimelineExtrap request.
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(5, ph2d_timeline::PropKind::ScaleX);
    for (id, label, _) in ids::TIMELINE_EXTRAP_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        host.store_mut().open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::TimelineExtrap {
                target,
                side: ids::TL_EXTRAP_SIDE_POST,
            },
        });
        host.store_mut().close_context_menu();
        assert_eq!(
            click(&mut host, &mut state, id),
            EventOutcome::Consumed,
            "extrap mode row `{label}` is painted but has no event.rs arm"
        );
    }
    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn the_time_remap_menu_has_no_extrapolation_cascade() {
    // The exclusion as a TABLE fact: a plain track offers the two cascades; a Time
    // Remap track does not (its clock is its own, so extrapolation is inert). A row
    // offered but inert would be the dead-item bug the one-table-per-menu shape
    // exists to prevent.
    let has = |menu: &[(ph2d_a11y::NodeId, &str, Option<[u8; 4]>)], id| {
        menu.iter().any(|(row, _, _)| *row == id)
    };
    assert!(
        has(&ids::TIMELINE_TRACK_MENU, ids::CTX_MENU_TL_EXTRAP_PRE)
            && has(&ids::TIMELINE_TRACK_MENU, ids::CTX_MENU_TL_EXTRAP_POST),
        "the plain track menu offers both extrapolation cascades"
    );
    assert!(
        !has(
            &ids::TIMELINE_TIMEREMAP_TRACK_MENU,
            ids::CTX_MENU_TL_EXTRAP_PRE
        ) && !has(
            &ids::TIMELINE_TIMEREMAP_TRACK_MENU,
            ids::CTX_MENU_TL_EXTRAP_POST
        ),
        "the Time-Remap track menu must NOT offer extrapolation (inert there)"
    );
}
