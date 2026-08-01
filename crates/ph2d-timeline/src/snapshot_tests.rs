//! Gates do `TimelineViewSnapshot` — extraídos de `snapshot.rs` (HR-18 LOC cap).

use super::*;
use crate::TimelineIntent as Ix;
use crate::{PropKind, apply_intent};
use ph2d_anim::{AnimValue, RationalTime};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

#[test]
fn the_snapshot_carries_each_tracks_extrapolation() {
    // The data path for the dope-sheet marks (plan §6): a track's Pre/Post reach
    // the panel through the snapshot, independently. Default is Hold/Hold (no mark).
    use crate::{Extrap, ExtrapSide};
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
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .expect("binding")
        .target;

    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks[0].pre, Extrap::Hold, "default Pre is Hold");
    assert_eq!(snap.tracks[0].post, Extrap::Hold, "default Post is Hold");

    apply_intent(
        &mut st,
        &mut ph,
        Ix::SetTrackExtrap {
            target,
            side: ExtrapSide::Post,
            mode: Extrap::Loop,
        },
    );
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks[0].post, Extrap::Loop, "Post reaches the panel");
    assert_eq!(snap.tracks[0].pre, Extrap::Hold, "Pre stays independent");
}

#[test]
fn the_snapshot_carries_each_tracks_expression() {
    // The data path for the inline formula field (ADR-0144): a binding's `expr`
    // reaches the panel through the snapshot so the field can seed from it. Default
    // is None (keyframe-driven). (Mutation: rebuild not setting `row.expr` -> None.)
    use ph2d_anim::AnimTarget;
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
    let target = st
        .doc
        .binding_for(1, PropKind::TranslationX)
        .expect("binding")
        .target;

    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.tracks[0].expr, None, "default is keyframe-driven");

    apply_intent(
        &mut st,
        &mut ph,
        Ix::SetBindingExpr {
            target: AnimTarget::new(target.get()),
            expr: Some("time*10".to_string()),
        },
    );
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(
        snap.tracks[0].expr.as_deref(),
        Some("time*10"),
        "the expression reaches the panel through the snapshot"
    );
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
fn the_clip_and_scene_veils_are_independent_scopes() {
    // Bug 8 (Enio, 2026-07-27): editing the CLIP's duration must NOT move the SCENE's veil,
    // and vice versa. `keys_mode = shows_keys()` now (not `&& stacked()`), so the Keys tab
    // reads the clip's scope and Arrange reads the scene's — INDEPENDENT. The old rule
    // collapsed them without a stack, and a clip-Dur fallback closed the Arrange view too.
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
    // A derived doc: neither scope is authored → no veil on either tab.
    snap.rebuild(&mut st, &ph, true); // Keys
    assert!(
        !snap.view_length_explicit,
        "Keys: a derived clip end never darkens"
    );
    snap.rebuild(&mut st, &ph, false); // Arrange
    assert!(
        !snap.view_length_explicit,
        "Arrange: a derived scene end never darkens"
    );

    // Author ONLY the CLIP's duration (the Keys scope).
    st.doc.set_clip_length_override(0, Some(2.0));
    snap.rebuild(&mut st, &ph, true); // Keys sees it
    assert!(
        snap.view_length_explicit,
        "Keys: the clip's authored Dur closes the Keys view"
    );
    assert!(
        (snap.view_length_seconds - 2.0).abs() < 1e-9,
        "and the veil starts at the clip Dur (2)"
    );
    snap.rebuild(&mut st, &ph, false); // Arrange must NOT see it — independent scopes
    assert!(
        !snap.view_length_explicit,
        "Arrange stays open — a clip Dur is the Keys scope, not Arrange's (the coupling is gone)"
    );

    // Author the SCENE's duration → Arrange closes, at ITS number, independently of the clip.
    st.doc.set_scene_length(Some(5.0));
    snap.rebuild(&mut st, &ph, false); // Arrange
    assert!(
        snap.view_length_explicit,
        "Arrange: the scene's authored Dur closes the Arrange view"
    );
    assert!(
        (snap.view_length_seconds - 5.0).abs() < 1e-9,
        "and the Arrange veil is at the SCENE Dur (5), independent of the clip Dur (2)"
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

/// **O snapshot publica a costura como DUAS fatias de uma curva só** (Enio, 2026-08-01).
///
/// A cauda leva `[0, f]` na borda de SAÍDA e a cabeça `[f, 1]` na de ENTRADA — é o que faz
/// o painel desenhar uma curva que começa na fade final e termina na inicial. E ela existe
/// só sob um loop que ENVOLVE: sob ping-pong o playhead reflete e não há volta a desenhar.
#[test]
fn the_snapshot_publishes_the_seam_as_two_slices_of_one_curve() {
    use crate::TimelineViewSnapshot;
    let mut st = crate::TimelineState::new();
    let e = 7_u64;
    st.doc.insert_key(
        e,
        crate::PropKind::TranslationX,
        s(0.0),
        ph2d_anim::AnimValue::Float(0.0),
        ph2d_anim::Interp::Linear,
    );
    let c2 = st.doc.add_clip("B".into());
    let lane = st.doc.add_lane("L".into()).unwrap();
    let main = st.doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    let tail = st.doc.add_strip(lane, c2, 4.0, 7.5).unwrap();
    st.doc.strip_mut(lane, main).unwrap().lead_in = 0.5;
    st.doc.strip_mut(lane, tail).unwrap().lead_out = 1.5;
    st.doc.set_active_loop_for(false, Some((0.0, 9.0)));
    let ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let mut snap = TimelineViewSnapshot::default();

    snap.rebuild(&mut st, &ph, false);
    let strips = &snap.lanes[0].strips;
    // `f = 1,5 / (1,5 + 0,5) = 0,75` — assimétrico, então uma fatia trocada não passa.
    let head = strips[0].seam.expect("a cabeça carrega a 2ª fatia");
    let tail_slice = strips[1].seam.expect("a cauda carrega a 1ª");
    assert_eq!((head.edge, head.u0, head.u1), (0, 0.75, 1.0));
    assert_eq!(
        (tail_slice.edge, tail_slice.u0, tail_slice.u1),
        (1, 0.0, 0.75)
    );

    // …e sob PING-PONG não há travessia da volta: nenhuma fatia.
    st.doc.set_active_ping_pong_for(false, true);
    snap.rebuild(&mut st, &ph, false);
    assert!(
        snap.lanes[0].strips.iter().all(|s| s.seam.is_none()),
        "sob ping-pong o playhead reflete — não há costura a desenhar"
    );
}

/// **O desenho decorativo segue a direção do relógio** (Enio, 2026-08-01: *"e o desenho
/// decorativo da curva de easing na fade também"*).
///
/// O snapshot publica a curva EFETIVA do frame — que inclui a direção —, e é por isso que o
/// painel não precisa saber de nada: ele desenha o que a pose faz.
#[test]
fn the_published_curve_follows_the_clocks_direction() {
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    const EASE_IN: Easing = Easing {
        family: EasingFamily::Quint,
        mode: EasingMode::In,
    };
    let mut st = crate::TimelineState::new();
    let lane = st.doc.add_lane("L".into()).unwrap();
    let a = st.doc.add_strip(lane, 0, 0.5, 4.0).unwrap();
    {
        let s = st.doc.strip_mut(lane, a).unwrap();
        s.lead_in = 0.5;
        s.curve_in = Some(EASE_IN);
    }
    let ph = ph2d_core::Playhead::new(1.0 / 60.0);
    let mut snap = crate::TimelineViewSnapshot::default();

    snap.rebuild(&mut st, &ph, false);
    assert_eq!(snap.lanes[0].strips[0].curve_in, Some(EASE_IN));

    st.doc.set_reverse_play(true);
    snap.rebuild(&mut st, &ph, false);
    assert_eq!(
        snap.lanes[0].strips[0].curve_in,
        Some(EASE_IN.mirrored()),
        "andando para trás o desenho mostra o espelho — o que a pose faz"
    );
}
