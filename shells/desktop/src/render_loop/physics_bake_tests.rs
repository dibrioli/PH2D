//! **W4 gates — the baked curve is the simulation, and it costs ONE undo step.**
//!
//! The oracle is deliberately the product path: the keys are written by
//! `bake_selection`, then read back by `ph2d_timeline::apply_from_doc` into a
//! `Transform` — the same function the frame loop calls. A gate that evaluated
//! the track directly would prove the fit works and say nothing about whether
//! an artist pressing play sees the drop they baked.

use ph2d_core::Vec2;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, register_ecs_components};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody, register_physics_components,
};
use ph2d_timeline::{PropKind, TimelineState};

use super::{BakeOutcome, DEFAULT_BAKE_SECONDS, bake_selection, ticks_for};

/// How many Ctrl+Z presses it takes to empty the timeline's undo stack — the
/// count as the artist experiences it. Destructive, so it is the LAST thing a
/// test does with that state. Counted rather than read off a field because
/// "how many presses" is the question, and a depth accessor would be a second
/// way to ask it.
fn undo_presses(timeline: &mut TimelineState) -> usize {
    let mut n = 0;
    while timeline.undo() {
        n += 1;
    }
    n
}

const DT: f64 = 1.0 / 60.0;
const BAKE_SECONDS: f64 = 1.5;

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    register_physics_components(&mut reg);
    reg
}

/// A ball dropped off-centre onto a sloped-ish floor so it bounces AND rolls:
/// all three channels move, which is what makes "the curve is the sim" a claim
/// about more than a parabola.
fn scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 8.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform {
            rotation: 0.12,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    let ball = sim
        .world_mut()
        .spawn((
            Name::new("Ball"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                restitution: 0.35,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(-1.0, 2.0)),
        ))
        .id();
    (sim, ball)
}

/// The trajectory the artist would see by pressing play, as `(t, x, y, rot)`.
fn simulated(ticks: u64) -> Vec<(f64, f32, f32, f32)> {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut out = Vec::new();
    bridge.dispatch(&mut sim, false, 0);
    for tick in 0..=ticks {
        if tick > 0 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let t = sim.world().get::<Transform>(ball).unwrap();
        out.push((
            tick as f64 * DT,
            t.translation.x,
            t.translation.y,
            t.rotation,
        ));
    }
    out
}

/// Bake the scene and hand back everything a gate might want to look at.
fn baked() -> (TimelineState, SimWorld, Entity, BakeOutcome) {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();
    let outcome = bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[ball],
        BAKE_SECONDS,
        DT,
        &queue,
        &reg,
    );
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("apply");
    (timeline, sim, ball, outcome)
}

/// **The baked curve reproduces the simulated trajectory.**
///
/// An APPEARANCE oracle: read the curve back through `apply_from_doc` — the
/// frame loop's own function — into a `Transform`, and compare against what the
/// sim did at that instant. Sampled at every tick, not at the ends, because a
/// curve pinned at both ends and wrong in the middle is exactly the failure a
/// fit produces.
///
/// The bar is the fit's declared fidelity (~1-3% of the motion's range, the
/// trade signed off in the timeline's §17.2), expressed against the RANGE of
/// the motion rather than an absolute distance — a 2 m drop and a 2 cm nudge
/// are not held to the same millimetre.
#[test]
fn the_baked_curve_reproduces_the_simulated_motion() {
    let ticks = ticks_for(BAKE_SECONDS, DT);
    let truth = simulated(ticks);
    let (mut timeline, mut sim, ball, _) = baked();

    let span = |pick: fn(&(f64, f32, f32, f32)) -> f32| {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for s in &truth {
            lo = lo.min(pick(s));
            hi = hi.max(pick(s));
        }
        (hi - lo).max(1e-3)
    };
    let (span_x, span_y, span_r) = (span(|s| s.1), span(|s| s.2), span(|s| s.3));
    const TOL: f32 = 0.05;

    let mut worst = 0.0f32;
    for s in &truth {
        ph2d_timeline::apply_from_doc(sim.world_mut(), &mut timeline.doc, s.0);
        let t = *sim.world().get::<Transform>(ball).unwrap();
        let dx = (t.translation.x - s.1).abs() / span_x;
        let dy = (t.translation.y - s.2).abs() / span_y;
        let dr = (t.rotation - s.3).abs() / span_r;
        worst = worst.max(dx).max(dy).max(dr);
        assert!(
            dx < TOL && dy < TOL && dr < TOL,
            "at t={:.3}s the baked curve is at ({:.4}, {:.4}, {:.4}) but the \
             simulation was at ({:.4}, {:.4}, {:.4}) — {:.1}% / {:.1}% / {:.1}% \
             of each channel's range",
            s.0,
            t.translation.x,
            t.translation.y,
            t.rotation,
            s.1,
            s.2,
            s.3,
            dx * 100.0,
            dy * 100.0,
            dr * 100.0
        );
    }
    assert!(
        worst > 0.0,
        "the curve matched the simulation EXACTLY at every sample — the fit \
         cannot have run, so this gate is measuring the dense keys and would \
         not notice a broken fit"
    );
}

