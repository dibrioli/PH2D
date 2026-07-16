//! Behavioral SEAM test for the timeline panel transport → shell wire
//! (architecture_interactive_crate_has_behavioral_test).
//!
//! Unit tests cover `intent_for_transport` (the shell half) and `apply_intent`
//! (the runtime), but neither proves the panel's `event.rs` actually raises the
//! action on a real WidgetEvent. This drives the full seam headless:
//!   populate → WidgetEvent → apply_event → bus → assert the TimelinePanelEvent
//! (the exact payload the shell drain translates into a TimelineIntent).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::EventOutcome;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_ui_testkit::MockPanelHost;

fn timeline_events(host: &mut MockPanelHost) -> Vec<PanelEvent> {
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::TimelinePanelEvent(pe) => Some(pe),
            _ => None,
        })
        .collect()
}

#[test]
fn play_button_click_raises_transport_event() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    let outcome =
        host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_PLAY));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "panel ignored the Play click — event.rs arm missing"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Click(ids::TIMELINE_PLAY)],
        "Play click must raise TimelinePanelEvent(Click(PLAY)) for the shell to map to TogglePlay"
    );
}

#[test]
fn add_track_button_toggles_the_dropdown_locally() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert!(!state.add_track_open);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_ADD_TRACK),
    );
    assert!(state.add_track_open, "+Track opens the dropdown");
    // Toggling is panel-local — no shell event.
    assert!(timeline_events(&mut host).is_empty());
}

#[test]
fn add_track_prop_raises_click_and_closes_the_dropdown() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState {
        add_track_open: true,
        ..TimelinePanelState::default()
    };
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_ADDPROP_TX),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        !state.add_track_open,
        "picking a property closes the dropdown"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Click(ids::TIMELINE_ADDPROP_TX)],
        "a +Track prop click must reach the shell so it binds the selected sprite"
    );
}

#[test]
fn time_chip_edit_raises_set_value() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    host.set_number_value(ids::TIMELINE_TIME_NUM, 1.5);
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_TIME_NUM),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::SetValue(ids::TIMELINE_TIME_NUM, 1.5)],
        "seconds-chip edit must carry the real value for the shell to Scrub to it"
    );
}

#[test]
fn ruler_scrub_maps_value_to_time_and_raises_scrub() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    // Simulate what paint stored: 10 s visible from t=0. A drag to the middle
    // (0.5) must Scrub to 5 s.
    let mut state = TimelinePanelState {
        view_start_s: 0.0,
        view_span_s: 10.0,
        ..TimelinePanelState::default()
    };
    host.set_slider_value(ids::TIMELINE_RULER, 0.5);

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::ValueChanged(ids::TIMELINE_RULER),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::SetValue(ids::TIMELINE_RULER, 5.0)],
        "ruler scrub at 0.5 over a 10 s span must Scrub to 5 s"
    );
}

#[test]
fn speed_toggle_flips_the_view_locally() {
    // The speed-graph view is panel-local view state (like +Track), not a
    // document command: the Toggled event must flip `speed_view` and raise NO
    // shell event.
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert!(!state.speed_view, "value view by default");

    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SPEED));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Speed toggle arm is missing from event.rs"
    );
    assert!(state.speed_view, "Speed toggle flips the panel-local view");
    assert!(
        timeline_events(&mut host).is_empty(),
        "the speed view is not a document command — it must not reach the shell"
    );

    // A second toggle flips it back.
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SPEED));
    assert!(
        !state.speed_view,
        "toggling again returns to the value view"
    );
}

#[test]
fn snap_toggle_raises_toggle_event() {
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // Snap is registered on (default true); a Toggled event re-reads the store.
    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_SNAP));
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_SNAP, true)],
        "snap toggle must carry its on-state for the shell to SetFrameSnap"
    );
}

