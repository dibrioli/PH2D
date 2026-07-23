//! Unit tests for [`super`] (`timeline_bridge.rs`), second half: **what K keys, and
//! on whose clock** — the entity's remapped time, a Time track's own seeding rules,
//! and solo mode. Split from `timeline_bridge_tests.rs` under the HR-18 shell LOC
//! cap; the transport half stays there.

use super::*;

#[test]
fn k_keys_scene_props_at_the_entity_remapped_clock() {
    use ph2d_anim::AnimValue::Float;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    // No remap: identity (a scene key lands at the playhead).
    let at = |st: &TimelineState, prop, t: f64| {
        key_insert_time(st, 1, prop, t)
            .expect("no stack: a key always has a home")
            .to_seconds()
    };
    assert_eq!(at(&st, PropKind::TranslationX, 1.0), 1.0);
    // 2x remap (0 → 0, 2 → 4): a TX key at playhead 1 lands at SOURCE 2 —
    // where the apply samples it — while the Time track itself keys at the
    // playhead (the map lives in playhead time).
    for (t, v) in [(0.0, 0.0f32), (2.0, 4.0)] {
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TimeRemap,
                t: ph2d_anim::RationalTime::from_seconds(t),
                value: Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    assert_eq!(at(&st, PropKind::TranslationX, 1.0), 2.0, "source time");
    assert_eq!(at(&st, PropKind::TimeRemap, 1.0), 1.0, "playhead time");
}

#[test]
fn k_seeds_a_time_remap_key_on_its_curve_or_at_the_identity() {
    use ph2d_anim::AnimValue::Float;
    use ph2d_ecs::World;
    let w = World::new();
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    // Empty Time track: K at t = 1.5 seeds the IDENTITY (source = playhead),
    // so the remap changes nothing until the author edits it.
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::Bind {
            entity: 1,
            prop: PropKind::TimeRemap,
        },
    );
    assert_eq!(
        key_value_for(&w, &st, 1, PropKind::TimeRemap, 1.5),
        Some(Float(1.5)),
        "empty Time track seeds the identity"
    );
    // With keys (0 → 0, 2 → 4), a K at t = 1 lands ON the curve (source 2)
    // — inserting it must not jump the retime the entity already plays.
    for (t, v) in [(0.0, 0.0f32), (2.0, 4.0)] {
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TimeRemap,
                t: ph2d_anim::RationalTime::from_seconds(t),
                value: Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    assert_eq!(
        key_value_for(&w, &st, 1, PropKind::TimeRemap, 1.0),
        Some(Float(2.0)),
        "a keyed Time track seeds ON its own curve"
    );
    // A scene prop still samples the world (a dead entity has none).
    assert_eq!(key_value_for(&w, &st, 1, PropKind::TranslationX, 1.0), None);
}

// The natural K flow — K at t=0, scrub, K again — must lay down an
// identity-shaped remap, not a FLAT one that freezes every track of the
// entity at source 0 ("Time nullifies the animation", 2026-07-11). Drives
// the exact functions the shell's K handler uses.
#[test]
fn time_remap_double_k_must_not_freeze_position() {
    use ph2d_anim::AnimValue::Float;
    use ph2d_core::Vec2;
    use ph2d_ecs::{Transform, World};
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let eb = e.to_bits();
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    for (t, v) in [(0.0, 0.0f32), (4.0, 10.0)] {
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: eb,
                prop: PropKind::TranslationX,
                t: ph2d_anim::RationalTime::from_seconds(t),
                value: Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    }
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::Bind {
            entity: eb,
            prop: PropKind::TimeRemap,
        },
    );
    // Two K presses through the SAME functions the shell's K handler uses.
    for playhead_t in [0.0f64, 2.0] {
        let v = key_value_for(&w, &st, eb, PropKind::TimeRemap, playhead_t).unwrap();
        let t = key_insert_time(&st, eb, PropKind::TimeRemap, playhead_t).unwrap();
        apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: eb,
                prop: PropKind::TimeRemap,
                t,
                value: v,
                interp: default_interp(),
            },
        );
    }
    ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, 1.0);
    let x1 = w.get::<Transform>(e).unwrap().translation.x;
    assert!(
        (x1 - 2.5).abs() < 1e-4,
        "posição congelada: x@1 = {x1}, esperado 2.5"
    );
    // And past the seeded range the identity must keep playing too.
    ph2d_timeline::apply_from_doc(&mut w, &mut st.doc, 3.0);
    let x3 = w.get::<Transform>(e).unwrap().translation.x;
    assert!(
        (x3 - 7.5).abs() < 1e-4,
        "posição congelada: x@3 = {x3}, esperado 7.5"
    );
}

// Hold's freeze-frame is DELIBERATE: past a Hold last key the entity plays
// frozen, so a K there seeds the frozen clock (what the entity plays), not
// a slope-1 continuation — seed and sampling stay the same transform.
#[test]
fn k_past_a_hold_freeze_seeds_the_frozen_clock() {
    use ph2d_anim::AnimValue::Float;
    use ph2d_ecs::World;
    let w = World::new();
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph2d_timeline::apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::Bind {
            entity: 1,
            prop: PropKind::TimeRemap,
        },
    );
    for (t, v, interp) in [
        (0.0, 0.0f32, ph2d_anim::Interp::Linear),
        (2.0, 4.0, ph2d_anim::Interp::Hold),
    ] {
        ph2d_timeline::apply_intent(
            &mut st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: 1,
                prop: PropKind::TimeRemap,
                t: ph2d_anim::RationalTime::from_seconds(t),
                value: Float(v),
                interp,
            },
        );
    }
    assert_eq!(
        key_value_for(&w, &st, 1, PropKind::TimeRemap, 3.0),
        Some(Float(4.0)),
        "K past a Hold freeze seeds the frozen source, matching the apply"
    );
    // Scene props key at the same frozen clock (where the apply samples).
    assert_eq!(
        key_insert_time(&st, 1, PropKind::TranslationX, 3.0)
            .unwrap()
            .to_seconds(),
        4.0,
        "scene keys land at the frozen source time"
    );
}

