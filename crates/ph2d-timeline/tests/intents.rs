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

// ── W2.E7 — clipboard (copy / cut / paste) ───────────────────────────────────

/// Bind entity 1's TranslationX and key it at `0.0` and `0.5`, both selected.
fn two_selected_keys(st: &mut TimelineState, ph: &mut Playhead) {
    add_key(st, ph, 1, PropKind::TranslationX, 0.0, 5.0);
    add_key(st, ph, 1, PropKind::TranslationX, 0.5, 9.0);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let ids: Vec<_> = st.doc.active_clip().track(target).unwrap().ids().to_vec();
    apply_intent(
        st,
        ph,
        I::SelectSingle(SelectedKey {
            target,
            key: ids[0],
        }),
    );
    apply_intent(
        st,
        ph,
        I::AddToSelection(SelectedKey {
            target,
            key: ids[1],
        }),
    );
    assert_eq!(st.selection.len(), 2);
}

fn key_times(st: &TimelineState) -> Vec<f64> {
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    st.doc
        .active_clip()
        .track(target)
        .unwrap()
        .keys()
        .iter()
        .map(|k| k.t.to_seconds())
        .collect()
}

#[test]
fn copy_then_paste_at_the_playhead_preserves_the_group_timing() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);

    apply_intent(&mut st, &mut ph, I::CopySelection);
    assert_eq!(st.clipboard.len(), 2);
    let offsets: Vec<f64> = st
        .clipboard
        .keys()
        .iter()
        .map(|k| k.offset_seconds)
        .collect();
    assert_eq!(
        offsets,
        vec![0.0, 0.5],
        "rebased to the earliest copied key"
    );

    // Scrub away and paste: the group lands at the playhead, same 0.5 s spacing.
    apply_intent(&mut st, &mut ph, I::Scrub(2.0));
    apply_intent(&mut st, &mut ph, I::Paste);
    assert_eq!(key_times(&st), vec![0.0, 0.5, 2.0, 2.5]);
    assert_eq!(
        st.selection.len(),
        2,
        "the pasted keys become the selection"
    );
    // Values rode along with the copy.
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let tr = st.doc.active_clip().track(target).unwrap();
    assert_eq!(tr.keys()[2].value, AnimValue::Float(5.0));
    assert_eq!(tr.keys()[3].value, AnimValue::Float(9.0));
}

#[test]
fn paste_is_a_single_undo_step() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    apply_intent(&mut st, &mut ph, I::CopySelection);
    apply_intent(&mut st, &mut ph, I::Scrub(2.0));
    apply_intent(&mut st, &mut ph, I::Paste);
    assert_eq!(key_times(&st).len(), 4);

    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        key_times(&st),
        vec![0.0, 0.5],
        "one undo removes the whole paste"
    );
    // The clipboard survives undo — it is not part of the document.
    assert_eq!(st.clipboard.len(), 2);
}

#[test]
fn cut_copies_then_deletes_in_one_undo_step() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);

    apply_intent(&mut st, &mut ph, I::CutSelection);
    assert_eq!(st.clipboard.len(), 2, "cut fills the clipboard");
    assert!(key_times(&st).is_empty(), "cut removed the keys");
    assert!(st.selection.is_empty());

    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(
        key_times(&st),
        vec![0.0, 0.5],
        "one undo restores the cut keys"
    );
}

#[test]
fn paste_with_an_empty_clipboard_is_a_no_op() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    let undos_before = st.history.can_undo();
    apply_intent(&mut st, &mut ph, I::Paste);
    assert_eq!(key_times(&st), vec![0.0], "nothing pasted");
    assert_eq!(st.history.can_undo(), undos_before);
    // And one undo still lands on the AddKey, not on an empty paste step.
    apply_intent(&mut st, &mut ph, I::Undo);
    assert!(st.doc.binding_for(1, PropKind::TranslationX).is_none());
}

#[test]
fn copy_with_no_selection_keeps_the_previous_clipboard() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    apply_intent(&mut st, &mut ph, I::CopySelection);
    assert_eq!(st.clipboard.len(), 2);

    apply_intent(&mut st, &mut ph, I::ClearSelection);
    apply_intent(&mut st, &mut ph, I::CopySelection);
    assert_eq!(
        st.clipboard.len(),
        2,
        "an empty copy must not clobber a good clipboard"
    );
}

// ── "The end" of a clip (go-to-end / default loop range) ─────────────────────

#[test]
fn end_seconds_follows_the_last_key_past_a_zero_duration() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    // A fresh clip has no authored duration: "the end" is t=0 until keys exist.
    assert_eq!(st.doc.end_seconds(), 0.0);

    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 2.5, 9.0);
    // Another track, ending earlier — the max across tracks wins.
    add_key(&mut st, &mut ph, 2, PropKind::Rotation, 1.0, 1.0);
    assert_eq!(
        st.doc.end_seconds(),
        2.5,
        "go-to-end lands on the last keyframe, not on the 0 duration"
    );
}

#[test]
fn end_seconds_respects_an_authored_duration_that_outlasts_the_keys() {
    use ph2d_anim::RationalTime;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 1.0, 5.0);
    st.doc
        .active_clip_mut()
        .set_duration(RationalTime::from_seconds(10.0));
    assert_eq!(
        st.doc.end_seconds(),
        10.0,
        "authored duration wins when longer"
    );
}

// ── W2.E7 — duplicate (Ctrl+D) ───────────────────────────────────────────────