#[test]
fn record_toggle_raises_toggle_event() {
    // The Record (performing) toggle must reach the shell as a document command
    // (→ SetPerforming), NOT stay panel-local like the Speed view toggle.
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // Turn it on in the store, then fire the event (dispatch re-reads the store).
    host.set_toggle_on(ids::TIMELINE_RECORD, true);
    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Toggled(ids::TIMELINE_RECORD));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Record toggle arm is missing from event.rs"
    );
    assert_eq!(
        timeline_events(&mut host),
        vec![PanelEvent::Toggle(ids::TIMELINE_RECORD, true)],
        "Record must reach the shell so it arms performing (SetPerforming)"
    );
}

/// A real document with one bound `(entity, prop)` row, its snapshot published
/// — the same objects the shell hands the live panel. Returns the row's raw
/// `AnimTarget`.
fn publish_one_track(entity: u64, prop: ph2d_timeline::PropKind) -> u64 {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, TimelineIntent::Bind { entity, prop });
    let target = st.doc.binding_for(entity, prop).unwrap().target.get();
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    target
}

#[test]
fn delete_track_menu_click_raises_an_unbind_for_that_row() {
    // The production flow: the Secondary Down opened the menu, the NEXT Down
    // closed it (parking the request in `last_context_menu`) and only then the
    // Click on the row arrives — reading only the OPEN menu here is exactly how
    // a context-menu item ships doing nothing.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_timeline::PropKind;

    let _ = ph2d_panel_timeline::drain_intents(); // isolate from sibling tests
    let entity = 7;
    let target = publish_one_track(entity, PropKind::TranslationX);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu(); // Down-before-Click parks it

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_DELETE_TRACK),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the Delete Track arm is missing from event.rs"
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::Unbind {
            entity,
            prop: PropKind::TranslationX
        }],
        "Delete Track must raise an Unbind for the row's binding"
    );
    assert!(
        host.store().last_context_menu().is_none(),
        "the request is spent — a stray later Click must not delete again"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn every_track_menu_row_is_handled_by_the_panel() {
    // The anti-dead-item gate, executable: a row added to TIMELINE_TRACK_MENU
    // without an event.rs arm is a painted menu item that silently does
    // nothing. Drive each one through the real seam and demand the panel
    // consume it.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(3, ph2d_timeline::PropKind::Opacity);
    for (id, label, _) in ph2d_editor_core::ids::TIMELINE_TRACK_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        host.store_mut().open_context_menu(ContextMenuRequest {
            x: 0.0,
            y: 0.0,
            kind: ContextMenuKind::TimelineTrack { target },
        });
        host.store_mut().close_context_menu();
        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "track-menu row `{label}` is painted but has no event.rs arm"
        );
    }
    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn a_delete_for_a_row_gone_from_the_snapshot_expires_quietly() {
    // The row was deleted between the menu opening and the click landing (e.g.
    // an undo pulled the binding out) — the action's target is gone, so no
    // intent may fire, least of all against a RE-USED target id.
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None); // empty snapshot: row gone

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineTrack { target: 999 },
    });
    host.store_mut().close_context_menu();

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::CTX_MENU_TL_DELETE_TRACK),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "a dead row's delete must not raise an intent"
    );
}

// ── Clip selector (W5) ──────────────────────────────────────────────────────
//
// The clips travel the DIRECT intent channel (`drain_intents`), like Delete Track:
// the panel resolves everything from the snapshot, so the shell learns nothing new.
// These drive the real WidgetEvents; a missing arm in `event.rs` is a control that
// paints and does nothing, which no amount of `cargo check` would notice.

/// Publish a snapshot with `names` as the clips, `active` selected — the same
/// object the shell hands the live panel.
fn publish_clips(names: &[&str], active: usize) {
    use ph2d_timeline::{TimelineIntent, TimelineState, TimelineViewSnapshot, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    // A fresh doc already holds "Main"; add the rest.
    for _ in 1..names.len() {
        apply_intent(&mut st, &mut ph, TimelineIntent::AddClip);
    }
    for (i, n) in names.iter().enumerate() {
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::RenameClip {
                index: i,
                name: (*n).to_string(),
            },
        );
    }
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::SetActiveClip { index: active },
    );
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph);
    assert_eq!(snap.clips, names, "the fixture really holds those clips");
    ph2d_panel_timeline::set_current_timeline(Some(snap));
}

