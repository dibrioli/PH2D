//! Per-body gravity scale (ADR-0131 W8).
//!
//! `BodyDesc::gravity_scale` multiplies the world gravity for one body. These
//! gates drive the REAL `spawn_body` (so the value reaches the solver, not just
//! the descriptor) and read the pose back after stepping — a trajectory oracle,
//! because the failure mode is "the multiplier was ignored" and the trajectory
//! is what shows it.

use ph2d_physics::{BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

/// Drop four balls from the same height under the same gravity, differing ONLY
/// in `gravity_scale`, spaced far apart on x so they never touch. After half a
/// second the trajectories must say the multiplier bit:
///
/// - `1.0` (control) falls,
/// - `0.0` is weightless (does not move),
/// - `2.0` falls exactly twice as far (free-fall distance is `½·a·t²`, and
///   `a = scale·g`, so the ratio is the scale ratio, independent of `t`),
/// - `-1.0` floats UP by about the control's fall.
///
/// Mutation-tested: `spawn_body` applying `.gravity_scale(1.0)` instead of
/// `desc.gravity_scale` makes every ball the control — the weightless one falls
/// and the ratio collapses to 1, so both the weightless and the `2×` assertions
/// go RED.
#[test]
fn gravity_scale_multiplies_the_bodys_fall() {
    let start_y = 10.0_f32;
    let mut w = PhysicsWorld::new();

    let ball = |x: f32, scale: f32| BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x,
        y: start_y,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Ball { radius: 0.5 },
        restitution: 0.0,
        friction: 0.5,
        layer: 0,
        is_sensor: false,
        gravity_scale: scale,
        linvel: [0.0, 0.0],
        angvel: 0.0,
        ccd: false,
        lock_rotation: false,
        lock_x: false,
        lock_y: false,
        offset: [0.0, 0.0],
    };
    // Spaced 3 m apart (radius 0.5) so no two ever collide — this is free fall.
    let control = w.spawn_body(ball(0.0, 1.0));
    let weightless = w.spawn_body(ball(3.0, 0.0));
    let heavy = w.spawn_body(ball(6.0, 2.0));
    let floater = w.spawn_body(ball(9.0, -1.0));

    for _ in 0..30 {
        w.step(); // 0.5 s @ 60 Hz
    }
    let y = |h| w.body_pose(h).expect("body exists").translation.vector.y;

    let control_drop = start_y - y(control);
    assert!(
        control_drop > 1.0,
        "the control (scale 1.0) should have fallen; drop = {control_drop}"
    );

    let weightless_move = (y(weightless) - start_y).abs();
    assert!(
        weightless_move < 0.01,
        "scale 0.0 is weightless and must not move; it moved {weightless_move}"
    );

    let heavy_drop = start_y - y(heavy);
    let ratio = heavy_drop / control_drop;
    assert!(
        (1.95..=2.05).contains(&ratio),
        "scale 2.0 must fall ~2× the control ({heavy_drop} vs {control_drop}, ratio {ratio})"
    );

    let float_rise = y(floater) - start_y;
    assert!(
        float_rise > 0.0,
        "scale -1.0 must float UP, not fall; Δy = {float_rise}"
    );
    let float_ratio = float_rise / control_drop;
    assert!(
        (0.9..=1.1).contains(&float_ratio),
        "scale -1.0 should rise about as far as the control falls (rise {float_rise}, \
         drop {control_drop}, ratio {float_ratio})"
    );
}
