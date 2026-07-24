//! **The FRAME of a force zone** — in whose axes is the push? (ADR-0131 W-AreaFrame).
//!
//! Before this wave the force was always in WORLD axes, so **rotating a zone did not
//! rotate the wind** — a diagonal conveyor was inexpressible, and a wind column turned
//! to fit the scene kept blowing the old way with nothing on screen to say why. The
//! default is now the zone's own frame; the `world_axes` flag is the escape that pins
//! the direction back to the world (Unity's `AreaEffector2D::useGlobalAngle`).
//!
//! ⚠️ **Every fixture sweeps the spawn order**, for the reason the sibling `effector.rs`
//! states at the top: the zone and its target reach the narrow phase as an UNORDERED
//! pair, and this is the line where a sign flip survived three gates because the
//! platform always happened to be spawned first.

use ph2d_physics::{
    AreaEffect, BodyDesc, PhysicsWorld, RigidBodyHandle, RigidBodyType, ShapeDesc,
    zone_force_world,
};

/// A body with everything neutral (the sibling file's helper, kept local so the two
/// gate files stay independent).
fn desc(body_type: RigidBodyType, x: f32, y: f32, shape: ShapeDesc) -> BodyDesc {
    BodyDesc {
        body_type,
        x,
        y,
        rotation: 0.0,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        offset: [0.0, 0.0],
        lock_x: false,
        lock_y: false,
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
    }
}

/// A sensor box at the origin carrying a force, ROTATED by `rotation` radians, in the
/// frame `world_axes` selects. Square and centred, so turning it does not change WHO is
/// inside — the only thing the rotation can move is the direction of the push, which is
/// exactly the variable under test.
fn zone(rotation: f32, force: [f32; 2], world_axes: bool, kind: RigidBodyType) -> BodyDesc {
    BodyDesc {
        is_sensor: true,
        rotation,
        effector: Some(AreaEffect {
            force,
            drag: 0.0,
            density: 0.0,
            form_drag: 0.0,
            torque: 0.0,
            world_axes,
        }),
        ..desc(
            kind,
            0.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 3.0,
                half_y: 3.0,
            },
        )
    }
}

fn ball() -> BodyDesc {
    desc(
        RigidBodyType::Dynamic,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.25 },
    )
}

fn spawn(
    w: &mut PhysicsWorld,
    z: BodyDesc,
    b: BodyDesc,
    zone_first: bool,
) -> (RigidBodyHandle, RigidBodyHandle) {
    if zone_first {
        let zh = w.spawn_body(z);
        (zh, w.spawn_body(b))
    } else {
        let bh = w.spawn_body(b);
        (w.spawn_body(z), bh)
    }
}

fn pos(w: &PhysicsWorld, h: RigidBodyHandle) -> (f32, f32) {
    let p = w.body_pose(h).expect("body alive").translation;
    (p.x, p.y)
}

/// Zero gravity: the zone is then the ONLY thing acting on the ball, so the direction
/// it ends up travelling is the direction of the push and nothing else.
fn run(z: BodyDesc, zone_first: bool) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    w.set_gravity(0.0, 0.0);
    let (_z, b) = spawn(&mut w, z, ball(), zone_first);
    for _ in 0..60 {
        w.step();
    }
    pos(&w, b)
}

/// **The wave, in one sentence: turn the zone and the wind turns with it.**
///
/// A zone rotated a quarter turn carries a force authored along its own +X, so the ball
/// must leave along world **+Y**. Red before this wave by construction: the old kernel
/// handed `effect.force` to the impulse untouched, which sends it along +X.
#[test]
fn a_rotated_zone_turns_its_wind() {
    for zone_first in [true, false] {
        let (x, y) = run(
            zone(
                std::f32::consts::FRAC_PI_2,
                [3.0, 0.0],
                false,
                RigidBodyType::Fixed,
            ),
            zone_first,
        );
        assert!(
            y > 0.5,
            "zone_first={zone_first}: a zone turned a quarter turn must blow along +Y, \
             but the ball is at ({x}, {y})"
        );
        assert!(
            x.abs() < 1e-3,
            "zone_first={zone_first}: nothing should push along X any more, x={x}"
        );
    }
}

