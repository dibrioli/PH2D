use super::*;
use crate::state;
use ph2d_editor_core::interaction::GestureMods;
use ph2d_host::PointerButton;
use ph2d_timeline::TimelineIntent;

const SURFACE: ph2d_a11y::NodeId = ph2d_a11y::NodeId(0);

fn gesture(kind: TimelineHitKind, phase: GesturePhase, x: f32, shift: bool) -> TimelineGesture {
    gesture_at(kind, phase, x, 0.0, shift)
}

fn gesture_at(
    kind: TimelineHitKind,
    phase: GesturePhase,
    x: f32,
    y: f32,
    shift: bool,
) -> TimelineGesture {
    TimelineGesture {
        surface: SURFACE,
        kind,
        phase,
        x,
        y,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift,
            cmd: false,
            alt: false,
        },
    }
}

/// 60 fps, frame-snap on, 120 px/s — one frame = 2 px.
fn snap() -> TimelineViewSnapshot {
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        ..TimelineViewSnapshot::default()
    }
}

#[test]
fn a_selection_time_handle_routes_to_scale_not_a_strip_stretch() {
    // The precious fade lives on the STRIP surface (`StretchStrip`). The router
    // must send the key-selection grip to the KEY-scale machine and nowhere
    // near a strip — this is the one arch-gate the crown-jewels §4 asks for.
    // (Mutation: route `SelectionTimeHandle` to `strip_drag` here -> RED.)
    use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};
    // Default already parks the view at t=0.
    let mut st = TimelinePanelState::default();
    let key = |i: u64, t: f64| KeyView {
        id: KeyId::new(i),
        t_seconds: t,
        value: 0.0,
        interp: Interp::Linear,
        selected: true,
        roving: false,
    };
    let s = TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: false,
        tracks: vec![TrackView {
            target: AnimTarget::new(7),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            pre: ph2d_timeline::Extrap::Hold,
            post: ph2d_timeline::Extrap::Hold,
            expr: None,
            keys: vec![key(0, 0.0), key(1, 1.0)],
        }],
        ..TimelineViewSnapshot::default()
    };
    let handle = TimelineHitKind::SelectionTimeHandle { right: true };
    // `feed` maps x -> time with time_x = 0, so x = 300 is t = 3 (factor 3).
    feed(
        &mut st,
        gesture(handle, GesturePhase::Begin, 200.0, false),
        100.0,
        &s,
    );
    feed(
        &mut st,
        gesture(handle, GesturePhase::Update, 300.0, false),
        100.0,
        &s,
    );
    feed(
        &mut st,
        gesture(handle, GesturePhase::End, 300.0, false),
        100.0,
        &s,
    );
    let got = state::drain_intents();
    assert!(
        got.iter()
            .any(|i| matches!(i, TimelineIntent::ScaleSelectedKeys { .. })),
        "the router sends the grip to the key-scale machine"
    );
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::StretchStrip { .. })),
        "and NEVER to the strip surface, where the fade lives"
    );
}

#[test]
fn an_alt_key_drag_routes_to_stagger_not_a_plain_move() {
    // Alt-drag on a key is the Quick-Offset cascade (§3): the router must send
    // it to the stagger machine (StaggerSelectedKeys), never the rigid key
    // move (MoveSelectedKeys). (Mutation: drop the `g.mods.alt` disjunct -> an
    // Alt drag emits MoveSelectedKeys -> RED.)
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: false,
        ..TimelineViewSnapshot::default()
    };
    let key = TimelineHitKind::Key { target: 1, key: 0 };
    let alt = |phase, x: f32| TimelineGesture {
        surface: SURFACE,
        kind: key,
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift: false,
            cmd: false,
            alt: true,
        },
    };
    feed(&mut st, alt(GesturePhase::Begin, 100.0), 100.0, &s);
    feed(&mut st, alt(GesturePhase::Update, 200.0), 100.0, &s);
    feed(&mut st, alt(GesturePhase::End, 200.0), 100.0, &s);
    let got = state::drain_intents();
    assert!(
        got.iter()
            .any(|i| matches!(i, TimelineIntent::StaggerSelectedKeys { .. })),
        "the router sends an Alt key-drag to the stagger machine"
    );
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::MoveSelectedKeys { .. })),
        "and NOT to the rigid key move"
    );
}

