use super::*;
use ph2d_editor_core::interaction::{GestureMods, TimelineHitKind};
use ph2d_host::PointerButton;

/// An Alt-held key gesture at `x`.
fn gesture(phase: GesturePhase, x: f32) -> TimelineGesture {
    TimelineGesture {
        surface: ph2d_a11y::NodeId(0),
        kind: TimelineHitKind::Key { target: 1, key: 0 },
        phase,
        x,
        y: 0.0,
        button: PointerButton::Primary,
        mods: GestureMods {
            shift: false,
            cmd: false,
            alt: true,
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

fn feed(state: &mut TimelinePanelState, g: TimelineGesture) {
    apply(state, 120.0, &snap(), SelectedKey::new(1, 0), g);
}

#[test]
fn an_alt_drag_streams_the_cascade_step_and_brackets_it_as_one_undo_step() {
    let mut st = TimelinePanelState::default();
    // Begin at x=100 (selects the key), drag to x=130 (30 px = 0.25 s at 120 px/s,
    // frame-aligned), release. The per-rank STEP is 0.25 s — one StaggerSelectedKeys.
    feed(&mut st, gesture(GesturePhase::Begin, 100.0));
    feed(&mut st, gesture(GesturePhase::Update, 130.0));
    feed(&mut st, gesture(GesturePhase::End, 130.0));
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(SelectedKey::new(1, 0)),
            TimelineIntent::StaggerSelectedKeys { step_seconds: 0.25 },
            TimelineIntent::EndEdit,
        ],
        "the step is emitted on the Update, not held to the release"
    );
    assert!(st.stagger_drag.is_none(), "the drag ended");
}

#[test]
fn a_continuing_drag_emits_only_the_step_that_accrued_since_the_last_frame() {
    // The cascade must follow the cursor and the streamed steps must SUM to the
    // drag: emitting the running total each frame would apply it twice.
    let mut st = TimelinePanelState::default();
    feed(&mut st, gesture(GesturePhase::Begin, 100.0));
    let _ = state::drain_intents();

    feed(&mut st, gesture(GesturePhase::Update, 130.0)); // +0.25 s
    assert_eq!(
        state::drain_intents(),
        vec![TimelineIntent::StaggerSelectedKeys { step_seconds: 0.25 }]
    );

    feed(&mut st, gesture(GesturePhase::Update, 142.0)); // total 0.35 s → owes +0.1
    let got = state::drain_intents();
    assert_eq!(got.len(), 1);
    let TimelineIntent::StaggerSelectedKeys { step_seconds } = got[0] else {
        panic!("{got:?}")
    };
    assert!((step_seconds - 0.1).abs() < 1e-9, "{step_seconds}");
    assert!((st.stagger_drag.unwrap().applied_step_s - 0.35).abs() < 1e-9);
}

#[test]
fn a_sub_frame_jitter_emits_no_step() {
    let mut st = TimelinePanelState::default();
    feed(&mut st, gesture(GesturePhase::Begin, 100.0));
    let _ = state::drain_intents();
    // < 1 px rounds to zero frames: nothing owed, nothing emitted.
    feed(&mut st, gesture(GesturePhase::Update, 100.5));
    assert_eq!(state::drain_intents(), vec![]);
}

#[test]
fn an_alt_click_without_dragging_collapses_a_preserved_group() {
    // Alt-press on an already-selected key preserves the group (so a drag would
    // cascade it); a plain click with no drag collapses to the pressed key and
    // the empty bracket commits no undo step.
    use ph2d_timeline::{AnimTarget, Interp, KeyId, KeyView, PropKind, TrackView};
    let mut st = TimelinePanelState::default();
    let s = TimelineViewSnapshot {
        tracks: vec![TrackView {
            target: AnimTarget::new(1),
            prop: PropKind::TranslationX,
            entity: 1,
            missing: false,
            buffer_ghost: None,
            keys: vec![KeyView {
                id: KeyId::new(0),
                t_seconds: 0.0,
                value: 0.0,
                interp: Interp::Linear,
                selected: true,
                roving: false,
            }],
        }],
        ..snap()
    };
    apply(&mut st, 120.0, &s, SelectedKey::new(1, 0), {
        let mut g = gesture(GesturePhase::Begin, 100.0);
        g.kind = TimelineHitKind::Key { target: 1, key: 0 };
        g
    });
    apply(
        &mut st,
        120.0,
        &s,
        SelectedKey::new(1, 0),
        gesture(GesturePhase::Click, 100.0),
    );
    assert_eq!(
        state::drain_intents(),
        vec![
            TimelineIntent::BeginEdit,
            TimelineIntent::SelectSingle(SelectedKey::new(1, 0)),
            TimelineIntent::EndEdit,
        ],
        "no SelectSingle from Begin (group preserved), collapse on the Click"
    );
    assert!(st.stagger_drag.is_none());
}