/// **The escape.** The same rotated zone, with the force pinned to world axes: the zone
/// turns, the blow does not. This is the half that makes the toggle a toggle — without
/// it the wave would be a default change wearing a flag.
#[test]
fn pinning_the_force_to_world_axes_keeps_the_blow_where_it_was() {
    for zone_first in [true, false] {
        let (x, y) = run(
            zone(
                std::f32::consts::FRAC_PI_2,
                [3.0, 0.0],
                true,
                RigidBodyType::Fixed,
            ),
            zone_first,
        );
        assert!(
            x > 0.5,
            "zone_first={zone_first}: pinned to world axes the push stays along +X, \
             but the ball is at ({x}, {y})"
        );
        assert!(
            y.abs() < 1e-3,
            "zone_first={zone_first}: nothing should push along Y here, y={y}"
        );
    }
}

/// **The regression pin, and the reason the default could change at all.**
///
/// On an UNROTATED zone the two frames are the same statement, and they must agree on
/// the BITS — `sin 0` is exactly `0.0` and `cos 0` exactly `1.0`, so the rotated branch
/// reduces to the identity. It is this that made the new default free: on the day of
/// this wave not one force zone in the repository — smoke scene or fixture — carried a
/// non-zero rotation, so every existing scene keeps its exact trajectory.
#[test]
fn an_unrotated_zone_is_bit_identical_in_both_frames() {
    for zone_first in [true, false] {
        let local = run(zone(0.0, [2.0, 1.0], false, RigidBodyType::Fixed), zone_first);
        let world = run(zone(0.0, [2.0, 1.0], true, RigidBodyType::Fixed), zone_first);
        assert_eq!(
            local.0.to_bits(),
            world.0.to_bits(),
            "zone_first={zone_first}: an unrotated zone must be bit-identical in both \
             frames, x {} vs {}",
            local.0,
            world.0
        );
        assert_eq!(
            local.1.to_bits(),
            world.1.to_bits(),
            "zone_first={zone_first}: ... and on Y too, {} vs {}",
            local.1,
            world.1
        );
    }
}

/// **The frame is the zone's LIVE pose, not the one it spawned with.**
///
/// A KINEMATIC zone a curve is turning — a fan sweeping the room — must sweep its blow
/// with it. Baking the rotation into the spawn `BodyDesc` would forbid that in silence,
/// and every gate above would stay green: they all use a zone whose live pose never
/// leaves the one it was born with, so for them "live" and "spawn" are the same number.
///
/// ⚠️ **The fixture spawns at a quarter turn and is driven to a half turn**, which is
/// what makes it tell THREE implementations apart instead of two — the first draft
/// started at zero, where *not rotating at all* and *rotating by the spawn pose* are
/// indistinguishable:
///
/// | kernel | blows | ends at |
/// |---|---|---|
/// | live pose (the product) | sweeps +Y round to −X | `y > 0`, **`x < 0`** |
/// | baked at spawn (π/2) | +Y, always | `y > 0`, `x ≈ 0` |
/// | no rotation at all | +X, always | `y ≈ 0`, `x > 0` |
///
/// So the `x < 0` assertion is the load-bearing one: only the live read can put the ball
/// on the negative X axis, because only it ever points the blow that way.
#[test]
fn the_frame_is_the_zones_live_pose_not_the_one_it_spawned_with() {
    for zone_first in [true, false] {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let (z, b) = spawn(
            &mut w,
            zone(
                std::f32::consts::FRAC_PI_2,
                [3.0, 0.0],
                false,
                RigidBodyType::KinematicPositionBased,
            ),
            ball(),
            zone_first,
        );
        // Drive the housing from a quarter turn round to a half turn, a tick at a time.
        for i in 0..60u8 {
            let f = f32::from(i) / 60.0;
            let angle = std::f32::consts::FRAC_PI_2 * (1.0 + f);
            w.set_next_kinematic_pose(z, 0.0, 0.0, angle);
            w.step();
        }
        let (x, y) = pos(&w, b);
        assert!(
            y > 0.2,
            "zone_first={zone_first}: the fan starts pointing +Y, so the ball must gain \
             +Y — it is at ({x}, {y})"
        );
        assert!(
            x < -0.05,
            "zone_first={zone_first}: the blow must SWEEP with the housing, round towards \
             −X — the ball is at ({x}, {y}). A kernel that baked the spawn rotation would \
             leave x at ~0; one that ignored the frame would push it to +X."
        );
    }
}

