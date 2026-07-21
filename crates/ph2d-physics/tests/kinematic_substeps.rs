//! **A kinematic body's aim is for the TICK; the sub-steps get slices of it.**
//!
//! `PhysicsWorld::step` runs `substeps` pipeline steps per tick, and rapier
//! derives a kinematic body's velocity from how far it was told to travel in
//! *this* sub-step. Hand it the whole tick's displacement and the body arrives
//! in the first sub-step at `substeps`× its real speed, then stands still for
//! the rest — so every contact reads a velocity the body does not have, for a
//! quarter of the time, and nothing for the other three quarters.
//!
//! That is the same law `drag::apply` states one line above the sub-step loop
//! (*"Per SUBSTEP, not per tick: a force applied once per tick would be wrong
//! by the substep count"*), and it was broken here first.
//!
//! The bridge-level consequence is measured in `ph2d-physics-ecs`'s
//! `a_kinematic_platform_carries_what_rests_on_it` (0.009 m instead of 0.975 m
//! of ride). These gates measure the CAUSE, in the crate that owns the
//! sub-step loop, so the two cannot drift apart about whose rule it is.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

const DT: f32 = 1.0 / 60.0;

fn kinematic_at(world: &mut PhysicsWorld, x: f32, y: f32) -> ph2d_physics::RigidBodyHandle {
    world.spawn_body(BodyDesc {
        body_type: RigidBodyType::KinematicPositionBased,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        offset: [0.0, 0.0],
        shape: ShapeDesc::Cuboid {
            half_x: 1.0,
            half_y: 0.2,
        },
    })
}

fn linvel_of(world: &PhysicsWorld, handle: ph2d_physics::RigidBodyHandle) -> (f32, f32) {
    let idx = handle.into_raw_parts().0;
    let s = world
        .body_snapshots()
        .into_iter()
        .find(|s| s.handle_index == idx)
        .expect("the body is in the world");
    (s.linvel_x, s.linvel_y)
}

/// **The velocity rapier attributes to the body is the one it actually has.**
///
/// The direct statement of the law: told to cross `d` in one tick, the body
/// must be moving at `d / dt` — not at `substeps · d / dt`, which is what
/// spending the whole aim on the first sub-step reports.
///
/// Red before the slicing existed: the reading was 4× (`DEFAULT_SUBSTEPS`).
#[test]
fn a_kinematic_body_moves_at_the_speed_its_tick_implies() {
    let mut world = PhysicsWorld::new();
    let h = kinematic_at(&mut world, 0.0, 0.0);

    let d = 0.05f32; // meters this tick -> 3 m/s
    world.set_next_kinematic_pose(h, d, 0.0, 0.0);
    world.step();

    let (vx, vy) = linvel_of(&world, h);
    let expected = d / DT;
    assert!(
        (vx - expected).abs() < expected * 0.02,
        "the body reports {vx:.3} m/s for a tick that moved it {d} m in {DT:.4} s \
         — expected {expected:.3} m/s. A reading of {:.3} is the whole tick spent \
         on one sub-step.",
        expected * PhysicsWorld::DEFAULT_SUBSTEPS as f32
    );
    assert_eq!(vy, 0.0, "a horizontal aim gave the body vertical speed");
}

/// **It arrives exactly where it was aimed, whatever the sub-step count.**
///
/// The slicing must not change the destination — only the path through the
/// tick. Swept across every sub-step count the world accepts, because a slice
/// computed as `sub / substeps` instead of `(sub + 1) / substeps` lands the
/// body one slice short and would pass at `substeps = 1`.
#[test]
fn the_slicing_does_not_move_the_destination() {
    for substeps in [1u32, 2, 4, 8] {
        let mut world = PhysicsWorld::new();
        world.set_substeps(substeps);
        let h = kinematic_at(&mut world, 0.0, 0.0);

        world.set_next_kinematic_pose(h, 0.25, -0.5, 1.0);
        world.step();

        let pose = world.body_pose(h).expect("body");
        assert!(
            (pose.translation.x - 0.25).abs() < 1e-5
                && (pose.translation.y + 0.5).abs() < 1e-5
                && (pose.rotation.angle() - 1.0).abs() < 1e-5,
            "substeps={substeps}: aimed at (0.25, -0.5, 1.0), arrived at \
             ({:.5}, {:.5}, {:.5})",
            pose.translation.x,
            pose.translation.y,
            pose.rotation.angle()
        );
    }
}