// ── Keys / solo view (ADR-0115 R8 amendment, Enio 2026-07-16) ─────────────────

/// Build a doc where a lane plays `Left` (index 0) then `Right`, and `Right` is the
/// active clip. `Right` ramps X 1 → 5 over its 3 s; `Left` holds X at −3.
fn solo_doc(bits: u64) -> TimelineState {
    use ph2d_anim::AnimValue::Float;
    let mut st = TimelineState::new();
    let mut ph = Playhead::new(1.0 / 60.0);
    let mut key = |st: &mut TimelineState, prop, t: f64, v: f32| {
        ph2d_timeline::apply_intent(
            st,
            &mut ph,
            TimelineIntent::AddKey {
                entity: bits,
                prop,
                t: ph2d_anim::RationalTime::from_seconds(t),
                value: Float(v),
                interp: ph2d_anim::Interp::Linear,
            },
        );
    };
    st.doc.rename_clip(0, "Left".into());
    key(&mut st, PropKind::TranslationX, 0.0, -3.0);
    key(&mut st, PropKind::TranslationX, 3.0, -3.0);
    let right = st.doc.add_clip("Right".into());
    st.doc.set_active(right);
    key(&mut st, PropKind::TranslationX, 0.0, 1.0);
    key(&mut st, PropKind::TranslationX, 3.0, 5.0);
    st.doc.set_active(0);
    let lane = st.doc.add_lane("L".into()).unwrap();
    st.doc.add_strip(lane, 0, 0.0, 3.0);
    st.doc.add_strip(lane, right, 3.0, 6.0);
    st.doc.set_active(right); // editing Right
    st
}

/// **`run(solo = true)` shows the ACTIVE CLIP soloed, at the clip playhead** — not the
/// stack. This is the seam the panel's Keys tab drives: the shell passes the clip
/// playhead + `solo`, and the scene must be the clip you are editing.
#[test]
fn run_in_solo_mode_shows_the_active_clip_not_the_stack() {
    use ph2d_ecs::{SimWorld, Transform};
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), ph2d_ecs::Name::new("Solo")))
        .id()
        .to_bits();
    let mut st = solo_doc(bits);
    let mut ak = crate::render_loop::autokey_pass::AutokeyState::default();
    let mut clip_ph = Playhead::new(1.0 / 60.0);
    clip_ph.pause();
    clip_ph.seek(1.5);
    let mut intents = Vec::new();

    // Solo: Right at clip 1.5 (ramp 1→5) = 3.
    super::run(
        sim.world_mut(),
        &mut st,
        &mut clip_ph,
        &mut intents,
        None,
        &mut ak,
        true,
        None,
    );
    let x = |sim: &SimWorld| {
        f64::from(
            sim.world()
                .get::<Transform>(ph2d_ecs::Entity::from_bits(bits))
                .unwrap()
                .translation
                .x,
        )
    };
    assert_eq!(
        x(&sim),
        3.0,
        "soloed Right at clip 1.5, not the stack's Left"
    );

    // The SAME instant, NOT solo (timeline 1.5): the stack plays Left, holding −3.
    let mut tl_ph = Playhead::new(1.0 / 60.0);
    tl_ph.pause();
    tl_ph.seek(1.5);
    super::run(
        sim.world_mut(),
        &mut st,
        &mut tl_ph,
        &mut intents,
        None,
        &mut ak,
        false,
        None,
    );
    assert_eq!(
        x(&sim),
        -3.0,
        "the stack at 1.5 is Left — solo really differs"
    );
}

/// **K in the Keys/solo view keys the pose at CLIP time, and never refuses.** Under a
/// stack the Arrange-side K can refuse (`key_insert_time` → `None`); the solo path
/// cannot — there is no stack in view to override the clip or play it twice.
#[test]
fn solo_k_keys_at_clip_time_and_never_refuses() {
    use ph2d_anim::AnimValue::Float;
    use ph2d_core::Vec2;
    use ph2d_ecs::{SimWorld, Transform};
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(9.0, 0.0)),
            ph2d_ecs::Name::new("Solo"),
        ))
        .id()
        .to_bits();
    let st = solo_doc(bits);

    // The object is posed at X = 9 by hand. K at clip time 1.5 must capture 9 THERE.
    let (value, t) = key_authoring_solo(sim.world(), &st, bits, PropKind::TranslationX, 1.5)
        .expect("solo K never refuses");
    assert_eq!(
        value,
        Float(9.0),
        "the live pose, stored directly (no blend to invert)"
    );
    assert_eq!(t.to_seconds(), 1.5, "at the clip playhead's time");

    // The Arrange-side K at a timeline instant where Right does NOT play would refuse;
    // solo at the same clip time still answers. (Right plays timeline [3,6); at
    // timeline 1.0 it is not playing.)
    st_doc_active_is_right(&st);
    assert!(
        key_authoring_solo(sim.world(), &st, bits, PropKind::TranslationX, 1.0).is_some(),
        "solo answers at any clip time — there is no 'not playing' in isolation"
    );
}

/// Guard: the fixture really leaves `Right` active (so the assertions above are about
/// the clip the animator is editing).
fn st_doc_active_is_right(st: &TimelineState) {
    assert_eq!(st.doc.clips()[st.doc.active_index()].name, "Right");
}
