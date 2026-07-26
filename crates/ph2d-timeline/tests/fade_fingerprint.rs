//! THE GUARDIAN of the precious Clips/Strips/Containers/Fade system (plan
//! `docs/Timeline/07_plano_joias_da_coroa.md` §8).
//!
//! A scripted scene that exercises the whole fade surface at once — an **overlap
//! crossfade**, a **`lead_out` gap fade**, and a **nested container instance** —
//! sampled densely through the REAL apply into a REAL world, then folded into ONE
//! pinned hash. It is run (as a normal test) on the tree BEFORE a wave and again
//! AFTER: any change that moves this hash TOUCHED the fade, and fails here. This
//! is the Enio constraint *"o sistema de fade não pode ser afetado"* made
//! executable — the six timeline "crown jewels" (plan 07) each keep it green.
//!
//! Not a hand-rolled math harness: it spawns entities, builds a document, calls
//! `apply_from_doc`, and reads the `Transform` back — the same discipline as
//! `clip_stack_eval.rs` (a harness reproduces the mechanism, not the context).
//!
//! ⚠️ CPU-only pure f32/f64 arithmetic (smoothstep `ramp`, `fold`, lerp — no
//! transcendental, and Rust never auto-contracts FMA), so the hash is an EXACT
//! literal, the project template for a CPU-only gate. If a legitimate future
//! change to the fade is intended, the hash is re-pinned in the SAME commit with
//! the reason — never loosened to an epsilon.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Transform, World};
use ph2d_timeline::{PropKind, StackHost, StripSource, TimelineDoc, apply_from_doc};

fn s(t: f64) -> RationalTime {
    RationalTime::from_seconds(t)
}

/// Key `TranslationX` of `e` in clip `clip` as a ramp `a` (t=0) → `b` (t=2).
fn ramp(doc: &mut TimelineDoc, clip: usize, e: u64, a: f32, b: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(0.0),
        AnimValue::Float(a),
        Interp::Linear,
    );
    doc.insert_key(
        e,
        PropKind::TranslationX,
        s(2.0),
        AnimValue::Float(b),
        Interp::Linear,
    );
    doc.set_active(was);
}

/// The scene: one sprite, and a document whose scene stack holds
/// - lane A: clip 0 `[0,3)` + clip 1 `[2,5)` → a **1 s overlap crossfade** `[2,3)`,
///   and clip 1 carries a **`lead_out` of 1 s** → it plays fully then fades in the
///   gap `[5,6)` (the outward fade, `hold_at`).
/// - lane B: an **instance of container C** `[4,7)`, whose interior plays clip 2 —
///   the nesting clock.
///
/// Every value is distinct (0-10 / 100-110 / 50-60) so the blend is visible, and
/// the three regions overlap in time so cross-lane mix is exercised too.
fn build_scene() -> (World, TimelineDoc, u64) {
    let mut world = World::new();
    let e = world
        .spawn(Transform {
            translation: Vec2::new(0.0, 0.0),
            ..Default::default()
        })
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();

    ramp(&mut doc, 0, e, 0.0, 10.0);
    ramp(&mut doc, 1, e, 100.0, 110.0);
    ramp(&mut doc, 2, e, 50.0, 60.0);

    // Container C: interior lane plays clip 2.
    let c = doc.add_container("C".into());
    let ci = doc
        .add_lane_in(StackHost::Container(c), "ci".into())
        .expect("container lane");
    doc.add_strip_to(StackHost::Container(c), ci, StripSource::Clip(2), 0.0, 2.0)
        .expect("container interior strip");
    let c = u16::try_from(c).expect("container index fits u16");

    // Scene lane A: two overlapping clip strips + a lead_out on the second.
    let la = doc.add_lane("A".into()).expect("scene lane A");
    doc.add_strip_to(StackHost::Document, la, StripSource::Clip(0), 0.0, 3.0)
        .expect("strip A0");
    let b = doc
        .add_strip_to(StackHost::Document, la, StripSource::Clip(1), 2.0, 5.0)
        .expect("strip A1");
    doc.strip_mut(la, b).expect("strip A1 mut").lead_out = 1.0;

    // Scene lane B: an instance of the container.
    let lb = doc.add_lane("B".into()).expect("scene lane B");
    doc.add_strip_to(StackHost::Document, lb, StripSource::Container(c), 4.0, 7.0)
        .expect("container instance");

    (world, doc, e)
}

/// Sample `TranslationX` across `[0, 8]s` at 0.05 s and fold the exact f32 bits
/// into an FNV-1a hash. Returns `(hash, samples)` — the samples ride along so a
/// mismatch panic can show WHERE the fade moved.
fn fingerprint() -> (u64, Vec<(f64, f32)>) {
    let (mut world, mut doc, e) = build_scene();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325; // FNV-1a offset basis
    let mut samples = Vec::with_capacity(161);
    for i in 0..=160 {
        let t = f64::from(i) * 0.05;
        apply_from_doc(&mut world, &mut doc, t);
        let x = world
            .get::<Transform>(Entity::from_bits(e))
            .expect("entity has a transform")
            .translation
            .x;
        samples.push((t, x));
        for byte in x.to_bits().to_le_bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0000_0100_0000_01b3); // FNV-1a prime
        }
    }
    (h, samples)
}

/// The pinned fingerprint of the fade, captured 2026-07-26 on the untouched tree
/// (`line/anim`, before any of the plan-07 waves). If a wave moves this, it
/// touched the crossfade / lead_out / nesting clock — which the plan forbids.
const FADE_FINGERPRINT: u64 = 0x69dc_a881_1eb0_f8f8;

#[test]
fn the_fade_surface_is_byte_stable() {
    let (h, samples) = fingerprint();

    // Guard against a "green by accident" scene: if a future refactor made the
    // sampled curve flat/inert, the hash would still be stable but prove nothing.
    // The real fade crosses several distinct plateaus, so the sample set must vary.
    let (lo, hi) = samples
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), &(_, x)| {
            (lo.min(x), hi.max(x))
        });
    assert!(
        hi - lo > 25.0,
        "the fingerprint scene went inert (range {lo}..{hi}); it must exercise the fade"
    );

    assert_eq!(
        h, FADE_FINGERPRINT,
        "\nTHE FADE MOVED. This gate guards the Clips/Strips/Containers/Fade system \
         (plan 07 §8): a wave changed the crossfade / lead_out / nesting result.\n\
         If the change to the fade was intended, re-pin FADE_FINGERPRINT in the SAME \
         commit with the reason. Otherwise, revert the fade-touching change.\n\
         samples (t, x) = {samples:?}"
    );
}
