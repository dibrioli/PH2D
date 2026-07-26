//! The property-EXPRESSION field (ADR-0144), driven through the REAL panel seam:
//! the track menu's "Expression\u{2026}" row OPENS the inline field, and a Submit
//! raises `SetBindingExpr` with the field text (or `None` to clear). The row is
//! offered on scene tracks but not on the Time-Remap track (its clock is not a
//! value to drive).

use ph2d_editor_core::ids;
use ph2d_editor_core::interaction::{
    ContextMenuKind, ContextMenuRequest, InteractiveState, WidgetEvent,
};
use ph2d_editor_core::panel::{EventOutcome, PanelHostInternal};
use ph2d_editor_core::widget::TextInputState;
use ph2d_panel_timeline::TimelinePanel;
use ph2d_panel_timeline::state::TimelinePanelState;
use ph2d_ui_testkit::MockPanelHost;

/// Publish one track and return its raw `AnimTarget` (mirrors `extrapolation_seam`).
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

/// Open+park the track menu for `target`, then click the "Expression\u{2026}" row.
fn open_field(host: &mut MockPanelHost, state: &mut TimelinePanelState, target: u64) -> EventOutcome {
    host.store_mut().open_context_menu(ContextMenuRequest {
        x: 40.0,
        y: 50.0,
        kind: ContextMenuKind::TimelineTrack { target },
    });
    host.store_mut().close_context_menu();
    host.apply_panel_event::<TimelinePanel>(state, WidgetEvent::Click(ids::CTX_MENU_TL_EXPR))
}

/// Register the field's live text (what typing would leave in the store).
fn type_into_field(host: &mut MockPanelHost, text: &str) {
    host.store_mut().register(
        ids::TIMELINE_TRACK_EXPR_INPUT,
        InteractiveState::TextInput {
            state: TextInputState::Focused,
            text: text.to_string(),
            caret: text.len(),
            selection_anchor: None,
        },
    );
}

#[test]
fn the_expression_menu_row_opens_the_field() {
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(7, ph2d_timeline::PropKind::TranslationX);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    assert_eq!(open_field(&mut host, &mut state, target), EventOutcome::Consumed);
    let ee = state.expr_edit.expect("the Expression row must open the field");
    assert_eq!(ee.target, target, "the field edits the clicked row's binding");
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn committing_the_field_raises_set_binding_expr() {
    use ph2d_timeline::{AnimTarget, TimelineIntent};
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(3, ph2d_timeline::PropKind::TranslationY);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    open_field(&mut host, &mut state, target);
    type_into_field(&mut host, "value + wiggle(3, 20)");
    let _ = ph2d_panel_timeline::drain_intents(); // opening may have pushed nothing; clear anyway
    let outcome = host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Submit(ids::TIMELINE_TRACK_EXPR_INPUT),
    );
    assert_eq!(outcome, EventOutcome::Consumed);
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![TimelineIntent::SetBindingExpr {
            target: AnimTarget::new(target),
            expr: Some("value + wiggle(3, 20)".to_string()),
        }],
        "Submit must raise SetBindingExpr with the field text"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn an_empty_field_clears_the_expression() {
    use ph2d_timeline::{AnimTarget, TimelineIntent};
    let _ = ph2d_panel_timeline::drain_intents();
    let target = publish_one_track(9, ph2d_timeline::PropKind::Rotation);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut state = TimelinePanelState::default();

    open_field(&mut host, &mut state, target);
    type_into_field(&mut host, "   "); // whitespace only -> clear
    let _ = ph2d_panel_timeline::drain_intents();
    host.apply_panel_event::<TimelinePanel>(
        &mut state,
        WidgetEvent::Submit(ids::TIMELINE_TRACK_EXPR_INPUT),
    );
    assert_eq!(
        ph2d_panel_timeline::drain_intents(),
        vec![TimelineIntent::SetBindingExpr {
            target: AnimTarget::new(target),
            expr: None,
        }],
        "an empty field clears the expression (back to keyframes)"
    );
    ph2d_panel_timeline::set_current_timeline(None);
}

#[test]
fn the_expression_row_is_offered_on_scene_tracks_but_not_time_remap() {
    // A table fact: every scene-track menu offers Expression; the Time-Remap menu
    // does not (a row offered but inert would be the dead-item bug).
    let has = |menu: &[(ph2d_a11y::NodeId, &str, Option<[u8; 4]>)]| {
        menu.iter().any(|(row, _, _)| *row == ids::CTX_MENU_TL_EXPR)
    };
    assert!(has(&ids::TIMELINE_TRACK_MENU), "plain track offers Expression");
    assert!(has(&ids::TIMELINE_AXIS_TRACK_MENU), "axis track offers Expression");
    assert!(has(&ids::TIMELINE_PATH_TRACK_MENU), "path track offers Expression");
    assert!(
        !has(&ids::TIMELINE_TIMEREMAP_TRACK_MENU),
        "the Time-Remap track must NOT offer Expression (its clock is not a value)"
    );
}