#[test]
fn picking_a_clip_from_the_list_switches_to_it_and_closes_the_list() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents(); // isolate from sibling tests
    publish_clips(&["Main", "Walk", "Run"], 0);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    // Open the list (the generic dispatch flips `open`; it raises no event).
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(ids::TIMELINE_CLIP_DD)
    {
        *open = true;
    }

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_CLIP_OPT[2]),
    );
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the clip-option arm is missing from event.rs"
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::SetActiveClip { index: 2 }],
        "clicking the third clip must switch to the third clip"
    );
    // The chip has to read right THIS frame — the document round-trip only lands
    // on the next one, so the store carries the pick.
    match host.store().get(ids::TIMELINE_CLIP_DD) {
        Some(InteractiveState::Dropdown {
            open,
            selected_index,
            ..
        }) => {
            assert!(!open, "picking a clip closes the list");
            assert_eq!(*selected_index, Some(2), "…and the chip shows the new clip");
        }
        _ => panic!("the clip dropdown is not registered in populate.rs"),
    }
}

#[test]
fn the_plus_button_raises_add_clip() {
    let _ = ph2d_panel_timeline::drain_intents();
    publish_clips(&["Main"], 0);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    let outcome = host
        .apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(ids::TIMELINE_ADD_CLIP));
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::AddClip]
    );
}

#[test]
fn the_pencil_opens_a_rename_field_seeded_on_the_active_clip() {
    let _ = ph2d_panel_timeline::drain_intents();
    publish_clips(&["Main", "Walk"], 1);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    assert!(state.clip_rename.is_none());

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_RENAME_CLIP),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    let cr = state
        .clip_rename
        .expect("the pencil opens the rename field");
    assert_eq!(cr.index, 1, "it renames the ACTIVE clip, not the first one");
    assert!(!cr.opened, "paint seeds + focuses it on the first frame");
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "opening the field is not yet a document edit"
    );
}

#[test]
fn the_trash_deletes_the_active_clip_but_never_the_last_one() {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    // THE guard. With one clip the paint does not even register the trash's hit —
    // but a dimmed control that still dispatches is exactly the failure that guard
    // is for, so a Click that reaches here anyway must still be refused. A document
    // with no clip would panic in `active_clip()` on the very next frame.
    publish_clips(&["Main"], 0);
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_DELETE_CLIP),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "the LAST clip is never deleted — a document must always have one to edit"
    );

    // With two, the active one goes.
    publish_clips(&["Main", "Walk"], 1);
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ids::TIMELINE_DELETE_CLIP),
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::DeleteClip { index: 1 }]
    );
}

#[test]
fn the_clip_cap_and_the_option_ids_are_the_same_number() {
    // THE gate the two halves need. A dropdown's option ids are a FIXED array —
    // the chrome cannot mint a hit id at runtime — so the number of clips the
    // DOCUMENT accepts must equal the number of ids the PANEL can address. Let the
    // doc grow past the array and the extra clip paints an option that nothing can
    // click: pintado mas inerte, and no compiler would say a word.
    assert_eq!(
        ph2d_timeline::MAX_CLIPS,
        ids::TIMELINE_CLIP_OPT.len(),
        "raise MAX_CLIPS and TIMELINE_CLIP_OPT together, or not at all"
    );
}

// ── The clip stack (ADR-0115) ───────────────────────────────────────────────

/// The twin of the clip-cap gate, for lanes. A lane's header carries hit ids from
/// a FIXED array — the chrome cannot mint one at runtime — so the number of lanes
/// the DOCUMENT accepts must equal the number the PANEL can address. Let the doc
/// grow past the array and the extra lane paints a mute button nothing can click:
/// pintado mas inerte, and no compiler would say a word.
#[test]
fn the_lane_cap_and_the_lane_header_ids_are_the_same_number() {
    assert_eq!(
        ph2d_timeline::MAX_LANES,
        ids::TIMELINE_LANE_MUTE.len(),
        "raise MAX_LANES and TIMELINE_LANE_MUTE together, or not at all"
    );
    assert_eq!(
        ph2d_timeline::MAX_LANES,
        ids::TIMELINE_LANE_ADD_STRIP.len(),
        "and TIMELINE_LANE_ADD_STRIP with them"
    );
}

