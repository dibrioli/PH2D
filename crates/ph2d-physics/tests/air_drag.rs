//! **Air drag knows how big a body is** — the gate for the smoke that failed.
//!
//! Enio, 2026-07-18: *"Air Drag… todos os objetos grandes e pequenos caem na
//! mesma velocidade"*. He was right, and the knob he was moving was rapier's
//! `linear_damping`: a uniform velocity decay, into which mass and size cannot
//! enter. Measured at the time — four boxes spanning a **25× mass range** all
//! fell at **4.8925 m/s**, identical to four decimals.
//!
//! These gates pin the model that replaced it, and the one property that makes
//! it worth having: **size changes the answer**.

use ph2d_physics::{BodyDefaults, BodyDesc, PhysicsWorld, RigidBodyType, ShapeDesc};

fn box_of(side: f32) -> BodyDesc {
    BodyDesc {
        body_type: RigidBodyType::Dynamic,
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        density: 1.0,
        shape: ShapeDesc::Cuboid {
            half_x: side * 0.5,
            half_y: side * 0.5,
        },
        restitution: 0.0,
        friction: 0.5,
    }
}

/// Terminal fall speed of one box of `side`, under air drag `k`.
fn terminal_speed(k: f32, side: f32) -> f32 {
    let mut w = PhysicsWorld::new();
    w.set_air_drag(k);
    let h = w.spawn_body(box_of(side));
    // Ten seconds — long enough that the exponential approach to terminal has
    // converged to well under the tolerances below.
    for _ in 0..600 {
        w.step();
    }
    w.bodies().get(h).expect("alive").linvel().norm()
}

/// **The gate that reproduces the smoke.** Bigger falls faster, strictly.
///
/// This is the whole point of the model, and the exact property the previous
/// knob could not have: with a uniform decay every one of these is the same
/// number.
///
/// Mutation that must bleed: dropping `length` from the force (making drag
/// size-independent again).
#[test]
fn a_bigger_body_falls_faster_through_the_same_air() {
    let sides = [0.28f32, 0.5, 1.0, 2.0];
    let speeds: Vec<f32> = sides.iter().map(|&s| terminal_speed(1.0, s)).collect();
    for w in speeds.windows(2) {
        assert!(
            w[1] > w[0] * 1.1,
            "terminal speeds must grow with size and they went {speeds:?} for \
             sides {sides:?} — this is the smoke Enio reported: every body \
             falling at the same speed means the drag is not seeing size"
        );
    }
}

/// **The terminal speed is the one the drag equation predicts.**
///
/// The oracle is the published closed form, not a number this code produced:
/// at terminal, `m·g = k·L·v²`, so `v = √(m·g/(k·L))`. For a uniform-density
/// box of side `s`: `m = s²`, `L = s`.
///
/// A bar of 2% is far tighter than any plausible wrong model — a mass-blind
/// drag would put every row on the same number, and dropping the `½` or using
/// linear instead of quadratic drag moves these by tens of percent.
#[test]
fn the_terminal_speed_matches_the_drag_equation() {
    const G: f32 = 9.81;
    for k in [0.5f32, 1.0, 2.0] {
        for side in [0.28f32, 0.5, 1.0, 2.0] {
            let mass = side * side; // density 1, 2D
            let predicted = (mass * G / (k * side)).sqrt();
            let measured = terminal_speed(k, side);
            let err = (measured - predicted).abs() / predicted;
            assert!(
                err < 0.02,
                "k={k}, side={side}: measured {measured:.4} m/s vs the drag \
                 equation's {predicted:.4} m/s ({:.1}% off)",
                err * 100.0
            );
        }
    }
}

/// **Zero drag is byte-identical to no drag at all.**
///
/// The default is zero, so this is what keeps every existing project — and the
/// cross-OS C9 hashes — exactly where they were. The early-out in `drag::apply`
/// is the mechanism; this is the gate on it.
///
/// Mutation that must bleed: removing the `if k <= 0.0 { return; }` guard (the
/// per-body impulse of `0.0` is not free — it wakes the arithmetic and the
/// `apply_impulse` path).
#[test]
fn zero_drag_leaves_the_simulation_bit_identical() {
    let mut without = PhysicsWorld::new();
    let mut with_zero = PhysicsWorld::new();
    with_zero.set_air_drag(0.0);

    let a = without.spawn_body(box_of(0.5));
    let b = with_zero.spawn_body(box_of(0.5));
    for tick in 0..240 {
        without.step();
        with_zero.step();
        let ya = without.body_pose(a).unwrap().translation.y;
        let yb = with_zero.body_pose(b).unwrap().translation.y;
        assert_eq!(
            ya.to_bits(),
            yb.to_bits(),
            "tick {tick}: air drag of ZERO moved the simulation ({ya} vs {yb}) — \
             every existing project and both C9 hashes just changed"
        );
    }
}

/// **Air drag and damping are different models, and both still work.**
///
/// The fix for the smoke could have been "replace damping with drag". It was
/// not: uniform damping is the right tool for top-down friction and rapier,
/// Godot and Unity all ship it. So the gate says the uniform one is still
/// uniform — if a later refactor folded them together, the knob that is
/// *supposed* to ignore size would quietly stop doing so.
#[test]
fn damping_stays_uniform_while_drag_does_not() {
    let uniform: Vec<f32> = [0.28f32, 2.0]
        .iter()
        .map(|&side| {
            let mut w = PhysicsWorld::new();
            w.set_body_defaults(BodyDefaults {
                linear_damping: 2.0,
                ..BodyDefaults::rapier()
            });
            let h = w.spawn_body(box_of(side));
            for _ in 0..600 {
                w.step();
            }
            w.bodies().get(h).unwrap().linvel().norm()
        })
        .collect();
    assert_eq!(
        uniform[0].to_bits(),
        uniform[1].to_bits(),
        "linear damping must be UNIFORM — a 0.28 m and a 2 m body settled at \
         {uniform:?}. If this ever differs, the two models have been folded \
         together and the honest labelling in the panel is now a lie"
    );
}
