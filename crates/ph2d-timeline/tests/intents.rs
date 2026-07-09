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

// ── W3 — undo bracket around a multi-frame gesture (graph-handle drag) ───────

/// The outgoing interpolation of entity 1's first TranslationX key.
fn first_interp(st: &TimelineState) -> Interp {
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    st.doc.active_clip().track(target).unwrap().keys()[0].interp
}

fn set_interp(st: &mut TimelineState, ph: &mut Playhead, interp: Interp) {
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let key = st.doc.active_clip().track(target).unwrap().ids()[0];
    apply_intent(
        st,
        ph,
        I::SetInterp {
            target,
            key,
            interp,
        },
    );
}

#[test]
fn a_bracketed_gesture_is_one_undo_step_however_many_edits_it_streams() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    let before = first_interp(&st);

    // A handle drag: one SetInterp per frame, all inside one bracket.
    apply_intent(&mut st, &mut ph, I::BeginEdit);
    for i in 1..=5 {
        set_interp(
            &mut st,
            &mut ph,
            Interp::bezier(0.1 * f64::from(i), 0.9, 0.8, 0.2),
        );
    }
    apply_intent(&mut st, &mut ph, I::EndEdit);
    assert_eq!(first_interp(&st), Interp::bezier(0.5, 0.9, 0.8, 0.2));

    // ONE Ctrl+Z restores the pre-drag interpolation, not the 4th frame of it.
    st.undo();
    assert_eq!(
        first_interp(&st),
        before,
        "the drag collapsed to a single undo step"
    );
}

#[test]
fn a_bracket_that_changed_nothing_pushes_no_step() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    apply_intent(&mut st, &mut ph, I::BeginEdit);
    apply_intent(&mut st, &mut ph, I::EndEdit);
    // One undo goes back past the AddKey — the empty bracket added no step.
    st.undo();
    assert!(st.doc.bindings().is_empty());
}

#[test]
fn an_unmatched_end_edit_is_a_no_op() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    apply_intent(&mut st, &mut ph, I::EndEdit);
    assert!(!st.history.can_undo());
}

#[test]
fn undo_during_an_open_bracket_does_not_resurrect_a_stale_snapshot() {
    // Pointer capture lost mid-drag: the bracket is open, the user hits Ctrl+Z.
    // A later EndEdit must not commit the pre-drag doc on top of the undone one.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    two_selected_keys(&mut st, &mut ph);
    apply_intent(&mut st, &mut ph, I::BeginEdit);
    set_interp(&mut st, &mut ph, Interp::bezier(0.9, 0.1, 0.1, 0.9));
    apply_intent(&mut st, &mut ph, I::Undo);
    let after_undo = st.doc.clone();
    apply_intent(&mut st, &mut ph, I::EndEdit);
    assert_eq!(st.doc, after_undo, "EndEdit after an Undo changed the doc");
    assert!(!st.history.can_undo() || st.doc == after_undo);
}

#[test]
fn re_keying_an_instant_records_the_pose_and_keeps_the_easing() {
    // Auto-key (or K) on a key the author already eased in the graph editor must
    // capture the new pose without reverting the curve to the default.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    let custom = Interp::bezier(0.9, 1.4, 0.1, -0.4);
    set_interp(&mut st, &mut ph, custom);

    // The auto-key path: same `t`, new value, the shell's default interp.
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 42.0);

    assert_eq!(key_times(&st), vec![0.0], "no duplicate key stacked");
    assert_eq!(value_at(&st, 0.0), Some(42.0), "the pose was captured");
    assert_eq!(first_interp(&st), custom, "the authored ease survived");
}

#[test]
fn a_graph_anchor_drag_moves_and_retunes_in_one_undo_step() {
    // The exact intent stream `ph2d-panel-timeline::anchor_drag` emits for a drag
    // that goes both sideways (time, whole selection) and upward (value, this
    // track): a bracket, a streamed move per frame, and one absolute SetKeyValue
    // per grabbed key rebuilt from the value captured at Begin. Undo must take the
    // WHOLE gesture back — both axes — in one step.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 1.0, 9.0);
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .unwrap()
        .target;
    let key = st.doc.active_clip().track(target).unwrap().ids()[0];
    apply_intent(
        &mut st,
        &mut ph,
        I::SelectSingle(SelectedKey { target, key }),
    );
    let before = st.doc.clone();
    let undo_depth = |st: &TimelineState| st.history.can_undo();
    assert!(undo_depth(&st));

    apply_intent(&mut st, &mut ph, I::BeginEdit);
    // Three frames of one drag: +0.1 s then +0.15 s of time (accrued deltas), and
    // the value rebuilt each frame from base 5.0 + the pointer's total offset.
    for (dt, v) in [(0.1, 5.5), (0.15, 6.25), (0.0, 6.5)] {
        if dt != 0.0 {
            apply_intent(&mut st, &mut ph, I::MoveSelectedKeys { delta_seconds: dt });
        }
        apply_intent(
            &mut st,
            &mut ph,
            I::SetKeyValue {
                target,
                key,
                value: AnimValue::Float(v),
            },
        );
    }
    apply_intent(&mut st, &mut ph, I::EndEdit);

    assert_eq!(key_times(&st), vec![0.25, 1.0], "the streamed moves summed");
    assert_eq!(value_at(&st, 0.25), Some(6.5), "the last value won");
    assert_eq!(
        first_interp(&st),
        Interp::Linear,
        "retuning a value must not touch the segment's easing"
    );

    // One Ctrl+Z, and BOTH axes go back — a per-frame undo step would need six.
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(st.doc, before, "the whole drag is one undo step");
}

#[test]
fn an_anchor_drag_that_moved_nothing_commits_no_undo_step() {
    // Press, no motion, release: the bracket opens and closes having changed
    // nothing. A step here would make every stray click cost a Ctrl+Z.
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(DT);
    add_key(&mut st, &mut ph, 1, PropKind::TranslationX, 0.0, 5.0);
    let before = st.doc.clone();
    apply_intent(&mut st, &mut ph, I::BeginEdit);
    apply_intent(&mut st, &mut ph, I::EndEdit);
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_ne!(
        st.doc, before,
        "Undo skipped past the empty bracket to the AddKey"
    );
}
