//! **A duração explícita** (Enio, 2026-07-23): o modelo composition-duration do AE —
//! um tamanho autorado por clip, por container e para o Arranje que define "o fim"
//! (go-to-end, o loop recém-armado, a barra do container, a fatia de uma instância
//! nova) e **CORTA o excedente sem destruí-lo**: keys e strips além do fim ficam
//! autorados, o avaliador só clampa o relógio no corte.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{
    ClipLane, ClipStrip, PropKind, StackHost, StripSource, TimelineDoc, apply_active_clip,
    apply_from_doc,
};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

fn scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(4.0),
        AnimValue::Float(40.0),
        Interp::Linear,
    );
    (world, doc, e)
}

fn x_of(world: &World, e: u64) -> f32 {
    world
        .get::<Transform>(Entity::from_bits(e))
        .unwrap()
        .translation
        .x
}

// ── the doors ───────────────────────────────────────────────────────────────

/// An authored duration IS the end — shorter than the content (the cut) and
/// longer than it (room to grow) both win over the derived answer; clearing
/// (`None`, or the numeric box's 0) restores it.
#[test]
fn an_authored_duration_wins_over_the_derived_end_in_all_three_scopes() {
    let (_, mut doc, _) = scene();
    // Clip: derived end is the last key (4.0).
    assert!((doc.clip_end_seconds(0) - 4.0).abs() < 1e-9);
    doc.set_clip_length_override(0, Some(2.5));
    assert!((doc.clip_end_seconds(0) - 2.5).abs() < 1e-9, "shorter cuts");
    doc.set_clip_length_override(0, Some(9.0));
    assert!(
        (doc.clip_end_seconds(0) - 9.0).abs() < 1e-9,
        "longer extends"
    );
    doc.set_clip_length_override(0, Some(0.0));
    assert!((doc.clip_end_seconds(0) - 4.0).abs() < 1e-9, "0 clears");

    // Scene: derived end is the stack's extent.
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 6.0, 4.0));
    doc.stack_mut().push(lane);
    assert!((doc.view_end_seconds(false) - 6.0).abs() < 1e-9);
    doc.set_scene_length(Some(3.0));
    assert!((doc.view_end_seconds(false) - 3.0).abs() < 1e-9);
    doc.set_scene_length(None);
    assert!((doc.view_end_seconds(false) - 6.0).abs() < 1e-9);

    // Container: derived is the interior's extent (empty = its born 2 s).
    assert_eq!(doc.add_container("C".to_string()), 0);
    assert!((doc.container_length_seconds(0) - 2.0).abs() < 1e-9);
    doc.set_container_length_override(0, Some(5.0));
    assert!((doc.container_length_seconds(0) - 5.0).abs() < 1e-9);
}

// ── the cut, through the real apply ─────────────────────────────────────────

/// The SCENE's authored duration cuts frame 0's clock: past it the pose holds
/// the cut's value instead of playing on — and a strip lying wholly beyond the
/// cut never plays. Non-destructive: clearing the length brings it all back.
#[test]
fn the_scene_cut_freezes_the_pose_at_the_authored_end() {
    let (mut world, mut doc, e) = scene();
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 4.0));
    doc.stack_mut().push(lane);
    doc.set_scene_length(Some(2.0));

    apply_from_doc(&mut world, &mut doc, 1.0);
    assert!(
        (x_of(&world, e) - 10.0).abs() < 1e-4,
        "inside the cut: plays"
    );
    apply_from_doc(&mut world, &mut doc, 3.0);
    let x = x_of(&world, e);
    assert!(
        (x - 20.0).abs() < 1e-4,
        "x = {x}: past the scene's authored end the clock holds the cut (20.0); \
         30.0 means the excess still plays"
    );

    doc.set_scene_length(None); // non-destructive: the content was never touched
    apply_from_doc(&mut world, &mut doc, 3.0);
    assert!(
        (x_of(&world, e) - 30.0).abs() < 1e-4,
        "clearing restores the excess"
    );
}