/// **A bake is ONE undo step, not one per frame.** (The plan's gate 3.)
///
/// 90 ticks × 3 channels = 270 dense keys, plus the fit that replaces them.
/// The artist pressed one button, so Ctrl+Z is one press. Counted through the
/// real `TimelineHistory`, which is what Ctrl+Z actually pops.
#[test]
fn a_whole_bake_is_a_single_undo_step() {
    let (mut sim, ball) = scene();
    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();

    let outcome = bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[ball],
        BAKE_SECONDS,
        DT,
        &queue,
        &reg,
    );
    assert!(!outcome.is_empty(), "the fixture baked nothing");

    let presses = undo_presses(&mut timeline);
    assert_eq!(
        presses,
        1,
        "a bake of {} ticks across {} tracks takes {presses} Ctrl+Z presses to \
         undo — the artist pressed one button",
        ticks_for(BAKE_SECONDS, DT),
        outcome.tracks,
    );
}

/// **One Ctrl+Z takes the whole bake away.**
///
/// The other half of the step count: a single step that only reverts the
/// *cleanup* and leaves 270 dense keys behind would satisfy the count above and
/// be worse than no undo at all.
#[test]
fn undoing_a_bake_leaves_no_keys_behind() {
    let (mut timeline, _sim, ball, _) = baked();
    assert!(
        timeline
            .doc
            .binding_for(ball.to_bits(), PropKind::TranslationY)
            .is_some(),
        "the fixture did not bake a Y track"
    );

    timeline.undo();

    for prop in PropKind::ALL {
        let has_keys = timeline
            .doc
            .binding_for(ball.to_bits(), prop)
            .and_then(|b| timeline.doc.active_clip().track(b.target))
            .is_some_and(|t| !t.is_empty());
        assert!(
            !has_keys,
            "{prop:?} still holds keys after one undo of the bake that made them"
        );
    }
}

/// **The same scene bakes to the same curve.** (ADR-0131 D7.)
///
/// The sampler's determinism is gated in `ph2d-physics-ecs`; this is the claim
/// for everything downstream of it — the fit, the column merge, the key times.
/// Compared as the curve's own numbers, because that is what gets saved.
#[test]
fn baking_twice_writes_the_same_curve() {
    let keys = || {
        let (timeline, _sim, ball, _) = baked();
        let mut out: Vec<(PropKind, Vec<(f64, f32)>)> = Vec::new();
        for prop in PropKind::ALL {
            if let Some(track) = timeline
                .doc
                .binding_for(ball.to_bits(), prop)
                .and_then(|b| timeline.doc.active_clip().track(b.target))
            {
                out.push((
                    prop,
                    track
                        .keys()
                        .iter()
                        .map(|k| {
                            (
                                k.t.to_seconds(),
                                match k.value {
                                    ph2d_anim::AnimValue::Float(f) => f,
                                    _ => f32::NAN,
                                },
                            )
                        })
                        .collect(),
                ));
            }
        }
        out
    };
    assert_eq!(
        keys(),
        keys(),
        "two bakes of one scene wrote different curves"
    );
}