/// **The anti-dead-item gate**, for the stack's chrome: every button the lane
/// rows paint must be answered by `event.rs`. A painted button with no arm is a
/// control that silently does nothing — and it is the failure this panel has
/// already shipped once.
#[test]
fn every_stack_button_is_handled_by_the_panel() {
    let mut ids_to_check = vec![ids::TIMELINE_ADD_LANE];
    ids_to_check.extend(ids::TIMELINE_LANE_MUTE);
    ids_to_check.extend(ids::TIMELINE_LANE_ADD_STRIP);

    for id in ids_to_check {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "a lane button the panel paints but never answers is a dead control"
        );
    }
}

/// "+ Lane" raises the intent, and the panel's own snapshot is what a lane button
/// resolves against — a lane deleted since the paint that registered its button
/// raises nothing, exactly as Delete Track does.
#[test]
fn add_lane_raises_its_intent_and_a_vanished_lane_raises_none() {
    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut st = TimelinePanelState::default();

    let _ = host
        .apply_panel_event::<TimelinePanel>(&mut st, WidgetEvent::Click(ids::TIMELINE_ADD_LANE));
    assert!(
        ph2d_panel_timeline::drain_intents()
            .iter()
            .any(|i| matches!(i, ph2d_timeline::TimelineIntent::AddLane)),
        "+ Lane raises AddLane"
    );

    // No snapshot published -> no lanes -> the mute of lane 0 has no target.
    let _ = host.apply_panel_event::<TimelinePanel>(
        &mut st,
        WidgetEvent::Click(ids::TIMELINE_LANE_MUTE[0]),
    );
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "a mute for a lane that is not there mutes nothing"
    );
}

// ── The strip's right-click menu (ADR-0115 B6) ──────────────────────────────

/// A published snapshot holding one lane with one strip. Returns the strip's id.
fn publish_one_strip() -> ph2d_timeline::StripId {
    use ph2d_timeline::{TimelineIntent as I, TimelineViewSnapshot, apply_intent};
    let mut st = ph2d_timeline::TimelineState::default();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, I::AddLane);
    apply_intent(
        &mut st,
        &mut ph,
        I::AddStrip {
            lane: 0,
            clip: 0,
            t_start: 0.0,
            t_end: 2.0,
        },
    );
    let id = st.doc.stack()[0].strips[0].id;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
    id
}

/// Park a strip menu the way production does: the Secondary Down opened it, the
/// next Down CLOSED it (leaving the request in `last_context_menu`), and only
/// then does the Click on the row arrive.
fn park_strip_menu(host: &mut MockPanelHost, strip: ph2d_timeline::StripId) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineStrip {
            lane: 0,
            strip: strip.0,
        },
    });
    host.store_mut().close_context_menu();
}