/// The value of the key at `t` on entity 1's TranslationX track.
fn value_at(st: &TimelineState, t: f64) -> Option<f32> {
    let target = st.doc.binding_for(1, PropKind::TranslationX)?.target;
    st.doc
        .active_clip()
        .track(target)?
        .keys()
        .iter()
        .find(|k| (k.t.to_seconds() - t).abs() < 1e-9)
        .map(|k| match k.value {
            AnimValue::Float(v) => v,
            _ => panic!("expected a float key"),
        })
}

/// Two frames at the default 24 fps display rate.
const TWO_FRAMES: f64 = 2.0 / 24.0;

#[test]
fn duplicate_with_the_playhead_on_the_first_key_offsets_two_frames() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph); // keys at 0.0 and 0.5
    ph.seek(0.0); // on the first selected key

    apply_intent(&mut st, &mut ph, I::DuplicateSelection);

    let times = key_times(&st);
    assert_eq!(times.len(), 4);
    for (got, want) in times.iter().zip([0.0, TWO_FRAMES, 0.5, 0.5 + TWO_FRAMES]) {
        assert!((got - want).abs() < 1e-9, "{times:?}");
    }
}

#[test]
fn duplicate_lands_the_first_copy_on_the_playhead() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph); // keys at 0.0 and 0.5
    ph.seek(1.0);

    apply_intent(&mut st, &mut ph, I::DuplicateSelection);

    assert_eq!(
        key_times(&st),
        vec![0.0, 0.5, 1.0, 1.5],
        "the group keeps its internal timing, anchored at the playhead"
    );
}

#[test]
fn duplicate_before_the_selection_walks_the_copies_left() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 1.0, 5.0);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 1.5, 9.0);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let ids: Vec<_> = st.doc.active_clip().track(target).unwrap().ids().to_vec();
    apply_intent(
        &mut st,
        &mut ph,
        I::SelectSingle(SelectedKey {
            target,
            key: ids[0],
        }),
    );
    apply_intent(
        &mut st,
        &mut ph,
        I::AddToSelection(SelectedKey {
            target,
            key: ids[1],
        }),
    );
    ph.seek(0.25);

    apply_intent(&mut st, &mut ph, I::DuplicateSelection);
    assert_eq!(key_times(&st), vec![0.25, 0.75, 1.0, 1.5]);
}

#[test]
fn a_duplicate_overwrites_the_key_it_lands_on() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph); // 0.0 -> 5.0, 0.5 -> 9.0
    ph.seek(0.5); // the first copy lands exactly on the second key

    apply_intent(&mut st, &mut ph, I::DuplicateSelection);

    assert_eq!(
        key_times(&st),
        vec![0.0, 0.5, 1.0],
        "three keys, not four — the copy replaced the key at 0.5"
    );
    assert_eq!(
        value_at(&st, 0.5),
        Some(5.0),
        "0.5 now holds the copy's value"
    );
    assert_eq!(value_at(&st, 1.0), Some(9.0));
    assert_eq!(value_at(&st, 0.0), Some(5.0), "the source is untouched");
}

#[test]
fn duplicate_selects_the_copies_so_the_next_drag_moves_them() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let originals: Vec<_> = st.doc.active_clip().track(target).unwrap().ids().to_vec();
    ph.seek(1.0);

    apply_intent(&mut st, &mut ph, I::DuplicateSelection);

    assert_eq!(st.selection.len(), 2, "the selection is the two COPIES");
    for key in &originals {
        assert!(
            !st.selection.contains(SelectedKey { target, key: *key }),
            "an original stayed selected — a follow-up drag would move it, \
             leaving the duplicate behind"
        );
    }
    apply_intent(&mut st, &mut ph, I::MoveSelectedKeys { delta_seconds: 1.0 });
    assert_eq!(key_times(&st), vec![0.0, 0.5, 2.0, 2.5]);
}

#[test]
fn duplicate_is_one_undo_step() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    ph.seek(1.0);
    apply_intent(&mut st, &mut ph, I::DuplicateSelection);
    assert_eq!(key_times(&st).len(), 4);
    st.undo();
    assert_eq!(
        key_times(&st),
        vec![0.0, 0.5],
        "one step undoes both copies"
    );
}

#[test]
fn duplicate_with_no_selection_changes_nothing() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    apply_intent(&mut st, &mut ph, I::ClearSelection);
    apply_intent(&mut st, &mut ph, I::DuplicateSelection);
    assert_eq!(key_times(&st), vec![0.0]);
    assert_eq!(st.selection.len(), 0);
    // The no-op must not have pushed an undo step of its own: one undo goes all
    // the way back past the AddKey, to the empty document.
    st.undo();
    assert!(
        st.doc.bindings().is_empty(),
        "a no-op duplicate polluted the undo stack"
    );
}

#[test]
fn a_frame_snapped_move_lands_exactly_on_the_frame() {
    // The regression that made overwrite-on-duplicate unreliable: a key dragged
    // by two frames used to land a fraction of a microsecond off the frame, so
    // a later duplicate onto it inserted a second key instead of replacing it.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    apply_intent(
        &mut st,
        &mut ph,
        I::MoveSelectedKeys {
            delta_seconds: TWO_FRAMES,
        },
    );
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let t = st.doc.active_clip().track(target).unwrap().keys()[0].t;
    assert_eq!(
        t,
        RationalTime::from_frame(2, 24),
        "a two-frame drag must be exactly two frames, not 83333 us"
    );
}