/// **The bake hands the pose over: the body ends Kinematic.**
///
/// Without this the curve is written and then overwritten by the solver every
/// frame, and the button reads as broken (module docs). Asserted on the ECS
/// after the command queue is applied — the same path the §11 chip takes, so a
/// gate passing here means the chip works too.
#[test]
fn a_baked_body_is_handed_over_to_the_scene() {
    let (_timeline, sim, ball, outcome) = baked();
    assert!(!outcome.is_empty());
    assert_eq!(
        sim.world().get::<RigidBody>(ball).map(|rb| rb.kind),
        Some(BodyKind::Kinematic),
        "the baked body is still simulated, so the solver will overwrite the \
         curve that was just written and nothing on screen will change"
    );
}

/// **A body that never moves produces no bake, and no undo step.**
///
/// The empty case has to be empty all the way down: no tracks, and no bracket
/// left on the stack for the artist to press Ctrl+Z through.
#[test]
fn baking_a_body_that_never_moves_writes_nothing() {
    let mut sim = SimWorld::new();
    let wall = sim
        .world_mut()
        .spawn((
            Name::new("Wall"),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    let mut timeline = TimelineState::default();
    let queue = EditorCommandQueue::default();
    let reg = registry();
    let outcome = bake_selection(
        &mut timeline,
        &mut bridge,
        &mut sim,
        &[wall],
        BAKE_SECONDS,
        DT,
        &queue,
        &reg,
    );

    assert!(
        outcome.is_empty(),
        "a static wall baked {} tracks",
        outcome.tracks
    );
    assert_eq!(
        undo_presses(&mut timeline),
        0,
        "an empty bake left an undo step the artist has to press through"
    );
}

/// **The default range is enough for the thing it exists for.**
///
/// `DEFAULT_BAKE_SECONDS` claims a drop has finished inside it. That claim is
/// checkable, so it is checked — a default that silently cut the motion off
/// halfway would produce a curve that stops in mid-air.
///
/// ⚠️ Its own FLAT-floor fixture, not the bouncing scene above. That one is
/// built on a tilted floor to make all three channels move, which means the
/// ball rolls downhill and off the end — it is still travelling at 5 s, at 10 s
/// and at any number you pick, because nothing stops it. Measuring "has the
/// motion finished" there measures the ramp, and the first version of this gate
/// failed for exactly that reason while `DEFAULT_BAKE_SECONDS` was fine.
#[test]
fn the_default_range_outlasts_a_drop() {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 8.0,
                half_y: 0.2,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    // From the top of the default view, which is what the constant claims for.
    let ball = sim
        .world_mut()
        .spawn((
            Name::new("Ball"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                restitution: 0.35,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 4.0)),
        ))
        .id();

    let ticks = ticks_for(DEFAULT_BAKE_SECONDS, DT);
    let mut bridge = PhysicsBridge::new();
    let mut ys = Vec::new();
    for tick in 1..=ticks {
        bridge.dispatch(&mut sim, true, tick);
        ys.push(sim.world().get::<Transform>(ball).unwrap().translation.y);
    }

    let settled = (ys[ys.len() - 1] - ys[ys.len() - 20]).abs();
    assert!(
        settled < 0.01,
        "at the end of the default {DEFAULT_BAKE_SECONDS}s range the body is \
         still moving ({settled:.4} m over the last 20 ticks) — the default cuts \
         the motion off"
    );
    assert!(
        ys[0] > 3.5 && ys[ys.len() - 1] < 0.5,
        "the fixture is not a drop: it starts at {:.2} and ends at {:.2}",
        ys[0],
        ys[ys.len() - 1]
    );
}