/// A CONTAINER's authored duration cuts every instance's interior clock — even
/// instances whose slice was windowed before the duration was authored.
#[test]
fn a_containers_cut_reaches_an_instance_placed_before_it() {
    let (mut world, mut doc, e) = scene();
    assert_eq!(doc.add_container("C".to_string()), 0);
    let mut inner = ClipLane::new("inner");
    inner.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 4.0));
    doc.container_stack_mut(0).unwrap().push(inner);
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Container(0), 0.0, 4.0, 4.0));
    doc.stack_mut().push(lane);

    apply_from_doc(&mut world, &mut doc, 3.0);
    assert!(
        (x_of(&world, e) - 30.0).abs() < 1e-4,
        "no cut: plays through"
    );

    doc.set_container_length_override(0, Some(2.0));
    apply_from_doc(&mut world, &mut doc, 3.0);
    let x = x_of(&world, e);
    assert!(
        (x - 20.0).abs() < 1e-4,
        "x = {x}: the container's authored end cuts its interior clock inside \
         the already-placed instance (holds 20.0); 30.0 means the cut never \
         reached the evaluator"
    );
}

/// The CLIP's authored duration cuts the Keys solo AND the no-stack solo path —
/// the same instant in both, because they are the same door.
#[test]
fn a_clips_cut_freezes_both_solo_paths_at_the_same_instant() {
    let (mut world, mut doc, e) = scene();
    doc.set_clip_length_override(0, Some(2.0));

    apply_from_doc(&mut world, &mut doc, 3.0); // empty stack: the solo path
    assert!(
        (x_of(&world, e) - 20.0).abs() < 1e-4,
        "no-stack solo holds the cut"
    );

    apply_active_clip(&mut world, &mut doc, 3.0, |_| false); // the Keys solo
    assert!(
        (x_of(&world, e) - 20.0).abs() < 1e-4,
        "Keys solo holds the same cut"
    );
}

/// **The AUTHOR's clock is cut too** (the 2026-07-23 superbug): past the authored
/// end the apply freezes every pose at `curve(cut)`, so the time a key authored
/// "now" lands at is the BOUNDARY — the frame the animator is looking at — never
/// the raw playhead, which the apply does not sample. This is the root-empty
/// lane of `key_home` (Keys, and Arrange with an empty stack — the one view
/// whose playhead clamp arm reads `scene_length` and so cannot stand in for the
/// clip's own cut).
#[test]
fn a_key_authored_beyond_the_cut_lands_on_the_boundary() {
    let (_, mut doc, e) = scene();
    doc.set_clip_length_override(0, Some(2.0));
    assert_eq!(
        ph2d_timeline::key_time(&doc, e, 3.0),
        Some(2.0),
        "beyond the cut a key lands ON the cut — the instant the apply froze at"
    );
    assert_eq!(
        ph2d_timeline::key_time(&doc, e, 1.0),
        Some(1.0),
        "within the cut: identity, as always"
    );
    doc.set_clip_length_override(0, None);
    assert_eq!(
        ph2d_timeline::key_time(&doc, e, 3.0),
        Some(3.0),
        "no authored duration: the raw clock, byte-identical to before"
    );
}

