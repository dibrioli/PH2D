//! **The transport's Physics toggle, at the seam the product actually runs.**
//!
//! `crates/ph2d-physics-ecs/tests/hold.rs` proves `PhysicsBridge::hold` behaves.
//! That is worth nothing on its own: a `hold` nobody calls is unit-green and
//! dead in the product ([[feedback_tool_unit_green_integration_dead]]). These
//! gates drive `physics_bridge::dispatch` — the function the frame loop calls,
//! with the flag in the argument the frame loop passes — so the claim under
//! test is "the checkbox decides", not "a method exists".

use ph2d_core::{Playhead, Vec2};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};
use ph2d_timeline::TimelineDoc;

use super::physics_bridge::dispatch;

const DT: f64 = 1.0 / 60.0;
/// Two seconds of playback. Long enough that a fall is unmistakable and that a
/// clock-bookkeeping bug would owe a visible pile of ticks.
const FRAMES: u64 = 120;

fn falling_scene() -> (SimWorld, Entity) {
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
            Transform::from_translation(Vec2::new(0.0, 4.0)),
        ))
        .id();
    (sim, ball)
}

fn height(sim: &SimWorld, e: Entity) -> f32 {
    sim.world().get::<Transform>(e).unwrap().translation.y
}

/// Play `FRAMES` frames with the toggle in a given state, returning the ball's
/// height at the end and the number of solver steps it cost.
fn play(simulate: bool) -> (f32, u64) {
    let (mut sim, ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    playhead.play();

    for _ in 0..FRAMES {
        playhead.advance();
        dispatch(&mut bridge, &mut sim, &playhead, DT, &mut doc, simulate);
    }
    (height(&sim, ball), bridge.steps_taken())
}

/// The whole feature, stated once: Play advances the curves either way, and
/// whether it also advances the solver is the artist's call.
///
/// Both halves are asserted together on purpose. "Disarmed does not fall" alone
/// is satisfied by a bridge that never works at all, and "armed falls" alone is
/// satisfied by one that ignores the flag — the pair is what pins the switch to
/// a behaviour on each side of it.
#[test]
fn the_transport_toggle_decides_whether_play_steps_the_solver() {
    let (held_y, held_steps) = play(false);
    let (armed_y, armed_steps) = play(true);

    assert_eq!(
        held_steps, 0,
        "the solver ran with the transport's Physics toggle OFF"
    );
    assert_eq!(
        held_y, 4.0,
        "a disarmed frame loop moved the object the artist placed"
    );

    assert_eq!(
        armed_steps, FRAMES,
        "the armed frame loop did not step the owed ticks"
    );
    assert!(
        armed_y < 2.0,
        "the armed frame loop did not simulate: the ball is still at {armed_y}"
    );
}

/// Arming mid-take resumes; it does not replay the disarmed stretch.
///
/// This is the failure that would ship if "off" were implemented as "skip the
/// dispatch": the bridge's clock would still sit at the tick it last stepped, so
/// the frame after the artist ticks the box owes every tick that went by while
/// it was off — the app hangs, and the scene arrives somewhere nobody asked for.
#[test]
fn arming_mid_take_resumes_it_does_not_replay_what_was_skipped() {
    let (mut sim, _ball) = falling_scene();
    let mut bridge = PhysicsBridge::new();
    let mut doc = TimelineDoc::new();
    let mut playhead = Playhead::new(DT);
    playhead.play();

    for _ in 0..FRAMES {
        playhead.advance();
        dispatch(&mut bridge, &mut sim, &playhead, DT, &mut doc, false);
    }
    // The artist ticks the checkbox. One more frame goes by.
    playhead.advance();
    dispatch(&mut bridge, &mut sim, &playhead, DT, &mut doc, true);

    assert_eq!(
        bridge.steps_taken(),
        1,
        "arming cost {} steps — the disarmed stretch was replayed in one frame",
        bridge.steps_taken()
    );
}

/// Physics is opted INTO, per session.
///
/// The default is the answer to "what does Play do?" on the frame after a
/// project loads, and a simulation that starts itself is a scene that has
/// already changed by the time it is looked at. Enio asked for this explicitly;
/// it is a product decision, so it is pinned to a literal.
#[test]
fn the_simulation_is_disarmed_by_default() {
    assert!(
        !ph2d_timeline::TimelineFlags::default().simulate_physics,
        "a fresh document must open with Play driving the animation only"
    );
}