#[test]
fn a_ctrl_key_drag_also_routes_to_stagger_the_wm_safe_trigger() {
    // A KDE compositor grabs Alt+left-drag for window-move, so the app never
    // sees an Alt key-drag. Ctrl (`cmd`) is the WM-safe path and must reach the
    // SAME stagger machine. (Mutation: drop the `g.mods.cmd` disjunct -> a Ctrl
    // drag falls through to the rigid key move (MoveSelectedKeys) -> RED.)
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: false,
        ..TimelineViewSnapshot::default()
    };
    let key = TimelineHitKind::Key { target: 1, key: 0 };
    let ctrl = |phase, x: f32| TimelineGesture {
        surface: SURFACE,
        kind: key,
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift: false,
            cmd: true,
            alt: false,
        },
    };
    feed(&mut st, ctrl(GesturePhase::Begin, 100.0), 100.0, &s);
    feed(&mut st, ctrl(GesturePhase::Update, 200.0), 100.0, &s);
    feed(&mut st, ctrl(GesturePhase::End, 200.0), 100.0, &s);
    let got = state::drain_intents();
    assert!(
        got.iter()
            .any(|i| matches!(i, TimelineIntent::StaggerSelectedKeys { .. })),
        "the router sends a Ctrl key-drag to the stagger machine too"
    );
    assert!(
        !got.iter()
            .any(|i| matches!(i, TimelineIntent::MoveSelectedKeys { .. })),
        "and NOT to the rigid key move"
    );
}

// Run the real Primary router (bypassing the store drain, which needs a host)
// so these exercise the same dispatch the panel does, lock routing and all.
fn feed(
    state: &mut TimelinePanelState,
    g: TimelineGesture,
    px_per_s: f64,
    snap: &TimelineViewSnapshot,
) {
    dispatch_primary(state, 0.0, px_per_s, snap, g);
}

#[test]
fn a_twirl_click_toggles_the_tracks_graph_editor() {
    let mut st = TimelinePanelState::default();
    let twirl = TimelineHitKind::Twirl { target: 3 };
    assert!(!st.is_expanded(3));
    feed(
        &mut st,
        gesture(twirl, GesturePhase::Click, 0.0, false),
        120.0,
        &snap(),
    );
    assert!(st.is_expanded(3), "the row opened");
    feed(
        &mut st,
        gesture(twirl, GesturePhase::Click, 0.0, false),
        120.0,
        &snap(),
    );
    assert!(!st.is_expanded(3), "and closed again");
    assert_eq!(
        state::drain_intents(),
        vec![],
        "expansion is view state, not an edit"
    );
}

#[test]
fn pressing_a_twirl_without_releasing_leaves_the_row_alone() {
    // Only a Click counts: an accidental drag from the twirl must not toggle.
    let mut st = TimelinePanelState::default();
    let twirl = TimelineHitKind::Twirl { target: 3 };
    feed(
        &mut st,
        gesture(twirl, GesturePhase::Begin, 0.0, false),
        120.0,
        &snap(),
    );
    feed(
        &mut st,
        gesture(twirl, GesturePhase::End, 40.0, false),
        120.0,
        &snap(),
    );
    assert!(!st.is_expanded(3));
}

#[test]
fn the_duration_handle_routes_to_the_length_edit() {
    // The Primary router must send a DurationHandle gesture to `duration_drag`
    // (not swallow it): a drag on the veil edge authors a length. 120 px/s, so
    // the 4 s edge is at x = 480; grab it and drag to x = 240 (t = 2 s). Keys
    // tab (default) → the clip scope.
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        view_length_explicit: true,
        view_length_seconds: 4.0,
        // A stacked Keys view solos a clip → the clip scope. The scope keys on
        // `keys_mode`, not the tab (a no-stack Keys view edits the scene).
        keys_mode: true,
        ..snap()
    };
    feed(
        &mut st,
        gesture(
            TimelineHitKind::DurationHandle,
            GesturePhase::Begin,
            480.0,
            false,
        ),
        120.0,
        &s,
    );
    feed(
        &mut st,
        gesture(
            TimelineHitKind::DurationHandle,
            GesturePhase::Update,
            240.0,
            false,
        ),
        120.0,
        &s,
    );
    assert!(
        state::drain_intents().contains(&TimelineIntent::SetClipLength { len: Some(2.0) }),
        "the router must reach duration_drag and author the clip length"
    );
}

#[test]
fn collapsing_a_row_mid_drag_closes_the_handles_undo_bracket() {
    // The band is about to stop existing, so `resolve_drag` will never fire
    // again — leaving the bracket open would swallow the next atomic edit.
    let mut st = TimelinePanelState::default();
    st.toggle_expanded(3);
    st.handle_drag = Some(crate::state::HandleDrag {
        target: 3,
        key: 1,
        which: 0,
        x: 0.0,
        y: 0.0,
        range: None,
        ending: false,
    });
    let _ = state::drain_intents();
    st.toggle_expanded(3);
    assert!(st.handle_drag.is_none());
    assert_eq!(state::drain_intents(), vec![TimelineIntent::EndEdit]);
}