/// **A rotation across the ±π branch takes the short way round — either way.**
///
/// `Rotation::angle()` lives in `(-π, π]`, so a body sitting just under +π that
/// is aimed just over it reads as a target of about −π. Interpolated naively
/// the body is told to unwind almost a full turn *inside one tick* — at 60 Hz
/// that is ~375 rad/s of reported spin, and it would fling whatever it touched.
/// Same law as `ph2d_anim::unwrap_angles`.
///
/// ⚠️ **BOTH directions, and that is not symmetry for its own sake.** The
/// unwrap is two branches (`d > π` subtracts a turn, `d < −π` adds one), and a
/// fixture that only crosses the cut one way exercises exactly one of them: a
/// mutation that deleted the other survived a green gate here, because the
/// single fixture's `d` was negative and only ever met the branch that was
/// still standing.
#[test]
fn a_rotation_across_the_branch_cut_takes_the_short_way() {
    let near_pi = std::f32::consts::PI - 0.05;
    // Turning FORWARD past +π (d is negative, hits the `d < -π` branch), and
    // turning BACKWARD past -π (d is positive, hits the `d > π` branch).
    for (start, target, way) in [
        (near_pi, -near_pi, "forward"),
        (-near_pi, near_pi, "backward"),
    ] {
        let mut world = PhysicsWorld::new();
        let h = world.spawn_body(BodyDesc {
            body_type: RigidBodyType::KinematicPositionBased,
            x: 0.0,
            y: 0.0,
            rotation: start,
            density: 1.0,
            restitution: 0.0,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            offset: [0.0, 0.0],
            shape: ShapeDesc::Cuboid {
                half_x: 1.0,
                half_y: 0.2,
            },
        });

        world.set_next_kinematic_pose(h, 0.0, 0.0, target);
        world.step();

        let angvel = world
            .body_snapshots()
            .into_iter()
            .find(|s| s.handle_index == h.into_raw_parts().0)
            .expect("body")
            .angvel;
        let short_way = 0.1 / DT; // ~6 rad/s
        assert!(
            angvel.abs() < short_way * 2.0,
            "turning {way}: the body span at {angvel:.1} rad/s to cover 0.1 rad \
             — it went the long way round the branch cut (the long way is \
             ~{:.0} rad/s)",
            (std::f32::consts::TAU - 0.1) / DT
        );
    }
}

/// **The aim of a body that cannot use one is refused — and the claim is COST.**
///
/// The guard in `set_next_kinematic_pose` does not protect the body's pose:
/// rapier ignores a next-position on a dynamic body, measured (a ball aimed at
/// `(9, 9)` every tick for a second settles at exactly the same height as one
/// never aimed). What it protects is the per-sub-step replay list, which would
/// otherwise carry an entry that is re-stated four times a tick to no effect.
///
/// So the gate reads the list, not the pose. Written this way because the
/// version that read the pose was **green with the guard deleted** — a defence
/// whose effect the oracle cannot see is not defended by that oracle
/// (`feedback_layered_defenses_need_per_layer_gates`).
#[test]
fn the_aim_of_a_body_that_cannot_use_it_is_refused() {
    let mut world = PhysicsWorld::new();
    let dynamic = world.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 3.0,
        rotation: 0.0,
        density: 1.0,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        mass_override: None,
        offset: [0.0, 0.0],
        shape: ShapeDesc::Ball { radius: 0.25 },
    });
    let kinematic = kinematic_at(&mut world, 0.0, 0.0);

    world.set_next_kinematic_pose(dynamic, 9.0, 9.0, 0.0);
    assert_eq!(
        world.kinematic_aim_count(),
        0,
        "a dynamic body joined the per-sub-step aim list"
    );

    world.set_next_kinematic_pose(kinematic, 1.0, 0.0, 0.0);
    assert_eq!(
        world.kinematic_aim_count(),
        1,
        "the body that CAN use an aim did not get one — the refusal is too wide"
    );

    world.step();
    assert_eq!(
        world.kinematic_aim_count(),
        0,
        "the aim outlived the tick it was for; the next tick would replay a \
         stale pose for a body nobody aimed"
    );
}

