//! Gates do `TimelineViewSnapshot` — extraídos de `snapshot.rs` (HR-18 LOC cap).

use super::*;
use crate::TimelineIntent as Ix;
use crate::{PropKind, apply_intent};
use ph2d_anim::{AnimValue, RationalTime};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

#[test]
fn snapshot_projects_tracks_keys_selection_and_transport() {
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        Ix::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: s(0.0),
            value: AnimValue::Float(0.0),
            interp: Interp::Linear,
        },
    );
    apply_intent(&mut st, &mut ph, Ix::Pause);

    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks.len(), 1);
    assert_eq!(snap.tracks[0].prop, PropKind::TranslationX);
    assert_eq!(snap.tracks[0].keys.len(), 1);
    assert!(snap.tracks[0].keys[0].selected, "new key is selected");
    assert!(!snap.playing);

    // Rebuilding into the same snapshot reuses the buffers (no growth).
    let cap = snap.tracks[0].keys.capacity();
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks[0].keys.capacity(), cap, "key buffer reused");
}

#[test]
fn a_no_stack_clip_duration_makes_the_snapshot_explicit() {
    // The veil reads `view_length_explicit`; the panel publishes `keys_mode` as
    // `shows_keys() && stacked()`, so with no stack it hands `rebuild` FALSE even
    // on the Keys tab. The snapshot must STILL report the clip's authored Dur as
    // explicit (the clip is the timeline) so the dead zone darkens — the exact
    // gap of the re-smoke (Enio, 2026-07-23).
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    apply_intent(
        &mut st,
        &mut ph,
        Ix::AddKey {
            entity: 1,
            prop: PropKind::TranslationX,
            t: s(2.0),
            value: AnimValue::Float(10.0),
            interp: Interp::Linear,
        },
    );
    let mut snap = TimelineViewSnapshot::default();
    // keys_mode FALSE (no stack), no authored Dur yet → open-ended, no veil.
    snap.rebuild(&mut st, &ph, false);
    assert!(
        !snap.view_length_explicit,
        "a derived end never darkens the view"
    );
    // Author the CLIP's duration (the Keys-tab scope) → the snapshot closes even
    // with keys_mode false.
    st.doc.set_clip_length_override(0, Some(2.0));
    snap.rebuild(&mut st, &ph, false);
    assert!(
        snap.view_length_explicit,
        "an authored clip Dur must darken the no-stack view"
    );
    assert!(
        (snap.view_length_seconds - 2.0).abs() < 1e-9,
        "and the veil starts at the authored end the box shows"
    );
}

#[test]
fn a_missing_binding_paints_no_row() {
    // Deleting an object must take its rows off the panel this frame (the
    // data stays dormant in the document; healing brings the row back).
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for entity in [1u64, 2] {
        apply_intent(
            &mut st,
            &mut ph,
            Ix::AddKey {
                entity,
                prop: PropKind::TranslationX,
                t: s(0.0),
                value: AnimValue::Float(0.0),
                interp: Interp::Linear,
            },
        );
    }
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks.len(), 2, "both objects alive: two rows");

    // Entity 1's object dies (the apply pass flags it).
    st.doc.bindings_mut()[0].missing = true;
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks.len(), 1, "the dead object's row is gone");
    assert_eq!(snap.tracks[0].entity, 2, "the live row survived");
    assert_eq!(
        st.doc.bindings().len(),
        2,
        "the document keeps the dormant binding — hidden, not dropped"
    );

    // It heals (the object came back) → the row returns.
    st.doc.bindings_mut()[0].missing = false;
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks.len(), 2, "healed: the row is back");
}

/// **The snapshot shows each VIEW its own loop.** With a different loop parked in
/// each pair, rebuilding in `keys_mode` publishes the Keys loop; rebuilding in
/// Arrange publishes the timeline loop — the braces the panel draws follow the tab
/// (Enio, 2026-07-16). Read from the DOC, not the playhead, so a tab switch shows
/// the right loop before any sync runs.
#[test]
fn the_snapshot_publishes_the_views_own_loop() {
    let mut st = TimelineState::new();
    let ph = Playhead::new(1.0 / 60.0);
    st.doc.set_active_loop_for(false, Some((0.0, 2.0))); // Arrange
    st.doc.set_active_ping_pong_for(false, false);
    st.doc.set_active_loop_for(true, Some((1.5, 4.0))); // Keys
    st.doc.set_active_ping_pong_for(true, true);

    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, true);
    assert_eq!(
        snap.loop_range,
        Some((1.5, 4.0)),
        "Keys tab shows the clip loop"
    );
    assert!(snap.loop_ping_pong, "and its ping-pong");

    snap.rebuild(&mut st, &ph, false);
    assert_eq!(
        snap.loop_range,
        Some((0.0, 2.0)),
        "Arrange tab shows the timeline loop"
    );
    assert!(!snap.loop_ping_pong, "which wraps");
}