// ── column lock: press a key, grab its whole column (or not) ─────────────

/// Two tracks, each with a key at t = 0, so pressing either key's diamond
/// finds a two-key column. Track 0 key 1; track 5 key 7.
fn two_key_column() -> TimelineViewSnapshot {
    use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};
    let k = |id: u64| KeyView {
        id: KeyId::new(id),
        t_seconds: 0.0,
        value: 0.0,
        interp: Interp::Linear,
        selected: false,
        roving: false,
    };
    TimelineViewSnapshot {
        fps: 60.0,
        frame_snap: true,
        tracks: vec![
            TrackView {
                target: AnimTarget::new(0),
                prop: PropKind::TranslationX,
                entity: 1,
                missing: false,
                buffer_ghost: None,
                pre: ph2d_timeline::Extrap::Hold,
                post: ph2d_timeline::Extrap::Hold,
                expr: None,
                keys: vec![k(1)],
            },
            TrackView {
                target: AnimTarget::new(5),
                prop: PropKind::Opacity,
                entity: 1,
                missing: false,
                buffer_ghost: None,
                pre: ph2d_timeline::Extrap::Hold,
                post: ph2d_timeline::Extrap::Hold,
                expr: None,
                keys: vec![k(7)],
            },
        ],
        ..snap()
    }
}

#[test]
fn locked_pressing_a_key_grabs_its_whole_column() {
    // With the padlock CLOSED, pressing one key at t = 0 selects EVERY key
    // at t = 0, so a drag moves the vertical group — routed through the
    // Summary machine. (Open is the default — see the test below.)
    let mut st = TimelinePanelState {
        column_lock: true,
        ..TimelinePanelState::default()
    };
    let key = TimelineHitKind::Key { target: 0, key: 1 };
    feed(
        &mut st,
        gesture(key, GesturePhase::Begin, 10.0, false),
        120.0,
        &two_key_column(),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::ClearSelection,
            TimelineIntent::AddToSelection(ph2d_timeline::SelectedKey::new(0, 1)),
            TimelineIntent::AddToSelection(ph2d_timeline::SelectedKey::new(5, 7)),
        ],
        "locked: the whole column, not just the pressed key"
    );
}

#[test]
fn unlocked_pressing_a_key_grabs_only_that_key() {
    // THE default (Enio 2026-07-11): a key click selects just that key —
    // it must never silently grab the whole Summary column.
    let mut st = TimelinePanelState::default();
    assert!(!st.column_lock, "open by default");
    let key = TimelineHitKind::Key { target: 0, key: 1 };
    feed(
        &mut st,
        gesture(key, GesturePhase::Begin, 10.0, false),
        120.0,
        &two_key_column(),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(ph2d_timeline::SelectedKey::new(0, 1)),
        ],
        "unlocked: just the pressed key — the other track stays put"
    );
}

#[test]
fn a_locked_press_on_a_key_with_no_column_falls_back_to_the_key_itself() {
    // Snapshot lag: the pressed key isn't in the published snapshot. Rather
    // than doing nothing, treat it as a plain key press.
    let mut st = TimelinePanelState {
        column_lock: true,
        ..TimelinePanelState::default()
    };
    let key = TimelineHitKind::Key { target: 9, key: 9 };
    feed(
        &mut st,
        gesture(key, GesturePhase::Begin, 10.0, false),
        120.0,
        &two_key_column(),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(ph2d_timeline::SelectedKey::new(9, 9)),
        ],
    );
}

#[test]
fn clicking_the_padlock_toggles_the_column_lock() {
    let mut st = TimelinePanelState::default();
    assert!(!st.column_lock, "open by default");
    feed(
        &mut st,
        gesture(
            TimelineHitKind::SummaryLock,
            GesturePhase::Click,
            0.0,
            false,
        ),
        120.0,
        &snap(),
    );
    assert!(st.column_lock, "one click closes it");
    feed(
        &mut st,
        gesture(
            TimelineHitKind::SummaryLock,
            GesturePhase::Click,
            0.0,
            false,
        ),
        120.0,
        &snap(),
    );
    assert!(!st.column_lock, "another opens it");
    assert_eq!(
        state::drain_intents(),
        vec![],
        "the lock is view state, not an edit"
    );
}

#[test]
fn pressing_the_padlock_without_releasing_does_not_toggle() {
    // Only a Click counts, so a stray drag from the padlock leaves it alone.
    let mut st = TimelinePanelState::default();
    feed(
        &mut st,
        gesture(
            TimelineHitKind::SummaryLock,
            GesturePhase::Begin,
            0.0,
            false,
        ),
        120.0,
        &snap(),
    );
    assert!(!st.column_lock, "a press is not a toggle (stays open)");
}
