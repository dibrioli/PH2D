//! Clip authoring on the document (W5 — NLA step 1): add / rename / delete /
//! switch, and the two invariants that keep the editor alive.

use ph2d_core::Playhead;
use ph2d_timeline::{
    MAX_CLIPS, PropKind, SelectedKey, TimelineIntent as I, TimelineState, apply_intent,
};

fn state() -> (TimelineState, Playhead) {
    (TimelineState::new(), Playhead::new(1.0 / 60.0))
}

#[test]
fn a_fresh_document_has_exactly_one_clip() {
    let (st, _) = state();
    assert_eq!(st.doc.clips().len(), 1);
    assert_eq!(st.doc.active_index(), 0);
}

#[test]
fn adding_a_clip_makes_it_active_and_names_it_uniquely() {
    let (mut st, mut ph) = state();
    apply_intent(&mut st, &mut ph, I::AddClip);
    assert_eq!(st.doc.clips().len(), 2);
    assert_eq!(st.doc.active_index(), 1, "a new clip is the one you edit");

    apply_intent(&mut st, &mut ph, I::AddClip);
    let names: Vec<&str> = st.doc.clips().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Main", "Clip 2", "Clip 3"]);
    // Two clips sharing a name would make the dropdown unreadable and the rename
    // ambiguous — the fresh name steps over whatever is taken.
    apply_intent(
        &mut st,
        &mut ph,
        I::RenameClip {
            index: 2,
            name: "Clip 4".to_string(),
        },
    );
    apply_intent(&mut st, &mut ph, I::AddClip);
    let names: Vec<&str> = st.doc.clips().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Main", "Clip 2", "Clip 4", "Clip 3"]);
}

#[test]
fn a_new_clip_is_empty_but_the_bindings_are_shared() {
    // THE model: bindings are document-wide, so every clip animates the same
    // objects and only the KEYS differ. That is what makes a second clip cost a
    // name and nothing else — "walk" and "run" over one rig.
    let (mut st, mut ph) = state();
    let e = 7;
    apply_intent(
        &mut st,
        &mut ph,
        I::Bind {
            entity: e,
            prop: PropKind::TranslationX,
        },
    );
    apply_intent(
        &mut st,
        &mut ph,
        I::AddKey {
            entity: e,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(1.0),
            value: ph2d_anim::AnimValue::Float(5.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    let target = st
        .doc
        .binding_for(e, PropKind::TranslationX)
        .unwrap()
        .target;
    assert_eq!(st.doc.active_clip().track(target).unwrap().len(), 1);

    apply_intent(&mut st, &mut ph, I::AddClip);
    assert!(
        st.doc.binding_for(e, PropKind::TranslationX).is_some(),
        "the binding survives the switch — the row is still there"
    );
    assert!(
        st.doc
            .active_clip()
            .track(target)
            .is_none_or(|t| t.is_empty()),
        "…but the new clip's curve is empty"
    );

    // And the first clip's keys are untouched, waiting where they were.
    apply_intent(&mut st, &mut ph, I::SetActiveClip { index: 0 });
    assert_eq!(st.doc.active_clip().track(target).unwrap().len(), 1);
}

#[test]
fn the_last_clip_is_never_deleted() {
    // A document with no clip would panic in `active_clip()` on the very next
    // frame — the refusal lives in the document, under every UI that might forget.
    let (mut st, mut ph) = state();
    apply_intent(&mut st, &mut ph, I::DeleteClip { index: 0 });
    assert_eq!(st.doc.clips().len(), 1, "the last clip stays");
    assert_eq!(st.doc.active_index(), 0);
}

#[test]
fn deleting_a_clip_keeps_the_active_index_pointing_at_a_real_clip() {
    let (mut st, mut ph) = state();
    apply_intent(&mut st, &mut ph, I::AddClip); // Clip 2 (active = 1)
    apply_intent(&mut st, &mut ph, I::AddClip); // Clip 3 (active = 2)

    // Delete the one BELOW the active: the active shifts down with it.
    apply_intent(&mut st, &mut ph, I::DeleteClip { index: 0 });
    assert_eq!(st.doc.clips().len(), 2);
    assert_eq!(st.doc.active_index(), 1, "still on Clip 3, now at index 1");
    assert_eq!(st.doc.clips()[1].name, "Clip 3");

    // Delete the ACTIVE one: it clamps to the clip below.
    apply_intent(&mut st, &mut ph, I::DeleteClip { index: 1 });
    assert_eq!(st.doc.clips().len(), 1);
    assert_eq!(st.doc.active_index(), 0);
    assert_eq!(st.doc.clips()[0].name, "Clip 2");
}

#[test]
fn switching_clips_clears_the_selection() {
    // A `KeyId` only means anything inside the track that issued it. Held across a
    // switch it would point at whatever key sits at that index in the NEW clip — a
    // stale selection that deletes the wrong keys.
    let (mut st, mut ph) = state();
    let e = 7;
    apply_intent(
        &mut st,
        &mut ph,
        I::Bind {
            entity: e,
            prop: PropKind::TranslationX,
        },
    );
    apply_intent(
        &mut st,
        &mut ph,
        I::AddKey {
            entity: e,
            prop: PropKind::TranslationX,
            t: ph2d_anim::RationalTime::from_seconds(1.0),
            value: ph2d_anim::AnimValue::Float(5.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    let target = st
        .doc
        .binding_for(e, PropKind::TranslationX)
        .unwrap()
        .target;
    let key = st.doc.active_clip().track(target).unwrap().ids()[0];
    st.selection.set_single(SelectedKey { target, key });
    assert!(!st.selection.is_empty());

    apply_intent(&mut st, &mut ph, I::AddClip);
    assert!(
        st.selection.is_empty(),
        "a clip switch drops the selection — its ids belong to the old clip"
    );
}

#[test]
fn the_document_refuses_more_clips_than_the_selector_can_address() {
    // The cap is not a guess at what an animator needs: the dropdown's option ids
    // are a fixed array, so a clip past MAX_CLIPS would paint an option nothing can
    // click. The DOCUMENT refuses it, so no UI has to remember to.
    let (mut st, mut ph) = state();
    for _ in 0..MAX_CLIPS + 5 {
        apply_intent(&mut st, &mut ph, I::AddClip);
    }
    assert_eq!(st.doc.clips().len(), MAX_CLIPS);
    assert!(
        st.doc.active_index() < MAX_CLIPS,
        "…and never points past it"
    );
}

#[test]
fn a_clip_edit_is_one_undo_step() {
    let (mut st, mut ph) = state();
    apply_intent(&mut st, &mut ph, I::AddClip);
    assert_eq!(st.doc.clips().len(), 2);
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(st.doc.clips().len(), 1, "one Ctrl+Z takes the clip back");

    // And the ACTIVE clip is part of the state: an undo that put the keys back but
    // left you looking at another clip would be an undo you cannot see.
    apply_intent(&mut st, &mut ph, I::Redo);
    assert_eq!(st.doc.active_index(), 1);
    apply_intent(&mut st, &mut ph, I::Undo);
    assert_eq!(st.doc.active_index(), 0);
}
