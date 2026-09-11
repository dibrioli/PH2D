//! **Arrange is a first-class scope, independent of any clip** (Enio, 2026-07-27).
//!
//! Bug 6: *"O painel arrange vazio toca clips sem limite de tempo. Se ele está vazio não pode
//! tocar nada. ... Só na aba keys, até que algo seja colocado em arrange."*
//!
//! The Arrange world drive is [`apply_scene`], which FORCES the stack path: with nothing
//! arranged (an empty stack), every entity blends toward `rest` and plays NOTHING — instead of
//! [`apply_from_doc_except`], which solos the active clip when the stack is empty (the Keys /
//! single-clip semantics). MEASURED at the root: an empty Arrange left `Obj.x = 7` (the clip's
//! keyed value); now it stays at rest.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineDoc, apply_from_doc_except, apply_scene};

fn scene_with_a_keyed_clip() -> (SimWorld, TimelineDoc, u64) {
    let mut sim = SimWorld::new();
    let bits = sim
        .world_mut()
        .spawn((Transform::default(), Name::new("Obj")))
        .id()
        .to_bits();
    let mut doc = TimelineDoc::new();
    // The active clip keys X flat at 7.0 across [0,4]. NO lanes -> nothing arranged.
    for t in [0.0, 4.0] {
        doc.upsert_key(
            bits,
            PropKind::TranslationX,
            RationalTime::from_seconds(t),
            AnimValue::Float(7.0),
            Interp::Linear,
        );
    }
    (sim, doc, bits)
}

fn x(sim: &mut SimWorld, bits: u64) -> f32 {
    sim.world_mut()
        .get::<Transform>(Entity::from_bits(bits))
        .unwrap()
        .translation
        .x
}

/// **An empty Arrange plays NOTHING.**
///
/// Mutation that should bleed: route the Arrange drive back through
/// `apply_from_doc_except` (the empty-stack solo) — the object snaps to the clip's keyed 7.0.
#[test]
fn an_empty_arrange_plays_nothing() {
    let (mut sim, mut doc, bits) = scene_with_a_keyed_clip();
    apply_scene(sim.world_mut(), &mut doc, 2.0, |_| false);
    let got = x(&mut sim, bits);
    assert!(
        got.abs() < 1e-6,
        "empty Arrange must play nothing, but Obj.x = {got} (the active clip's keyed value \
         leaked in — apply_scene should force the stack path, blending to rest)"
    );
}

/// **The Keys / single-clip solo is UNCHANGED** — an empty stack still plays the active clip
/// through `apply_from_doc_except`. This is the control: the change is scoped to Arrange.
#[test]
fn the_single_clip_solo_still_plays_the_clip() {
    let (mut sim, mut doc, bits) = scene_with_a_keyed_clip();
    apply_from_doc_except(sim.world_mut(), &mut doc, 2.0, |_| false);
    let got = x(&mut sim, bits);
    assert!(
        (got - 7.0).abs() < 1e-6,
        "the single-clip solo must still play the active clip (Obj.x = 7), got {got}"
    );
}
