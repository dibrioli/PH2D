//! **A fade against nothing starts from where the object IS.**
//!
//! The pose is a function of the playhead, and a function of the playhead that an
//! animator can use is also CONTINUOUS: an animation system may not teleport. These
//! gates measure the trajectory and bound the per-frame step — the oracle is the
//! appearance ("does it jump?"), never the rule that produces it, so a future
//! evaluator that reaches the same continuity another way stays green
//! ([[reference_topic_oracle_discipline]]).
//!
//! The bug: the gap between two strips was SILENCE (nobody wrote, the object kept its
//! pose), while a strip covering `t` at weight zero — the first instant of a fade-in —
//! answered REST. The two disagreed across one pixel of ruler, so the sprite snapped
//! 3 units to the rest pose before starting the ramp it was meant to start from where
//! it was (Enio, 2026-07-16).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, p: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        p,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

/// `Left` holds x at −3 and `Right` at +3, on one lane. The sprite is born at the
/// ORIGIN, so `rest` is 0 — which is what makes the old bug visible: 0 is neither −3
/// nor +3, and a jump to it is a jump to somewhere the animation never asked for.
struct Scene {
    sim: SimWorld,
    st: TimelineState,
    bits: u64,
    lane: usize,
    right: usize,
}

fn scene() -> Scene {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("StackDemo")))
        .id()
        .to_bits();
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.rename_clip(0, "Left".into());
    key(doc, bits, PropKind::TranslationX, 0.0, -3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, -3.0);
    let right = doc.add_clip("Right".into());
    doc.set_active(right);
    key(doc, bits, PropKind::TranslationX, 0.0, 3.0);
    key(doc, bits, PropKind::TranslationX, 3.0, 3.0);
    doc.set_active(0);
    let lane = doc.add_lane("L".into()).unwrap();
    Scene {
        sim,
        st,
        bits,
        lane,
        right,
    }
}

impl Scene {
    fn x_at(&mut self, t: f64) -> f64 {
        apply_from_doc(self.sim.world_mut(), &mut self.st.doc, t);
        let e = ph2d_ecs::Entity::from_bits(self.bits);
        f64::from(self.sim.world().get::<Transform>(e).unwrap().translation.x)
    }

    /// The biggest single-frame step across `[from, to]`, and where it happened.
    /// 240 samples per second — four times a 60 fps display, so a jump cannot hide
    /// between two of them.
    fn worst_step(&mut self, from: f64, to: f64) -> (f64, f64) {
        let n = ((to - from) * 240.0) as i32;
        let mut prev = self.x_at(from);
        let (mut worst, mut at) = (0.0_f64, from);
        for i in 1..=n {
            let t = from + (to - from) * f64::from(i) / f64::from(n);
            let x = self.x_at(t);
            let step = (x - prev).abs();
            if step > worst {
                worst = step;
                at = t;
            }
            prev = x;
        }
        (worst, at)
    }
}

/// **The bug, as a bound.** `Left[0,3)`, a gap, `Right[4,7)` with a fade-in: the sprite
/// must cross from −3 to +3 without ever teleporting. It used to step 3 units — the
/// whole distance to the rest pose — in a single frame at `t = 4.0`.
#[test]
fn a_fade_in_across_a_gap_starts_from_where_the_object_is() {
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 0.0, 3.0);
    s.st.doc.add_strip(s.lane, s.right, 4.0, 7.0);
    s.st.doc.stack_mut()[s.lane].strips[1].ease_in = 0.5;

    assert_eq!(s.x_at(3.9), -3.0, "the gap holds the pose Left left behind");
    assert_eq!(
        s.x_at(4.0),
        -3.0,
        "and the fade STARTS there — not at the rest pose"
    );

    // Nothing anywhere in the crossing may teleport. The fade moves 6 units in 0.5 s,
    // so a frame at 240 Hz moves at most ~0.05 on a linear ramp; smoothstep peaks at
    // 1.5x that. A tenth of a unit is comfortably above the real motion and far below
    // the 3-unit snap this gate exists to catch.
    let (worst, at) = s.worst_step(2.0, 7.0);
    assert!(
        worst < 0.1,
        "the sprite teleported {worst} units in one frame at t={at}"
    );
}