/// **Without a stack a clip's authored duration CLOSES the view** (Enio, 2026-07-23):
/// the veil asks `view_authored_end` (the playhead clamp that also asked it was REMOVED
/// 2026-07-25 — the playhead is free so the timeline can drive physics past its end), and
/// it must return the clip override even with `keys_mode = false` — which is what the panel publishes on
/// the Keys tab when nothing is arranged (`shows_keys() && stacked()`). The Dur(s) box
/// already showed the number (via `clip_end_seconds`); this is the door that makes the
/// two consumers agree. A REAL stack keeps the clip override out (the scene decides).
#[test]
fn a_no_stack_clip_duration_closes_the_view_regardless_of_keys_mode() {
    let (_, mut doc, _) = scene();
    // No stack, no scene_length, a clip override — the screenshot's state.
    doc.set_clip_length_override(0, Some(2.0));
    // The reported bug: keys_mode is FALSE (no stack), and the door must still close.
    assert_eq!(
        doc.view_authored_end(None, false),
        Some(2.0),
        "no stack: the clip IS the timeline, so its Dur closes the view even when \
         keys_mode is false"
    );
    // And on the stacked Keys path (keys_mode true) it was always right.
    assert_eq!(doc.view_authored_end(None, true), Some(2.0));
    // Clear it → open-ended again (a derived end never darkens).
    doc.set_clip_length_override(0, None);
    assert_eq!(doc.view_authored_end(None, false), None);

    // A REAL stack: the scene decides, and an UN-authored scene stays open even
    // though the clip below carries an override (the override never leaks up).
    doc.set_clip_length_override(0, Some(2.0));
    let mut lane = ClipLane::new("L");
    lane.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 6.0, 4.0));
    doc.stack_mut().push(lane);
    assert_eq!(
        doc.view_authored_end(None, false),
        None,
        "with a stack the scene's own (unset) length rules — the clip override is \
         not the scene's"
    );
    doc.set_scene_length(Some(3.0));
    assert_eq!(
        doc.view_authored_end(None, false),
        Some(3.0),
        "the scene's own"
    );
}

/// **A new clip and a new container open as 4 s compositions** (Enio, 2026-07-23),
/// authored through the intent (the shell's create path) — not the raw `add_clip`/
/// `add_container`, which stay derived for every test that leans on them. The veil
/// follows: an authored duration is a visible one.
#[test]
fn a_new_clip_and_container_open_at_the_default_four_seconds() {
    use ph2d_timeline::{
        DEFAULT_DURATION_SECONDS, TimelineIntent as I, TimelineState, apply_intent,
    };
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);

    // The raw constructor stays derived (the invariant the crate tests rely on).
    let raw = st.doc.add_clip("raw".into());
    assert_eq!(
        st.doc.clip_length_override(raw),
        None,
        "add_clip is the DATA layer — derived, no default"
    );

    // The AUTHORING intent stamps the 4 s default, and makes the new clip active.
    apply_intent(&mut st, &mut ph, I::AddClip);
    let active = st.doc.active_index();
    assert_eq!(
        st.doc.clip_length_override(active),
        Some(DEFAULT_DURATION_SECONDS),
        "a clip created by the artist opens at 4 s"
    );
    assert!(
        (st.doc.clip_end_seconds(active) - 4.0).abs() < 1e-9,
        "so the Dur box shows 4, not 0"
    );

    // A container the same — an empty one's 2 s bar is not the default, and without
    // an authored duration its veil never shows.
    apply_intent(&mut st, &mut ph, I::AddContainer);
    let c = st.doc.containers().len() - 1;
    assert_eq!(
        st.doc.container_length_override(c),
        Some(DEFAULT_DURATION_SECONDS),
        "a container opens at 4 s, so its veil is visible from the start"
    );
    assert!(
        (st.doc.container_length_seconds(c) - 4.0).abs() < 1e-9,
        "and the container Dur box shows 4, not the empty 2"
    );
}

// ── the loop never leaves the authored area ─────────────────────────────────

