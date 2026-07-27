//! **A static body is where the artist put it — including during PLAY.**
//!
//! `settle` (the paused branch) makes every rapier body track its authored
//! `Transform`, and `drive_kinematic`'s own comment declared the coverage
//! complete: *"a wall that has been moved by hand is caught by `settle`, while
//! paused"*. During play nothing caught it — the solver does not own a static
//! body's pose (`readback` skips it), the scene does not push it per tick
//! (`drive_kinematic` skips it), and `settle` only runs at `Ordering::Equal`
//! while paused. So dragging a wall with the clock running moved the DRAWING
//! and left the collider behind: a phantom collider, which is exactly what the
//! artist reported.
//!
//! The law: **a static body's pose has exactly one author, the authored
//! `Transform`** — the solver never writes it, so there is no second writer to
//! disagree with, and the pose can therefore be honoured on every dispatch
//! rather than only on the paused ones.
//!
//! The oracle is BEHAVIOURAL, because "phantom collider" is a claim about where
//! the collider *acts*, not about a number we could read from the same place
//! that wrote it.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// A ball resting on a static slab. The slab is what gets dragged.
fn resting_scene() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let slab = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 4.0,
                    half_y: 0.5,
                },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.0)),
        ))
        .id();
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 1.5)),
        ))
        .id();
    (sim, slab, ball)
}

fn y_of(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

fn set_y(sim: &mut SimWorld, e: Entity, y: f32) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation.y = y;
    }
}

fn run(bridge: &mut PhysicsBridge, sim: &mut SimWorld, from: u64, ticks: u64) -> u64 {
    let mut tick = from;
    for _ in 0..ticks {
        tick += 1;
        bridge.dispatch(sim, true, tick);
    }
    tick
}

/// **The gate.** Drag the slab DOWN a metre with the clock running; the ball has
/// to come down and rest on the new top, because the collider is where the
/// drawing is.
///
/// ⚠️ The direction is not arbitrary and the first version of this fixture got
/// it wrong: dragging the slab UP *through* the ball jumps its whole span past
/// the ball (span `[-0.5, 0.5]` → `[1.5, 2.5]`, ball bottom at 0.499), so there
/// is no overlap to resolve, the ball is left with nothing under it and falls —
/// a correct outcome that the oracle called a failure. Downwards there is only
/// one physical answer: the ball follows the floor it is standing on.
///
/// Written RED-first: before the fix the ball hung at its old resting height
/// while the slab was drawn a metre below it.
#[test]
fn a_static_body_dragged_during_play_carries_its_collider() {
    let (mut sim, slab, ball) = resting_scene();
    let mut bridge = PhysicsBridge::default();

    // Let it settle so the ball is genuinely ASLEEP on the slab — the state an
    // artist actually reaches before reaching for the wall, and the state a
    // wake-up bug hides in.
    let tick = run(&mut bridge, &mut sim, 0, 180);
    let rest = y_of(&sim, ball);
    assert!(
        (rest - 0.8).abs() < 0.01,
        "fixture: the ball should rest on top of the slab, got y = {rest:.4}"
    );

    // The gesture: the artist drags the slab down while the clock runs.
    set_y(&mut sim, slab, -1.0);
    run(&mut bridge, &mut sim, tick, 120);

    let carried = y_of(&sim, ball);
    assert!(
        (carried - (rest - 1.0)).abs() < 0.05,
        "the slab moved to y = -1.0 (top at -0.5) and the ball is at \
         y = {carried:.4} instead of {:.4}: the collider is not where the \
         drawing is",
        rest - 1.0
    );
    // And the collider itself reports the authored pose, not the spawn one.
    let (_, slab_y, _) = bridge.body_pose(slab).expect("slab has a body");
    assert!(
        (slab_y - -1.0).abs() < 1e-6,
        "the rapier slab is at y = {slab_y:.6}, authored y = -1.0"
    );
}

/// The CONTROL, and it is the half that keeps the fix from becoming a bug: a
/// static body nobody touched must not be teleported (and re-woken) every
/// dispatch. `settle` earned this guard while paused — teleporting
/// unconditionally zeroes velocity, and doing it per frame during play would
/// re-wake every sleeping stack forever.
#[test]
fn an_untouched_static_body_is_not_disturbed_during_play() {
    let (mut sim, slab, ball) = resting_scene();
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 180);
    let rest = y_of(&sim, ball);
    let before = bridge.body_pose(slab).expect("slab has a body");

    run(&mut bridge, &mut sim, tick, 120);

    assert_eq!(
        bridge.body_pose(slab).expect("slab has a body"),
        before,
        "an untouched static body moved"
    );
    let after = y_of(&sim, ball);
    assert!(
        (after - rest).abs() < 1e-4,
        "the ball drifted from {rest:.6} to {after:.6} with nothing touched"
    );
}

/// A DYNAMIC body is not covered by this law, and the distinction is the whole
/// reason it is safe: the solver owns a dynamic pose, so honouring an authored
/// `Transform` during play would be a second author — and the one that ran last
/// would win in silence, which is the frame-order bug W4 documented.
///
/// Here the ball is falling and its `Transform` is overwritten every dispatch by
/// `readback`. Writing to it mid-play must not teleport the body.
#[test]
fn a_dynamic_body_is_not_settled_during_play() {
    let mut sim = SimWorld::new();
    let ball = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(0.0, 10.0)),
        ))
        .id();
    let mut bridge = PhysicsBridge::default();
    let tick = run(&mut bridge, &mut sim, 0, 30);
    let falling = y_of(&sim, ball);

    // Someone writes the Transform of a body the solver owns. The next dispatch
    // must ignore it: the fall continues from where the SOLVER left it.
    set_y(&mut sim, ball, 100.0);
    run(&mut bridge, &mut sim, tick, 1);
    let next = y_of(&sim, ball);
    assert!(
        next < falling,
        "a dynamic body was teleported by an authored Transform mid-play \
         ({falling:.4} -> {next:.4})"
    );
}
