//! W1.T5/T6 — the panel→runtime seam, headless: intents drive the document,
//! selection, transport (Playhead) and undo/redo, each doc edit one undo step.
//! This is the proof that the panel (W2) only needs to *emit* these.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Playhead;
use ph2d_timeline::{PropKind, SelectedKey, TimelineIntent as I, TimelineState, apply_intent};

const DT: f64 = 1.0 / 60.0;

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn add_key(
    state: &mut TimelineState,
    ph: &mut Playhead,
    entity: u64,
    prop: PropKind,
    t: f64,
    v: f32,
) {
    apply_intent(
        state,
        ph,
        I::AddKey {
            entity,
            prop,
            t: s(t),
            value: AnimValue::Float(v),
            interp: Interp::Linear,
        },
    );
}

#[test]
fn add_key_binds_creates_track_selects_and_is_one_undo_step() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);

    // Bound + track created + the new key is the sole selection.
    let b = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .expect("bound");
    assert_eq!(st.doc.active_clip().track(b.target).unwrap().len(), 1);
    assert_eq!(st.selection.len(), 1);
    assert!(st.history.can_undo());

    // One undo removes the whole gesture (back to empty doc).
    apply_intent(&mut st, &mut ph, I::Undo);
    assert!(st.doc.binding_for(1, PropKind::TranslationX).is_none());
    assert!(st.selection.is_empty());
    // Redo restores it.
    apply_intent(&mut st, &mut ph, I::Redo);
    assert!(st.doc.binding_for(1, PropKind::TranslationX).is_some());
}

#[test]
fn move_selected_keys_shifts_only_selection() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 0.0);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 1.0, 1.0);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;

    // Select only the key at t=1.0 (the 2nd), then shift +0.5 s.
    let track = st.doc.active_clip().track(target).unwrap();
    let key_at_1 = track
        .keys()
        .iter()
        .zip(track.ids())
        .find(|(k, _)| (k.t.to_seconds() - 1.0).abs() < 1e-9)
        .map(|(_, id)| *id)
        .unwrap();
    apply_intent(
        &mut st,
        &mut ph,
        I::SelectSingle(SelectedKey {
            target,
            key: key_at_1,
        }),
    );
    apply_intent(&mut st, &mut ph, I::MoveSelectedKeys { delta_seconds: 0.5 });

    let moved = st
        .doc
        .active_clip()
        .track(target)
        .unwrap()
        .key(key_at_1)
        .unwrap();
    assert!(
        (moved.t.to_seconds() - 1.5).abs() < 1e-4,
        "selected key shifted to 1.5"
    );
    // A no-op move (empty selection) is not a new undo step.
    apply_intent(&mut st, &mut ph, I::ClearSelection);
    let steps_before = st.history.can_undo();
    apply_intent(&mut st, &mut ph, I::MoveSelectedKeys { delta_seconds: 9.0 });
    assert_eq!(
        st.history.can_undo(),
        steps_before,
        "empty move added no step"
    );
}

#[test]
fn delete_selection_removes_keys() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::Rotation, 0.0, 0.0);
    add_key(&mut st, &mut ph, 1, PropKind::Rotation, 1.0, 1.0);
    let target = st.doc.binding_for(1, PropKind::Rotation).unwrap().target;
    // Select both.
    for id in st.doc.active_clip().track(target).unwrap().ids().to_vec() {
        apply_intent(
            &mut st,
            &mut ph,
            I::AddToSelection(SelectedKey { target, key: id }),
        );
    }
    apply_intent(&mut st, &mut ph, I::DeleteSelection);
    assert!(st.doc.active_clip().track(target).unwrap().is_empty());
    assert!(st.selection.is_empty());
}

#[test]
fn scrub_drives_playhead_with_frame_snap() {
    let mut st = TimelineState::new(); // frame_snap on by default, fps 24
    let mut ph = Playhead::new(DT);
    // 0.51 s at 24 fps snaps to frame 12 = 0.5 s.
    apply_intent(&mut st, &mut ph, I::Scrub(0.51));
    assert!(
        (ph.time() - 0.5).abs() < 1e-6,
        "snapped to frame 12 → 0.5s, got {}",
        ph.time()
    );
    // With snapping off it lands exactly.
    apply_intent(&mut st, &mut ph, I::SetFrameSnap(false));
    apply_intent(&mut st, &mut ph, I::Scrub(0.333));
    assert!((ph.time() - 0.333).abs() < 1e-9);
}

#[test]
fn transport_intents_toggle_and_loop() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    apply_intent(&mut st, &mut ph, I::Pause);
    assert!(!ph.is_playing());
    apply_intent(&mut st, &mut ph, I::TogglePlay);
    assert!(ph.is_playing());
    apply_intent(&mut st, &mut ph, I::SetLoop(Some((1.0, 2.0))));
    assert_eq!(ph.loop_range(), Some((1.0, 2.0)));
    apply_intent(&mut st, &mut ph, I::SetLoop(None));
    assert_eq!(ph.loop_range(), None);
}
