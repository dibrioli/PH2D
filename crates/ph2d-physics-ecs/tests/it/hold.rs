//! **The simulation is disarmed** — `PhysicsBridge::hold`, the transport's
//! Physics toggle turned off (`TimelineFlags::simulate_physics`, default).
//!
//! One transport, two consumers: the animation curves and the rapier world.
//! Play means "advance time", and left implicit it advances BOTH — so scrubbing
//! to review an animation also drops every dynamic body a little further, and
//! the scene being judged is never the scene that was authored. These gates hold
//! the other half of that split.
//!
//! What is being defended is NOT "nothing happens". A disarmed bridge still
//! reconciles, still settles, and still keeps its clock honest; it just never
//! steps. Each of those is a separate way to get this wrong, so each has its own
//! gate — a single "the ball did not fall" assertion stays green over three of
//! the four bugs (`feedback_layered_defenses_need_per_layer_gates`).

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

/// Far enough into a run that a bug in the clock bookkeeping is unmissable:
/// re-arming after this many held ticks must owe ONE tick, not six hundred.
const HELD_TICKS: u64 = 600;

fn spawn_ball(sim: &mut SimWorld, at: Vec2) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.3 },
                ..Collider::default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn pose(sim: &SimWorld, e: Entity) -> (f32, f32, f32) {
    let t = sim.world().get::<Transform>(e).unwrap();
    (t.translation.x, t.translation.y, t.rotation)
}

/// A ball in mid-air with nothing under it. Gravity is the whole fixture: if
/// the bridge steps even once, this moves, and it moves in a direction no
/// rounding error can produce.
fn falling_scene() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let ball = spawn_ball(&mut sim, Vec2::new(0.0, 4.0));
    (sim, ball)
}

/// The headline claim of the checkbox: with it off, running the clock the whole
/// length of a take moves nothing, however long the take is.
///
/// The oracle is the `Transform` — what the artist SEES — and it is compared
/// bit-for-bit, not within a tolerance. "Physics contributes no motion" admits
/// no epsilon: a body that drifts a micron per tick is a body being simulated.
#[test]
fn a_held_world_never_steps_however_far_the_clock_runs() {
    let (mut sim, ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();

    let before = pose(&sim, ball);
    for tick in 1..=HELD_TICKS {
        bridge.hold(&mut sim, tick);
    }

    assert_eq!(
        bridge.steps_taken(),
        0,
        "the solver ran while the transport's Physics toggle was off"
    );
    assert_eq!(
        pose(&sim, ball),
        before,
        "a disarmed bridge moved the pose the artist authored"
    );
}

/// The trap that makes "just skip the dispatch" wrong.
///
/// A bridge that holds without advancing `last_stepped` is a bridge that thinks
/// it owes every tick that elapsed while it was off. Play ten seconds disarmed,
/// arm it, and the next frame simulates six hundred ticks at once: the app
/// freezes and the scene lands somewhere nobody asked for.
#[test]
fn arming_after_a_held_stretch_owes_one_tick_not_the_whole_span() {
    let (mut sim, _ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();

    for tick in 1..=HELD_TICKS {
        bridge.hold(&mut sim, tick);
    }
    assert_eq!(
        bridge.last_stepped(),
        HELD_TICKS,
        "the held clock fell behind the transport"
    );

    // The artist ticks the checkbox and the clock advances one frame.
    bridge.dispatch(&mut sim, true, HELD_TICKS + 1);

    assert_eq!(
        bridge.steps_taken(),
        1,
        "arming replayed the whole disarmed stretch instead of resuming"
    );
}

/// A body added while disarmed still exists.
///
/// Without the reconcile, physics authored with the toggle off would be
/// invisible until it was armed — no collider outline, and a world that has to
/// be built mid-play at the exact moment the artist wants to see motion.
#[test]
fn a_body_authored_while_held_is_reconciled_not_deferred() {
    let mut sim = SimWorld::new();
    let mut bridge = PhysicsBridge::new();
    bridge.hold(&mut sim, 1);

    let ball = spawn_ball(&mut sim, Vec2::new(0.0, 4.0));
    bridge.hold(&mut sim, 2);

    assert!(
        bridge.body_pose(ball).is_some(),
        "a body authored while the simulation was disarmed never reached the world"
    );
}

/// Arming resumes from what is ON SCREEN.
///
/// While held, the rapier body tracks the authored `Transform` — whether that
/// pose came from the artist's hand or from a baked curve. Skip the settle and
/// the world still holds the pose from whenever the toggle was last on, so
/// arming teleports everything back to a position the artist has since moved
/// away from.
#[test]
fn the_held_world_tracks_the_pose_the_scene_authored() {
    let (mut sim, ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();
    bridge.hold(&mut sim, 1);

    // The scene moves the object — a gizmo drag, or a timeline curve.
    const MOVED_TO: f32 = -2.5;
    sim.world_mut()
        .get_mut::<Transform>(ball)
        .unwrap()
        .translation
        .x = MOVED_TO;
    bridge.hold(&mut sim, 2);

    let (x, _, _) = bridge.body_pose(ball).expect("the body exists");
    assert_eq!(
        x, MOVED_TO,
        "the disarmed world ignored the scene, so arming would snap the object back"
    );
}

/// Disarming ends the run, so the scrub cache goes with it.
///
/// Every checkpoint the ring holds describes a trajectory that is over. Seeding
/// a later scrub from one would answer with a state from before the artist
/// disarmed and rearranged the scene by hand — silently, and only for the ticks
/// that happened to be cached, so the same scrub would disagree with itself
/// depending on where it landed.
#[test]
fn holding_drops_the_checkpoints_of_a_run_that_is_over() {
    let (mut sim, _ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();

    // Play far enough, armed, that the sparse ring has recorded something.
    bridge.dispatch(&mut sim, true, 120);
    let (cached, _) = bridge.ring_stats();
    assert!(
        cached > 0,
        "fixture is inert: the armed run cached nothing, so dropping it proves nothing"
    );

    bridge.hold(&mut sim, 121);

    let (after, _) = bridge.ring_stats();
    assert_eq!(
        after, 0,
        "checkpoints from the finished run survived the toggle going off"
    );
}
