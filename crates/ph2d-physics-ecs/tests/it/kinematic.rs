//! **W4 — `BodyKind::Kinematic`: the pose is an INPUT.**
//!
//! `Dynamic` and `Static` between them cover "the solver decides" and "nothing
//! decides". The kind this wave adds covers the third case, which is the one a
//! baked body is in: **the scene decides, and the solver is told**.
//!
//! The gate that matters is not that a kinematic body holds still — `Static`
//! does that too, and a broken drive stage would pass it. It is that a
//! kinematic body **pushes**: it carries what rests on it and shoves what
//! stands in its way, because rapier derives its velocity from the pose it was
//! aimed at. Teleporting the body instead (`set_body_pose`, which zeroes the
//! velocity) looks identical on screen right up to the first contact, and then
//! leaves the load behind. That difference is what these tests measure.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, PhysicsBridge, RigidBody};

const PLATFORM_HALF_X: f32 = 1.0;
const PLATFORM_HALF_Y: f32 = 0.2;
const BOX_HALF: f32 = 0.25;

fn spawn_body(sim: &mut SimWorld, kind: BodyKind, shape: ColliderShape, at: Vec2) -> Entity {
    sim.world_mut()
        .spawn((
            RigidBody { kind },
            Collider {
                shape,
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

fn set_x(sim: &mut SimWorld, e: Entity, x: f32) {
    sim.world_mut()
        .get_mut::<Transform>(e)
        .unwrap()
        .translation
        .x = x;
}

/// A box resting on a platform. The platform's kind is the variable.
fn platform_scene(kind: BodyKind) -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let platform = spawn_body(
        &mut sim,
        kind,
        ColliderShape::Cuboid {
            half_x: PLATFORM_HALF_X,
            half_y: PLATFORM_HALF_Y,
        },
        Vec2::new(0.0, 0.0),
    );
    // Resting ON it: bottom of the box exactly at the top of the platform.
    let cargo = spawn_body(
        &mut sim,
        BodyKind::Dynamic,
        ColliderShape::Cuboid {
            half_x: BOX_HALF,
            half_y: BOX_HALF,
        },
        Vec2::new(0.0, PLATFORM_HALF_Y + BOX_HALF),
    );
    (sim, platform, cargo)
}

/// **A kinematic body does not fall, and the solver never writes its pose.**
///
/// The first half of the contract. Red before the variant existed (there was
/// no third kind to author); red again if `body_desc` mapped it to `Dynamic`,
/// which is the mapping typo this catches — it would drop 1.2 m in 50 ticks.
#[test]
fn a_kinematic_body_ignores_gravity_and_keeps_its_authored_pose() {
    let mut sim = SimWorld::new();
    let e = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(1.5, 3.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=50 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        pose(&sim, e),
        (1.5, 3.0, 0.0),
        "the solver moved a body whose pose the SCENE owns — `readback` must \
         skip every kind but Dynamic"
    );
}

/// **A kinematic body goes exactly where the `Transform` says — in RAPIER.**
///
/// ⚠️ The oracle is the SOLVER's pose, not the entity's `Transform`, and the
/// first version of this gate got that wrong in a way that made it vacuous:
/// `readback` skips kinematic bodies, so asserting that `Transform.x` still
/// holds the value the test just wrote asserts something no code path can
/// falsify. It stayed **green with the whole drive stage deleted**. What it
/// actually duplicated was `the_readback_does_not_erase_a_scene_owned_pose`,
/// under a name claiming to measure the drive.
///
/// Asking the bridge where the body really is makes the aim the only thing
/// that can put it there.
#[test]
fn a_kinematic_body_follows_the_transform_it_is_given() {
    let mut sim = SimWorld::new();
    let e = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(0.0, 1.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=20 {
        // The scene writes the pose — a curve would write exactly here.
        set_x(&mut sim, e, tick as f32 * 0.05);
        bridge.dispatch(&mut sim, true, tick);
        let solver_x = bridge.body_pose(e).expect("the body is in the world").0;
        assert!(
            (solver_x - tick as f32 * 0.05).abs() < 1e-5,
            "tick {tick}: the scene put the body at {:.4} and the solver has it \
             at {solver_x:.4} — the aim did not reach rapier",
            tick as f32 * 0.05
        );
    }
}

/// **How the frame was CHUNKED cannot change the result.**
///
/// One dispatch owing N ticks must land where N dispatches of one tick land.
/// The artist has no way to know whether a frame owed one tick or six — a
/// stutter, a slow frame, or a drag of the ruler all arrive as a bigger jump —
/// so a result that depends on the chunking is a result that changes for
/// reasons nobody can see.
///
/// Born RED over a real defect: an aim is spent by the step it is for, so
/// aiming once per DISPATCH fed the first step and left the rest with none.
/// The body crossed the whole span inside one tick at N× speed and then stood
/// still, which reads on screen as the platform hurling its cargo away.
/// Measured at the time: riding at `x = 1.049` tick-by-tick, flung to
/// `x = -0.520` in one jump.
#[test]
fn a_multi_tick_dispatch_lands_where_tick_by_tick_lands() {
    let travel = |chunk: u64| {
        let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=30 {
            bridge.dispatch(&mut sim, true, tick);
        }
        let start_x = pose(&sim, cargo).0;
        // 120 ticks of travel, delivered `chunk` ticks at a time. The scene
        // writes the platform's pose per FRAME, exactly as a curve would.
        let mut tick = 30u64;
        let mut i = 0u64;
        while i < 120 {
            i += chunk;
            set_x(&mut sim, platform, 1.0 * i as f32 / 120.0);
            tick += chunk;
            bridge.dispatch(&mut sim, true, tick);
        }
        pose(&sim, cargo).0 - start_x
    };

    let one = travel(1);
    assert!(
        one > 0.5,
        "the fixture is not carrying anything ({one:.3} m) — the comparison \
         below would be between two failures"
    );
    for chunk in [2u64, 3, 6] {
        let n = travel(chunk);
        assert!(
            (n - one).abs() < 0.15,
            "delivered {chunk} ticks at a time the cargo travelled {n:.3} m, \
             but one tick at a time it travelled {one:.3} m — the result \
             depends on how the frame happened to be chunked"
        );
    }
}

/// **A kinematic platform CARRIES what rests on it.**
///
/// The gate the variant exists for, and the one that separates it from every
/// cheaper thing that looks the same standing still:
///
/// - `Static` — the platform never moves in rapier at all, so the cargo stays
///   put and travels **0**.
/// - drive stage missing — identical to the above.
/// - `set_body_pose` instead of aiming — the platform teleports with **zero
///   velocity**, so the contact has no tangential motion to read, friction has
///   nothing to transmit, and the cargo is left behind.
///
/// The oracle is what the artist would SEE: the cargo arrives near the far end
/// of the platform's travel, not near where it started. The bar is deliberately
/// loose (friction is a real coefficient, not a weld — the box slips a little)
/// but the fossa between "carried" and "left behind" is the platform's whole
/// 1.0 m of travel.
#[test]
fn a_kinematic_platform_carries_what_rests_on_it() {
    let travel = 1.0f32;
    let ticks = 120u64;

    let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
    let mut bridge = PhysicsBridge::new();
    // Let it settle onto the platform first, so the contact exists before the
    // ride starts. A box dropped and dragged in the same instant would measure
    // the settling, not the carrying.
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let start_x = pose(&sim, cargo).0;
    for i in 1..=ticks {
        set_x(&mut sim, platform, travel * i as f32 / ticks as f32);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let carried = pose(&sim, cargo).0 - start_x;

    assert!(
        carried > travel * 0.5,
        "the cargo travelled {carried:.3} m of the platform's {travel:.3} m — a \
         kinematic body that does not push is a Static body with a nicer name"
    );

    // The control: the SAME scene with a static platform, whose Transform is
    // moved identically. Nothing must ride along — this is what proves the
    // measurement above is reading the drive stage and not gravity, drift, or
    // a lucky initial nudge.
    let (mut sim, platform, cargo) = platform_scene(BodyKind::Static);
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let start_x = pose(&sim, cargo).0;
    for i in 1..=ticks {
        set_x(&mut sim, platform, travel * i as f32 / ticks as f32);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let unmoved = (pose(&sim, cargo).0 - start_x).abs();
    assert!(
        unmoved < travel * 0.1,
        "a STATIC platform carried its cargo {unmoved:.3} m — the control is \
         supposed to be the thing that does not happen"
    );
}

/// **The readback never writes a pose the SCENE owns.**
///
/// `readback` asks `BodyKind::solver_owns_pose` and skips everything else. The
/// obvious place to test that is a kinematic body — and it is the wrong place:
/// the drive stage puts the body exactly where the `Transform` said, and the
/// wrapper's final sub-step is the target *bit-exactly*, so writing it back
/// writes the identical number. Deleting the guard passes such a gate.
///
/// The case where it is observable is a **static** body moved during PLAY.
/// `settle` only runs while paused, so rapier still holds the old pose; without
/// the guard the readback pushes that stale pose back over the artist's edit,
/// and the wall the artist just dragged snaps home every frame.
#[test]
fn the_readback_does_not_erase_a_scene_owned_pose() {
    let mut sim = SimWorld::new();
    let wall = spawn_body(
        &mut sim,
        BodyKind::Static,
        ColliderShape::Cuboid {
            half_x: 0.2,
            half_y: 1.0,
        },
        Vec2::new(0.0, 0.0),
    );
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=10 {
        bridge.dispatch(&mut sim, true, tick);
    }
    // The artist drags the wall while the clock is running.
    set_x(&mut sim, wall, 4.0);
    for tick in 11..=20 {
        bridge.dispatch(&mut sim, true, tick);
    }
    assert_eq!(
        pose(&sim, wall).0,
        4.0,
        "the readback pushed rapier's stale pose over an edit the artist made \
         — a body the solver does not own must not be written back"
    );
}

/// **A kinematic body shoves a dynamic one out of its way.**
///
/// The other half of "pushes": not friction along a surface but a head-on
/// contact. A ram sweeps across a resting ball; the ball must end up beyond
/// where the ram's leading face reached, i.e. it was moved rather than
/// tunnelled through or ignored.
#[test]
fn a_kinematic_body_shoves_a_dynamic_one() {
    let mut sim = SimWorld::new();
    // A floor to rest on, so the ball's motion is horizontal and unambiguous.
    spawn_body(
        &mut sim,
        BodyKind::Static,
        ColliderShape::Cuboid {
            half_x: 20.0,
            half_y: 0.1,
        },
        Vec2::new(0.0, 0.0),
    );
    let ram = spawn_body(
        &mut sim,
        BodyKind::Kinematic,
        ColliderShape::Cuboid {
            half_x: 0.2,
            half_y: 0.5,
        },
        Vec2::new(-2.0, 0.6),
    );
    let ball = spawn_body(
        &mut sim,
        BodyKind::Dynamic,
        ColliderShape::Ball { radius: 0.25 },
        Vec2::new(0.0, 0.35),
    );

    let mut bridge = PhysicsBridge::new();
    for tick in 1..=30 {
        bridge.dispatch(&mut sim, true, tick);
    }
    let rest_x = pose(&sim, ball).0;
    // Sweep the ram from x=-2 to x=0 — it arrives where the ball is standing.
    for i in 1..=100u64 {
        set_x(&mut sim, ram, -2.0 + 2.0 * i as f32 / 100.0);
        bridge.dispatch(&mut sim, true, 30 + i);
    }
    let pushed = pose(&sim, ball).0 - rest_x;
    assert!(
        pushed > 0.1,
        "the ram passed through the ball ({pushed:.3} m of displacement) — a \
         kinematic body must carry momentum into the contact, not haunt it"
    );
}

/// How far the stand-in "curve" moves the platform per tick.
///
/// Fast enough that the 7 ticks a scrub replays are VISIBLE: at a tenth of this
/// the platform's whole replay error fitted inside the tolerance, and a mutation
/// that drove the body without asking the scene where it belonged survived. A
/// fixture has to contain the phenomenon it is measuring.
const PLATFORM_SPEED: f32 = 0.03;

/// A scene whose kinematic platform follows a known "curve":
/// `x = tick · PLATFORM_SPEED`. Stands in for the timeline, which is what the
/// shell supplies.
struct MovingPlatform {
    platform: Entity,
}

impl ph2d_physics_ecs::SceneAtTick for MovingPlatform {
    fn put(&mut self, sim: &mut SimWorld, tick: u64) -> bool {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(self.platform) {
            t.translation.x = tick as f32 * PLATFORM_SPEED;
        }
        true
    }
}

/// **A scrub lands where a play lands, with a scene-driven body in the scene.**
///
/// The invariant the whole bridge rests on is that the world is a function of
/// the tick. `Kinematic` broke it: the pose is an INPUT, and the replay used to
/// run with `world.step()` alone, so a kinematic body sat frozen at its rest
/// pose for the entire replay. A platform that is not where it was puts every
/// dynamic body that touched it somewhere else.
///
/// Born RED, and worse than merely wrong — the answer depended on the CACHE:
/// a partial replay off a ring anchor diverged by ~3.4 cm, while a ring miss
/// (any settings change or spawn clears it) rebuilt from rest and the cargo
/// never travelled at all. Same gesture, different result on different days.
///
/// ⚠️ **Measured AT the scrub target, not after playing on from it.** The first
/// version of this gate scrubbed back to 40 and ran forward to 90 again, then
/// compared endpoints — and the forward run is driven correctly, so it washed
/// the replay's error out: a box on a platform is a damped system and it
/// re-converges. Both mutations that delete the replay's scene driving survived
/// that. It is the W1.5 lesson (*"o oráculo do scrub é a TRAJETÓRIA, não o
/// endpoint"*) re-learned inside a gate written to honour it.
#[test]
fn a_scrub_lands_where_the_play_landed_with_a_driven_body() {
    // ⚠️ NOT a multiple of the checkpoint STRIDE. At tick 40 the ring holds an
    // anchor exactly there, so the scrub seeds from it and replays ZERO steps —
    // the replay loop never runs and every mutation that breaks it survives. 37
    // seeds at 30 and replays 7, which is the path being measured.
    const TARGET: u64 = 37;

    // The truth: play straight to TARGET, one tick at a time.
    let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
    let mut scene = MovingPlatform { platform };
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=TARGET {
        bridge.dispatch_with_scene(&mut sim, true, tick, &mut scene);
    }
    let played = pose(&sim, cargo);

    // The same tick, arrived at BACKWARDS — the replay path.
    let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
    let mut scene = MovingPlatform { platform };
    let mut bridge = PhysicsBridge::new();
    for tick in 1..=90 {
        bridge.dispatch_with_scene(&mut sim, true, tick, &mut scene);
    }
    bridge.dispatch_with_scene(&mut sim, false, TARGET, &mut scene);
    let scrubbed = pose(&sim, cargo);

    assert!(
        played.0 > 0.1,
        "the fixture's platform carried the cargo only {:.4} m by tick \
         {TARGET}, so the comparison below would hold between two failures",
        played.0
    );
    assert!(
        (played.0 - scrubbed.0).abs() < 0.02 && (played.1 - scrubbed.1).abs() < 0.02,
        "playing to tick {TARGET} left the cargo at ({:.4}, {:.4}) and scrubbing \
         BACK to it left it at ({:.4}, {:.4}) — the replay did not put the \
         driven body where it was, so the scene it replayed is not the scene \
         that ran",
        played.0,
        played.1,
        scrubbed.0,
        scrubbed.1
    );
}

/// **A forward JUMP with a scene lands where tick-by-tick with a scene lands.**
///
/// The sibling of `a_multi_tick_dispatch_lands_where_tick_by_tick_lands`, for
/// the case where the scene can answer per tick. The two are different code
/// paths on purpose: with no curve to consult the bridge SLICES the frame's
/// move across the ticks it owes (the honest reconstruction for a hand drag),
/// and with a curve it takes the exact pose for each tick. Using the slice when
/// an exact answer exists interpolates from a stale start and lags.
///
/// Dragging the ruler forward is the gesture; the artist cannot tell whether
/// that arrived as one jump or sixty frames.
#[test]
fn a_forward_jump_with_a_scene_lands_where_tick_by_tick_lands() {
    let run = |chunk: u64| {
        let (mut sim, platform, cargo) = platform_scene(BodyKind::Kinematic);
        let mut scene = MovingPlatform { platform };
        let mut bridge = PhysicsBridge::new();
        for tick in 1..=30 {
            bridge.dispatch_with_scene(&mut sim, true, tick, &mut scene);
        }
        let mut tick = 30u64;
        while tick < 90 {
            tick = (tick + chunk).min(90);
            bridge.dispatch_with_scene(&mut sim, true, tick, &mut scene);
        }
        pose(&sim, cargo)
    };

    let one = run(1);
    assert!(
        one.0 > 0.5,
        "the fixture carried the cargo only {:.4} m — nothing to compare",
        one.0
    );
    for chunk in [5u64, 20, 60] {
        let n = run(chunk);
        assert!(
            (n.0 - one.0).abs() < 0.05 && (n.1 - one.1).abs() < 0.05,
            "arriving at tick 90 in jumps of {chunk} left the cargo at \
             ({:.4}, {:.4}), but one tick at a time left it at ({:.4}, {:.4}) — \
             the scene answered for every tick and the bridge used something \
             else",
            n.0,
            n.1,
            one.0,
            one.1
        );
    }
}