/// **The anti-dead-item gate for the strip menu — and it checks WHICH intent.**
///
/// Demanding only that the panel *consume* the click would pass a menu whose six
/// rows all quietly did the same thing: the loop-mode arm falls back to `Once`,
/// so a new row wired to nothing would land there and look handled. Each row must
/// therefore raise the intent its own label promises.
#[test]
fn every_strip_menu_row_raises_the_intent_its_label_promises() {
    use ph2d_editor_core::ids as c;
    use ph2d_timeline::{StripLoop, TimelineIntent as I};

    let _ = ph2d_panel_timeline::drain_intents();
    let strip = publish_one_strip();
    let lane = 0;

    let expected = |id: ph2d_editor_core::NodeId| -> I {
        if id == c::CTX_MENU_TL_STRIP_DUPLICATE {
            I::DuplicateStrip { lane, id: strip }
        } else if id == c::CTX_MENU_TL_STRIP_DELETE {
            I::RemoveStrip { lane, id: strip }
        } else if id == c::CTX_MENU_TL_STRIP_RESET_SPEED {
            I::SetStripSpeed {
                lane,
                id: strip,
                speed: 1.0,
            }
        } else if id == c::CTX_MENU_TL_STRIP_LOOP {
            I::SetStripLoop {
                lane,
                id: strip,
                loop_mode: StripLoop::Loop,
            }
        } else if id == c::CTX_MENU_TL_STRIP_PINGPONG {
            I::SetStripLoop {
                lane,
                id: strip,
                loop_mode: StripLoop::PingPong,
            }
        } else {
            I::SetStripLoop {
                lane,
                id: strip,
                loop_mode: StripLoop::Once,
            }
        }
    };

    for (id, label, _) in c::TIMELINE_STRIP_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        park_strip_menu(&mut host, strip);

        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "strip-menu row `{label}` is painted but has no event.rs arm"
        );
        assert_eq!(
            ph2d_panel_timeline::drain_intents(),
            vec![expected(id)],
            "strip-menu row `{label}` must raise the intent it names"
        );
        assert!(
            host.store().last_context_menu().is_none(),
            "row `{label}` left its request parked: a stray later Click would fire it again"
        );
    }
    ph2d_panel_timeline::set_current_timeline(None);
}

/// A strip deleted between the menu opening and the row being clicked resolves to
/// nothing. The id is stable, so it cannot be re-used — but the strip is gone, and
/// an intent naming a dead strip is an intent the document would have to guess at.
#[test]
fn a_strip_menu_click_for_a_vanished_strip_expires_quietly() {
    let _ = ph2d_panel_timeline::drain_intents();
    ph2d_panel_timeline::set_current_timeline(None); // no lanes at all

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    park_strip_menu(&mut host, ph2d_timeline::StripId(404));

    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Click(ph2d_editor_core::ids::CTX_MENU_TL_STRIP_DELETE),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "a dead strip's delete must not raise an intent"
    );
}

// ── The lane's right-click menu + its weight field (ADR-0115 B5) ────────────

/// A published snapshot with one lane (no strips).
fn publish_one_lane() {
    use ph2d_timeline::{TimelineIntent as I, TimelineViewSnapshot, apply_intent};
    let mut st = ph2d_timeline::TimelineState::default();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);
    apply_intent(&mut st, &mut ph, I::AddLane);
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph);
    ph2d_panel_timeline::set_current_timeline(Some(snap));
}

fn park_lane_menu(host: &mut MockPanelHost, lane: usize) {
    use ph2d_editor_core::interaction::{ContextMenuKind, ContextMenuRequest};
    use ph2d_editor_core::panel::PanelHostInternal;
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 0.0,
        y: 0.0,
        kind: ContextMenuKind::TimelineLane { lane },
    });
    host.store_mut().close_context_menu(); // the Down-before-Click parks it
}

/// Every lane-menu row raises the intent its label promises — including
/// **Delete Lane**, which until this row existed was an intent (`RemoveLane`) the
/// document could serve and no gesture could reach: a lane, once added, could not
/// be removed. A dead intent is the mirror image of a dead menu item, and neither
/// compiles red.
#[test]
fn every_lane_menu_row_raises_the_intent_its_label_promises() {
    use ph2d_editor_core::ids as c;
    use ph2d_timeline::{LaneMode, TimelineIntent as I};

    let _ = ph2d_panel_timeline::drain_intents();
    publish_one_lane();

    let expected = |id: ph2d_editor_core::NodeId| -> I {
        if id == c::CTX_MENU_TL_LANE_DELETE {
            I::RemoveLane { lane: 0 }
        } else if id == c::CTX_MENU_TL_LANE_ADDITIVE {
            I::SetLaneMode {
                lane: 0,
                mode: LaneMode::Additive,
            }
        } else {
            I::SetLaneMode {
                lane: 0,
                mode: LaneMode::Override,
            }
        }
    };

    for (id, label, _) in c::TIMELINE_LANE_MENU {
        let mut host = MockPanelHost::with_panel::<TimelinePanel>();
        let mut state = TimelinePanelState::default();
        park_lane_menu(&mut host, 0);

        let outcome = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Click(id));
        assert_eq!(
            outcome,
            EventOutcome::Consumed,
            "lane-menu row `{label}` is painted but has no event.rs arm"
        );
        assert_eq!(
            ph2d_panel_timeline::drain_intents(),
            vec![expected(id)],
            "lane-menu row `{label}` must raise the intent it names"
        );
        assert!(
            host.store().last_context_menu().is_none(),
            "row `{label}` left its request parked"
        );
    }
    ph2d_panel_timeline::set_current_timeline(None);
}