/// **The torque is invariant under the zone's rotation, and that is geometry.**
///
/// A 2D torque is a scalar about Z, and an in-plane rotation is a rotation about Z — so
/// `τ_local ≡ τ_world` and there is nothing for a frame to turn. This gate exists so that
/// nobody later "completes" the wave by making the torque frame-dependent too.
///
/// ⚠️ **The first version of this gate compared the two FLAGS at one rotation, and a
/// mutation that scaled the torque by `cos_r` sailed through it** — because that defect
/// scales BOTH flags by the same factor, so the ratio it measured was healthy over two
/// sick numbers ([[feedback_two_quantities_that_should_differ_can_coincide_by_fixture_phase]]).
/// The property that actually holds is invariance under the **ROTATION**, so that is what
/// is asserted, across the flag as well for completeness.
#[test]
fn the_torque_does_not_care_how_the_zone_is_turned() {
    let spin = |rotation: f32, world_axes: bool| {
        let mut w = PhysicsWorld::new();
        w.set_gravity(0.0, 0.0);
        let mut z = zone(rotation, [0.0, 0.0], world_axes, RigidBodyType::Fixed);
        if let Some(e) = z.effector.as_mut() {
            e.torque = 1.5;
        }
        let (_z, b) = spawn(&mut w, z, ball(), true);
        for _ in 0..60 {
            w.step();
        }
        w.bodies().get(b).expect("body alive").angvel()
    };
    let upright = spin(0.0, false);
    assert!(upright.abs() > 0.1, "the fixture must actually spin: {upright}");
    // The load-bearing one: TURNING the zone may not change the spin it imparts.
    for rotation in [0.7, std::f32::consts::FRAC_PI_2, -2.1] {
        let turned = spin(rotation, false);
        assert_eq!(
            upright.to_bits(),
            turned.to_bits(),
            "a 2D torque is about Z and an in-plane rotation leaves Z fixed, so turning \
             the zone by {rotation} rad must not change the spin: {upright} vs {turned}"
        );
    }
    // ... and the flag, which governs the force alone, may not reach it either.
    assert_eq!(
        upright.to_bits(),
        spin(0.7, true).to_bits(),
        "the force's frame flag must not reach the torque"
    );
}

/// The door itself, on the two cases the sim cannot show cheaply: that `world_axes`
/// returns the authored vector **untouched** (not "rotated by zero"), and that the
/// rotation is the ordinary CCW one.
#[test]
fn the_door_rotates_ccw_and_passes_a_pinned_force_through_untouched() {
    // Pinned: byte-for-byte the same vector, whatever the pose says.
    let f = [1.25, -3.5];
    let out = zone_force_world(f, true, 0.87, 0.49);
    assert_eq!(out[0].to_bits(), f[0].to_bits());
    assert_eq!(out[1].to_bits(), f[1].to_bits());
    // A quarter turn CCW sends +X to +Y (sin = 1, cos = 0).
    let out = zone_force_world([2.0, 0.0], false, 1.0, 0.0);
    assert!(out[0].abs() < 1e-6 && (out[1] - 2.0).abs() < 1e-6, "{out:?}");
}