/// **Nenhum loop passa do fim autorado** (Enio, 2026-07-23): armar ou arrastar
/// uma brace além dele puxa a brace de volta; ENCOLHER a duração encolhe o loop
/// junto (e o playhead vivo resincroniza); um loop inteiro além do fim não
/// abraça nada e é LIMPO. Sem duração autorada a brace é livre, como sempre.
#[test]
fn no_loop_reaches_past_the_authored_duration() {
    use ph2d_timeline::{TimelineIntent as I, TimelineState, apply_intent};
    let mut st = TimelineState::new();
    let mut ph = ph2d_core::Playhead::new(1.0 / 60.0);

    // Sem Dur autorada: a brace fica onde foi posta.
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((0.0, 5.0)),
            ping_pong: false,
        },
    );
    assert_eq!(st.doc.active_loop_for(false), Some((0.0, 5.0)));

    // Dur da cena = 2: a brace existente encolhe na hora.
    apply_intent(&mut st, &mut ph, I::SetSceneLength { len: Some(2.0) });
    assert_eq!(st.doc.active_loop_for(false), Some((0.0, 2.0)));
    // E armar além do fim já nasce preso.
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((1.0, 9.0)),
            ping_pong: false,
        },
    );
    assert_eq!(st.doc.active_loop_for(false), Some((1.0, 2.0)));
    // Um loop inteiro além do fim não abraça nada: limpo.
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((3.0, 9.0)),
            ping_pong: false,
        },
    );
    assert_eq!(st.doc.active_loop_for(false), None);

    // O par do Keys responde ao CLIP, não à cena.
    st.keys_mode = true;
    apply_intent(
        &mut st,
        &mut ph,
        I::SetLoop {
            range: Some((0.0, 8.0)),
            ping_pong: false,
        },
    );
    assert_eq!(
        st.doc.active_loop_for(true),
        Some((0.0, 8.0)),
        "clip sem Dur: livre"
    );
    apply_intent(&mut st, &mut ph, I::SetClipLength { len: Some(1.0) });
    assert_eq!(st.doc.active_loop_for(true), Some((0.0, 1.0)));

    // E o loop de um CONTAINER responde à duração DELE.
    let mut st = TimelineState::new();
    let c = st.doc.add_container("C".to_string());
    apply_intent(
        &mut st,
        &mut ph,
        I::SetContainerLoop {
            container: c,
            range: Some((0.0, 6.0)),
            ping_pong: true,
        },
    );
    assert_eq!(st.doc.container_loop(c).0, Some((0.0, 6.0)));
    apply_intent(
        &mut st,
        &mut ph,
        I::SetContainerLength {
            container: c,
            len: Some(2.0),
        },
    );
    assert_eq!(st.doc.container_loop(c).0, Some((0.0, 2.0)));
}

// ── persistence ─────────────────────────────────────────────────────────────

/// The three overrides survive the round-trip (v11, appended fields).
#[test]
fn the_three_durations_survive_the_round_trip() {
    let (_, mut doc, _) = scene();
    assert_eq!(doc.add_container("C".to_string()), 0);
    doc.set_clip_length_override(0, Some(1.5));
    doc.set_container_length_override(0, Some(2.5));
    doc.set_scene_length(Some(3.5));

    let back = TimelineDoc::from_bytes(&doc.to_bytes().unwrap()).unwrap();
    assert!((back.clip_end_seconds(0) - 1.5).abs() < 1e-9);
    assert!((back.container_length_seconds(0) - 2.5).abs() < 1e-9);
    assert!((back.view_end_seconds(false) - 3.5).abs() < 1e-9);
}

/// A new instance of a cut container is SIZED to the cut — the strip the `+`
/// places, the bar the list draws and the slice the evaluator windows are one
/// number, through one door.
#[test]
fn a_new_instance_is_sized_to_the_containers_authored_duration() {
    let (_, mut doc, _) = scene();
    assert_eq!(doc.add_container("C".to_string()), 0);
    let mut inner = ClipLane::new("inner");
    inner.insert(ClipStrip::new(StripSource::Clip(0), 0.0, 4.0, 4.0));
    doc.container_stack_mut(0).unwrap().push(inner);
    doc.set_container_length_override(0, Some(2.0));
    doc.stack_mut().push(ClipLane::new("L"));

    let id = doc
        .add_strip_to(StackHost::Document, 0, StripSource::Container(0), 1.0, 3.0)
        .unwrap();
    let strip = doc.strip(0, id).unwrap();
    assert!(
        (strip.slice() - 2.0).abs() < 1e-9,
        "slice = {}: the strip's source window must be the container's authored \
         duration (2.0), not its interior's extent (4.0)",
        strip.slice()
    );
}
