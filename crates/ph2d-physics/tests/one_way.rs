//! **A one-way (jump-through) platform is solid from one side only.**
//!
//! The iconic 2D platformer collider: a body arriving from below passes clean through
//! and then LANDS on it coming back down. Realised by rapier's
//! `update_as_oneway_platform` through `world::oneway::OneWayHooks`; this proves the
//! integration — the `user_data` bit, the `MODIFY_SOLVER_CONTACTS` flag, and the
//! allowed-normal direction.
//!
//! Red-first and mutation-verified: stepping with `()` hooks instead of `OneWayHooks`
//! (or dropping the `user_data`/`active_hooks` on the collider) makes the platform
//! solid from both sides, so the body launched from below is stopped underneath and
//! the pass-through assertion fails.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// A platform at y=0 (optionally rotated, optionally one-way) plus a ball launched
/// from `start_y` at `vy`. Steps 3 s and returns `(max_y_reached, final_y)`.
///
/// ⚠️ `platform_first` chooses the SPAWN ORDER, and it is not decoration: rapier hands
/// the hook a `collider1`/`collider2` pair it orders itself, and the allowed normal is
/// expressed in **collider1's** frame — so the platform being second is a genuinely
/// different code path (the sign flip). A fixture that only ever spawns the platform
/// first leaves that branch unproven, which is exactly what it did until this
/// parameter existed: the mutation that deletes the flip passed every test.
fn platform_and_ball(
    rotation: f32,
    one_way: bool,
    start_y: f32,
    vy: f32,
    platform_first: bool,
) -> (f32, f32) {
    let mut w = PhysicsWorld::new();
    let base = |body_type, x: f32, y: f32, rot: f32, shape, vy: f32, one_way: bool| BodyDesc {
        body_type,
        x,
        y,
        rotation: rot,
        density: 1.0,
        shape,
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: 1.0,
        linvel: [0.0, vy],
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
        one_way,
    };
    let platform = base(
        RigidBodyType::Fixed,
        0.0,
        0.0,
        rotation,
        ShapeDesc::Cuboid {
            half_x: 5.0,
            half_y: 0.1,
        },
        0.0,
        one_way,
    );
    let ball_desc = base(
        RigidBodyType::Dynamic,
        0.0,
        start_y,
        0.0,
        ShapeDesc::Ball { radius: 0.25 },
        vy,
        false,
    );
    let ball = if platform_first {
        w.spawn_body(platform);
        w.spawn_body(ball_desc)
    } else {
        let b = w.spawn_body(ball_desc);
        w.spawn_body(platform);
        b
    };
    let mut max_y = start_y;
    for _ in 0..180 {
        w.step();
        let y = w.bodies().get(ball).expect("ball").position().translation.y;
        max_y = max_y.max(y);
    }
    let final_y = w.bodies().get(ball).expect("ball").position().translation.y;
    (max_y, final_y)
}

#[test]
fn a_body_from_below_passes_through_and_then_lands_on_top() {
    // BOTH spawn orders: the platform must behave the same whether rapier calls it
    // collider1 or collider2 (see `platform_and_ball`).
    for platform_first in [true, false] {
        // Launched UP from well under the platform at 8 m/s: it clears the surface,
        // reaches its apex above it, falls back, and RESTS on top (y ≈ 0.1 + 0.25).
        let (max_up, rest) = platform_and_ball(0.0, true, -2.0, 8.0, platform_first);
        assert!(
            max_up > 1.0,
            "platform_first={platform_first}: a body launched from below a ONE-WAY platform \
             should pass through it, but its highest point was y={max_up} — it was stopped \
             underneath (are the hooks installed? is the collider1/collider2 sign right?)"
        );
        assert!(
            (rest - 0.35).abs() < 0.1,
            "platform_first={platform_first}: after passing through, the body should LAND on \
             the platform (y ≈ 0.35), but it settled at y={rest} — not solid from above"
        );

        // The SOLID control: the identical launch is stopped underneath and never gets
        // above the surface. This is what the whole feature is measured against, and it
        // is exactly what a dropped hook turns the one-way case back into.
        let (max_solid, _) = platform_and_ball(0.0, false, -2.0, 8.0, platform_first);
        assert!(
            max_solid < 0.0,
            "platform_first={platform_first}: a body launched from below a SOLID platform must \
             be stopped underneath, but it reached y={max_solid} — the fixture no longer \
             contains the phenomenon"
        );
    }
}

#[test]
fn a_dropped_body_lands_on_a_one_way_platform() {
    // The other half: one-way does not mean "not solid". Dropped from above with no
    // launch, the body lands and stays — in either spawn order.
    for platform_first in [true, false] {
        let (_, rest) = platform_and_ball(0.0, true, 3.0, 0.0, platform_first);
        assert!(
            (rest - 0.35).abs() < 0.1,
            "platform_first={platform_first}: a body dropped onto a one-way platform should \
             rest on it (y ≈ 0.35), but it is at y={rest} — it fell through"
        );
    }
}

#[test]
fn the_solid_side_follows_the_platforms_own_rotation() {
    // ⚠️ The test the direction math earns. Turn the platform UPSIDE DOWN (π): its
    // local +Y — the solid side — now points DOWN in world, so a body dropped from
    // ABOVE is on the forbidden side and must fall straight through.
    //
    // Hard-coding world-up (or passing a constant ±Y for the collider1/collider2 cases,
    // as the rapier demo's axis-aligned fixture can afford to) keeps this body resting
    // on top and fails here.
    for platform_first in [true, false] {
        let (_, rest) = platform_and_ball(std::f32::consts::PI, true, 3.0, 0.0, platform_first);
        assert!(
            rest < -1.0,
            "platform_first={platform_first}: an UPSIDE-DOWN one-way platform is solid from \
             below, so a body dropped from above must pass through — but it settled at \
             y={rest}, meaning the solid side was taken from the world instead of from the \
             platform's own pose"
        );

        // And the same rotated platform still catches a body coming from underneath —
        // the direction flipped, it did not simply stop working.
        let (max_up, _) = platform_and_ball(std::f32::consts::PI, true, -3.0, 6.0, platform_first);
        assert!(
            max_up < 0.0,
            "platform_first={platform_first}: an upside-down one-way platform should be SOLID \
             from below, stopping a body launched up at it, but it reached y={max_up}"
        );
    }
}