/// The weight field carries the lane's influence to the document — and a lane the
/// snapshot no longer has raises nothing, like every other stack control.
#[test]
fn the_weight_field_sets_the_lane_weight_and_a_vanished_lane_sets_none() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal;

    let _ = ph2d_panel_timeline::drain_intents();
    publish_one_lane();

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let id = ids::TIMELINE_LANE_WEIGHT[0];
    if let Some(InteractiveState::NumberInput { value, .. }) = host.store_mut().get_mut(id) {
        *value = 0.25;
    }
    let outcome =
        host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::ValueChanged(id));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "the weight field has no arm"
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![ph2d_timeline::TimelineIntent::SetLaneWeight {
            lane: 0,
            weight: 0.25
        }]
    );

    // Lane 1 was never in the snapshot: its field is registered (the store is
    // populated once, for all MAX_LANES) but it addresses nothing.
    let gone = ids::TIMELINE_LANE_WEIGHT[1];
    let _ = host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::ValueChanged(gone));
    assert!(
        ph2d_panel_timeline::drain_intents().is_empty(),
        "a weight for a lane that is not there weighs nothing"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

/// **One gesture, one Ctrl+Z.** Dispatch emits a `ValueChanged` for every Move of
/// a number body-drag; unbracketed, each one is its own atomic undo step, and
/// sliding the weight across its range left dozens of them behind. Every other
/// document-mutating gesture in this panel brackets — this was the one that did
/// not (audit, 2026-07-12).
#[test]
fn dragging_the_lane_weight_is_one_undo_step_not_one_per_frame() {
    use ph2d_editor_core::interaction::InteractiveState;
    use ph2d_editor_core::panel::PanelHostInternal;
    use ph2d_timeline::TimelineIntent as I;

    let _ = ph2d_panel_timeline::drain_intents();
    publish_one_lane();

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();
    let id = ids::TIMELINE_LANE_WEIGHT[0];

    // The pointer goes down on the field (Focus), scrubs it across three frames,
    // and lets go (Blur) — exactly what dispatch emits for a body drag.
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Focus(id));
    for v in [0.8, 0.5, 0.25] {
        if let Some(InteractiveState::NumberInput { value, .. }) = host.store_mut().get_mut(id) {
            *value = v;
        }
        host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::ValueChanged(id));
    }
    host.apply_panel_event::<TimelinePanel>(&mut state, WidgetEvent::Blur(id));

    let out = ph2d_panel_timeline::drain_intents();
    assert!(
        matches!(out.first(), Some(I::BeginEdit)),
        "the bracket opens on Focus: {out:?}"
    );
    assert!(
        matches!(out.last(), Some(I::EndEdit)),
        "and closes on Blur, folding the whole scrub into ONE undo step: {out:?}"
    );
    assert_eq!(
        out.iter()
            .filter(|i| matches!(i, I::BeginEdit | I::EndEdit))
            .count(),
        2,
        "exactly one bracket, however many frames the drag spans: {out:?}"
    );
    assert_eq!(
        out.iter()
            .filter(|i| matches!(i, I::SetLaneWeight { .. }))
            .count(),
        3,
        "and every frame's value still reaches the document (a live drag)"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}
