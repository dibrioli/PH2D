//! **CCD stops a fast body that discrete detection lets tunnel** (W-CCD).
//!
//! Discrete collision detection tests a body only at each (sub-)step's END
//! pose, so a small fast body can be on one side of thin geometry at one pose
//! and clean past it at the next — never overlapping, never colliding. That is
//! the collision a game MISSES (a bullet through a wall), distinct from the deep
//! LANDING overlap the sub-stepping default fixes (a heavy body that is already
//! inside the floor the frame it touches). CCD sweeps the body's motion between
//! poses and stops it at the first impact.
//!
//! The gate is behavioural and red-first: a fast ball is launched straight at a
//! thin static wall. Without CCD it ends up on the FAR side (tunnelled); with
//! CCD it is stopped on the NEAR side. Mutating `spawn_body`'s
//! `.ccd_enabled(desc.ccd)` to `.ccd_enabled(false)` makes the CCD body tunnel
//! too, and the "stopped" assertion goes RED — the flag is the only thing
//! standing between the two outcomes.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Launch a small ball at `speed` m/s straight at a thin wall at x=0 and return
/// where it ends up after it has had time to cross. `ccd` toggles continuous
/// detection on the ball; everything else is identical.
///
/// Gravity is zeroed so the only motion is the horizontal launch — the tunnel
/// is a fact about detection, not a parabola that happens to clear the wall.
fn final_x_after_launch(ccd: bool, speed: f32) -> f32 {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);

    // A thin, TALL static wall at x=0: 0.04 m thick (half 0.02), 2 m tall so the
    // ball cannot go around it. This is the "thin geometry" a fast body tunnels.
    w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Fixed,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: 0.02,
            half_y: 1.0,
        },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
    });

    // A small ball, one metre to the LEFT, launched right at `speed` m/s. At
    // 80 m/s it moves 80/240 ≈ 0.33 m per sub-step (default 4 sub-steps) — far
    // more than the 0.04 m wall plus the ball's own 0.1 m diameter, so no
    // discrete pose ever overlaps the wall.
    let ball = w.spawn_body(BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: -1.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.05 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [speed, 0.0],
        angvel: 0.0,
        ccd,
    });

    // 30 ticks = 0.5 s: at 80 m/s the ball would travel 40 m unobstructed, so a
    // tunnelled ball is far past the wall and a stopped one has long since
    // settled against it.
    for _ in 0..30 {
        w.step();
    }
    w.body_pose(ball).expect("ball exists").translation.x
}

#[test]
fn a_discrete_body_tunnels_through_a_thin_wall_and_a_continuous_one_does_not() {
    // 200 m/s: measured discrete_x ≈ 99 (tunnelled clean through), ccd_x ≈ -0.07
    // (stopped at the wall). 80 m/s was NOT chosen because the 4 sub-steps happen
    // to sample a pose exactly inside the wall there — the tunnel is real but
    // alignment-sensitive, and 100..=600 m/s all tunnel with a wide margin.
    const SPEED: f32 = 200.0;

    // Discrete (rapier's default): the ball is never tested at a pose that
    // overlaps the wall, so it passes clean through and ends far to the RIGHT.
    let discrete_x = final_x_after_launch(false, SPEED);
    assert!(
        discrete_x > 0.5,
        "a discrete fast ball should tunnel through the thin wall and end past \
         it (x > 0.5), but it ended at x={discrete_x} — if this is no longer \
         true, the fixture no longer contains the phenomenon the gate exists to \
         prove and the numbers need re-choosing"
    );

    // Continuous: the sweep catches the wall and the ball is stopped on the NEAR
    // side. Its centre rests at roughly the wall's left face (-0.02) minus its
    // own radius (0.05) ≈ -0.07; the bar is simply "did not reach the wall".
    let ccd_x = final_x_after_launch(true, SPEED);
    assert!(
        ccd_x < 0.0,
        "a continuous fast ball should be stopped on the near side of the wall \
         (x < 0), but it ended at x={ccd_x} — CCD did not catch the collision \
         (mutating `.ccd_enabled(desc.ccd)` to `false` reproduces exactly this)"
    );
}