/// The BOUNDED gap is not silence, and it is not rest either: it is the pose the
/// last strip to end is still asserting — that determinism is what lets a fade
/// cross it. **Past the LAST strip the lane RELEASES** (Enio, 2026-07-23: an
/// upper lane with one short strip was masking every lane below it forever):
/// nothing is written, so the object keeps whatever pose it HAD — played
/// through, that is the end pose, and the trailing region reads identically.
#[test]
fn the_gap_holds_the_last_strips_pose() {
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 0.0, 3.0);
    s.st.doc.add_strip(s.lane, s.right, 4.0, 7.0);
    for t in [3.0, 3.25, 3.5, 3.99] {
        assert_eq!(s.x_at(t), -3.0, "t={t} in the gap");
    }
    s.x_at(6.9); // play through Right's end…
    for t in [7.0, 8.0, 50.0] {
        assert_eq!(
            s.x_at(t),
            3.0,
            "t={t}: released — nothing writes, the pose it HAD stays"
        );
    }
    // And the release is a RELEASE, not a hold: come from inside Left instead,
    // and the trailing region keeps THAT pose (nobody writes there).
    s.x_at(1.0);
    assert_eq!(s.x_at(50.0), -3.0, "released: keeps the pose it came with");
}

/// **The pose is a function of the playhead.** Scrub in from the left and in from the
/// right and the same instant must give the same answer — the invariant that broke
/// when zero weight was read as silence, and that the hold must not break again.
#[test]
fn the_pose_does_not_depend_on_which_side_you_scrub_from() {
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 0.0, 3.0);
    s.st.doc.add_strip(s.lane, s.right, 4.0, 7.0);
    s.st.doc.stack_mut()[s.lane].strips[1].ease_in = 0.5;

    for probe in [3.0, 3.5, 4.0, 4.25, 4.5, 5.0] {
        s.x_at(0.5); // arrive from the left
        let from_left = s.x_at(probe);
        s.x_at(6.5); // arrive from the right
        let from_right = s.x_at(probe);
        assert_eq!(
            from_left, from_right,
            "t={probe}: {from_left} from the left, {from_right} from the right"
        );
    }
}

/// **The overlap is untouched** — the crossfade Enio already approved
/// (*"com a sobreposição está funcionando perfeitamente"*). Two strips through their
/// overlap sum to exactly 1, so nothing is held and the mix is the same one it was.
#[test]
fn an_overlap_still_crossfades_with_nothing_held() {
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 0.0, 3.0);
    s.st.doc.add_strip(s.lane, s.right, 2.0, 5.0); // 1 s overlap — the smoke scene

    assert!(s.x_at(0.5) < -2.5, "starts left");
    assert!(s.x_at(4.5) > 2.5, "ends right");
    let mid = s.x_at(2.5);
    assert!(
        (-2.5..=2.5).contains(&mid),
        "the middle of the overlap is a real mix: {mid}"
    );
    let (worst, at) = s.worst_step(0.0, 5.0);
    assert!(worst < 0.1, "teleported {worst} units at t={at}");
}

/// A LONE strip's fade-in, with nothing before it, still ramps from the rest pose —
/// there is nothing behind it to hold, and fading in from rest at the top of a
/// timeline is a real thing to want. The hold is forward-only on purpose.
#[test]
fn the_first_strip_has_nothing_to_hold_and_fades_in_from_rest() {
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 1.0, 4.0);
    s.st.doc.stack_mut()[s.lane].strips[0].ease_in = 0.5;

    assert_eq!(
        s.x_at(1.0),
        0.0,
        "the rest pose: nothing precedes this strip"
    );
    assert_eq!(s.x_at(1.5), -3.0, "and the fade lands on the clip");
    let (worst, at) = s.worst_step(1.0, 4.0);
    assert!(worst < 0.1, "teleported {worst} units at t={at}");
}

/// `K` must not key into a strip that is OVER. The held strip shapes the pose, but it
/// is not *playing* — and "where is this clip right now" has to say so, or a key lands
/// past the end of a strip nobody is watching.
#[test]
fn a_held_strip_is_not_playing() {
    use ph2d_timeline::{KeyRefusal, clip_playhead};
    let mut s = scene();
    s.st.doc.add_strip(s.lane, 0, 0.0, 3.0);
    s.st.doc.add_strip(s.lane, s.right, 4.0, 7.0);

    // t = 3.5: Left is HOLDING (it shapes the pose — the gate above proves x = −3)…
    s.st.doc.prime_stack(3.5);
    assert_eq!(s.x_at(3.5), -3.0);
    s.st.doc.prime_stack(3.5);
    assert_eq!(
        clip_playhead(&s.st.doc, 3.5),
        Err(KeyRefusal::NotPlaying),
        "…and it is still not PLAYING there"
    );
    // Where it really is playing, it answers.
    s.st.doc.prime_stack(1.0);
    assert_eq!(clip_playhead(&s.st.doc, 1.0), Ok(1.0));
}
