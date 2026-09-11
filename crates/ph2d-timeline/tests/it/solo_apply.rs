//! **Editing a clip's keys shows you THAT clip — soloed, not the stack.**
//!
//! The Keys tab drives [`ph2d_timeline::apply_active_clip`]: the active clip alone,
//! at its own clip clock, with the stack entirely out of the way. This is the AE
//! precomp model, and it is what makes authoring honest — a lane above cannot hide
//! the pose you are keying (ADR-0115 R9's "Overridden" case cannot arise when there
//! is no stack in view). The oracle is the pose, not the mechanism.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_active_clip, apply_from_doc};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// One object, `Left` (index 0) holds x at −3, `Right` at +3. A lane plays `Left`
/// then `Right`. `Right` is the ACTIVE clip.
fn scene() -> (SimWorld, TimelineState, u64) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Solo")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Left".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, bits, PropKind::TranslationX, 0.0, 1.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 5.0); // Right ramps x: 1 → 5
    // A lane so the STACK path is live — the whole point is that solo ignores it.
    let lane = doc.add_lane("L".into()).unwrap();
    doc.add_strip(lane, 0, 0.0, 3.0); // Left at [0,3)
    doc.add_strip(lane, right, 3.0, 6.0); // Right at [3,6)
    (sim, st, bits)
}

fn x(sim: &SimWorld, bits: u64) -> f64 {
    f64::from(
        sim.world()
            .get::<Transform>(Entity::from_bits(bits))
            .unwrap()
            .translation
            .x,
    )
}

/// **Solo shows the ACTIVE clip at its own clip time**, whatever the stack is doing.
/// At clip time 1.5, `Right` ramps 1 → 5, so x = 3 — not −3 (where the stack's `Left`
/// strip would put it at timeline 1.5).
#[test]
fn solo_samples_the_active_clip_at_clip_time() {
    let (mut sim, mut st, bits) = scene();
    apply_active_clip(sim.world_mut(), &mut st.doc, 1.5, |_| false);
    assert_eq!(
        x(&sim, bits),
        3.0,
        "Right at clip 1.5 (ramp 1→5), not the stack"
    );

    apply_active_clip(sim.world_mut(), &mut st.doc, 0.0, |_| false);
    assert_eq!(x(&sim, bits), 1.0, "Right's first key");
    apply_active_clip(sim.world_mut(), &mut st.doc, 3.0, |_| false);
    assert_eq!(x(&sim, bits), 5.0, "Right's last key");
}

/// **The stack is IGNORED.** The same instant, applied through the stack, gives a
/// different pose — proving solo is not accidentally reading the same thing.
#[test]
fn solo_is_not_the_stack() {
    let (mut sim, mut st, bits) = scene();
    // Through the stack at timeline 1.5: Left is playing, holding x = −3.
    apply_from_doc(sim.world_mut(), &mut st.doc, 1.5);
    assert_eq!(x(&sim, bits), -3.0, "the stack at 1.5 is Left");
    // Soloed at clip 1.5: Right, x = 3. Different pose, same instant.
    apply_active_clip(sim.world_mut(), &mut st.doc, 1.5, |_| false);
    assert_eq!(x(&sim, bits), 3.0, "solo at 1.5 is Right");
}

/// **Switching the active clip switches what solo shows.** Solo is "the clip you are
/// editing", and the dropdown chooses it.
#[test]
fn solo_follows_the_active_clip() {
    let (mut sim, mut st, bits) = scene();
    st.doc.set_active(0); // now editing Left
    apply_active_clip(sim.world_mut(), &mut st.doc, 1.5, |_| false);
    assert_eq!(x(&sim, bits), -3.0, "Left holds −3");
}

/// A clip with no track for this channel writes nothing — the object keeps its pose
/// (sparsity, same rule the stacked apply honours). Never a snap to a default.
#[test]
fn solo_leaves_an_unanimated_channel_alone() {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(7.0, 0.0)),
            Name::new("Solo"),
        ))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    // Bind Y only; X is never keyed.
    key(&mut st.doc, bits, PropKind::TranslationY, 0.0, 0.0);
    apply_active_clip(sim.world_mut(), &mut st.doc, 0.5, |_| false);
    assert_eq!(x(&sim, bits), 7.0, "X was never animated — left untouched");
}

/// `skip` is honoured — the gizmo-dragged entity is not clobbered, same contract as
/// the stacked apply.
#[test]
fn solo_honours_skip() {
    let (mut sim, mut st, bits) = scene();
    // Put the object somewhere by hand, then solo-apply but SKIP it.
    if let Some(mut xf) = sim
        .world_mut()
        .get_mut::<Transform>(Entity::from_bits(bits))
    {
        xf.translation.x = 42.0;
    }
    apply_active_clip(sim.world_mut(), &mut st.doc, 1.5, |b| b == bits);
    assert_eq!(
        x(&sim, bits),
        42.0,
        "skipped: the document did not write it"
    );
}

/// Solo honours the entity's Time Remap (the clip's own clock), so a remapped object
/// stays remapped when soloed — the same door the scene apply and K read.
#[test]
fn solo_honours_time_remap() {
    let (mut sim, mut st, bits) = scene();
    st.doc.set_active(0); // Left: holds −3 everywhere, so use Right to see a ramp
    st.doc.set_active(1);
    // A Time Remap that freezes the clip at time 0 (a flat Hold at 0).
    st.doc.upsert_key(
        bits,
        PropKind::TimeRemap,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Hold,
    );
    apply_active_clip(sim.world_mut(), &mut st.doc, 2.0, |_| false);
    assert_eq!(
        x(&sim, bits),
        1.0,
        "Time Remap holds clip time at 0 → Right's first key, not the 2.0 ramp"
    );
}