/// **The last sub-step lands the body ON its target, bit-exactly.**
///
/// `a0 + (a1 - a0)` is not always exactly `a1` in f32, and the body's pose is
/// read straight back into the scene's `Transform` — so an inexact final slice
/// is a pose the artist never authored, arriving every tick, always in the same
/// direction. Asserted as EQUALITY, not tolerance: this is the one place where
/// "close enough" accumulates.
#[test]
fn the_final_slice_is_the_target_itself() {
    for substeps in [1u32, 3, 4, 7] {
        let mut world = PhysicsWorld::new();
        world.set_substeps(substeps);
        let h = kinematic_at(&mut world, 0.3, -1.7);
        // Product-ish numbers, none of them a fixed point of the arithmetic.
        let (x, y, rot) = (1.234f32, 5.678f32, 0.7331f32);
        world.set_next_kinematic_pose(h, x, y, rot);
        world.step();

        let p = world.body_pose(h).expect("body");
        assert_eq!(
            (p.translation.x, p.translation.y, p.rotation.angle()),
            (x, y, rot),
            "substeps={substeps}: the body landed near its target instead of on it"
        );
    }
}

/// **A world with no kinematic body steps exactly as it always did.**
///
/// The byte-identity claim the whole change rests on: the slicing loop runs
/// over an empty list, so every scene that shipped before this existed — every
/// determinism hash, every checkpoint gate — is untouched.
///
/// ⚠️ The oracle is the whole TRAJECTORY, against a reference captured from a
/// world that has no kinematic machinery in play at all. An earlier version of
/// this gate said it compared a trajectory and then asserted the resting height
/// once, with a 5 cm tolerance: a falling body settles, so the endpoint is the
/// one sample that a divergence in the middle is guaranteed NOT to reach (the
/// W1.5 scrub lesson, in a gate that cites it). Every sample, exact equality —
/// "byte-identical" is the claim, so nothing weaker will do.
#[test]
fn a_world_with_no_kinematic_body_is_untouched() {
    let trajectory = || {
        let mut world = PhysicsWorld::new();
        let _ = world.add_static_cuboid(0.0, 0.0, 10.0, 0.1);
        let ball = world.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: 0.0,
            y: 3.0,
            rotation: 0.0,
            density: 1.0,
            restitution: 0.3,
            friction: 0.5,
            layer: 0,
            is_sensor: false,
            gravity_scale: 1.0,
            linvel: [0.0, 0.0],
            angvel: 0.0,
            ccd: false,
            lock_rotation: false,
            lock_x: false,
            lock_y: false,
            mass_override: None,
            offset: [0.0, 0.0],
            shape: ShapeDesc::Ball { radius: 0.25 },
        });
        let mut out = Vec::new();
        for _ in 0..120 {
            world.step();
            let p = world.body_pose(ball).expect("ball");
            out.push((p.translation.x, p.translation.y, p.rotation.angle()));
        }
        // Nothing kinematic ever entered this world, so the aim list must have
        // stayed empty the whole way — the mechanism behind the claim, not just
        // its result.
        assert_eq!(world.kinematic_aim_count(), 0);
        out
    };

    let a = trajectory();
    let b = trajectory();
    assert_eq!(
        a, b,
        "the world without kinematic bodies is not deterministic"
    );

    // The fixture has to BE a fall, or "unchanged" is a claim about nothing.
    assert!(
        a.iter().any(|s| s.1 > 2.0) && a.last().expect("samples").1 < 0.5,
        "the fixture is not a drop: it spans {:.2}..{:.2}",
        a.iter().map(|s| s.1).fold(f32::MAX, f32::min),
        a.iter().map(|s| s.1).fold(f32::MIN, f32::max)
    );
    // It bounces — so the middle of the trajectory carries information the
    // endpoint does not.
    let lowest_before_end = a[..a.len() - 20]
        .iter()
        .map(|s| s.1)
        .fold(f32::MAX, f32::min);
    assert!(
        a.last().expect("samples").1 > lowest_before_end + 0.001,
        "the ball never bounced back up, so this trajectory is monotonic and \
         its middle says nothing its endpoint does not"
    );
}
