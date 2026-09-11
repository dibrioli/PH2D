//! **A joint that gives way** (W-J7) — the reading, the threshold, the break.
//!
//! Three separable claims, gated separately because they fail for different
//! reasons: the number is a **force** (and stays one when the solver's knobs
//! move), the threshold is **honoured** (and `∞` never is), and the action is
//! **disable, not delete** (the joint survives its own break, and a Reset brings
//! it back).

use ph2d_physics::{BodyDesc, JointDesc, JointKind, PhysicsWorld, RigidBodyType, ShapeDesc};

type Handle = ph2d_physics::RigidBodyHandle;
type JointHandle = ph2d_physics::ImpulseJointHandle;

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
    mass: Option<f32>,
) -> Handle {
    world.spawn_body(BodyDesc {
        body_type: kind,
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
        lock_x: false,
        lock_y: false,
        mass_override: mass,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

/// A hook at the origin with `mass` kg hanging 1 m below it on a joint of
/// `kind`, carrying the two thresholds. Returns the world, the joint and the
/// load body.
///
/// ⚠️ The anchors are converted ONCE, here — `JointDesc` takes body-LOCAL points
/// and handing it world ones puts the joint metres from where the fixture reads
/// (the trap the servo sweep of W-J6 fell into).
fn hanging(
    kind: JointKind,
    mass: f32,
    break_force: f32,
    break_torque: f32,
) -> (PhysicsWorld, JointHandle, Handle) {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.05 },
        None,
    );
    let load = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        -1.0,
        ShapeDesc::Ball { radius: 0.2 },
        Some(mass),
    );
    let (la, lb) = w
        .world_to_local_anchors(hook, load, [0.0, 0.0], [0.0, -1.0])
        .expect("anchors");
    let h = w
        .spawn_joint(
            hook,
            load,
            JointDesc {
                kind,
                anchor_a: la,
                anchor_b: lb,
                max_length: 1.0,
                break_force,
                break_torque,
                ..JointDesc::default()
            },
        )
        .expect("joint");
    (w, h, load)
}

fn settle(w: &mut PhysicsWorld, ticks: usize) {
    for _ in 0..ticks {
        w.step();
    }
}

/// **A joint reads the load it is holding, in newtons.**
///
/// The calibration, and the number is not approximate: a hanging weight reads
/// `m·g` exactly, at every mass.
///
/// Mutation: `joint_impulse_to_force` dividing by `substep_dt` alone (the first
/// version) — every row reads exactly a QUARTER of its weight, because rapier's
/// island solver splits each sub-step again into `num_solver_iterations` small
/// steps and the writeback reports one of those.
#[test]
fn a_joint_reads_the_load_it_is_holding_in_newtons() {
    for mass in [0.5_f32, 1.0, 2.0, 5.0, 10.0] {
        let (mut w, h, _) = hanging(JointKind::Pin, mass, f32::INFINITY, f32::INFINITY);
        settle(&mut w, 120);
        let got = w.joint_load(h).expect("load").force;
        let want = mass * 9.81;
        assert!(
            (got - want).abs() < 0.01,
            "{mass} kg hanging has to read its own weight {want:.4} N, read {got:.4}"
        );
    }
}

/// **It is a FORCE, so neither of the solver's two divisors moves it.**
///
/// The whole reason the threshold is written in newtons rather than in
/// newton-seconds: an impulse threshold would halve every time the sub-step count
/// doubled, and a chain the artist tuned to break would quietly stop breaking.
/// Both knobs are the artist's (`set_substeps`, `set_solver_iterations`), which is
/// why both are turned here.
///
/// Mutation: dropping `num_solver_iterations` from the conversion — the iteration
/// column goes 2.45 / 4.90 / 9.81 / 19.62 / 39.24 instead of staying at 9.81.
#[test]
fn the_reading_is_a_force_so_it_survives_both_of_the_solvers_divisors() {
    for subs in [1_u32, 2, 4, 8, 16] {
        let (mut w, h, _) = hanging(JointKind::Pin, 1.0, f32::INFINITY, f32::INFINITY);
        w.set_substeps(subs);
        settle(&mut w, 120);
        let got = w.joint_load(h).expect("load").force;
        assert!(
            (got - 9.81).abs() < 0.02,
            "{subs} sub-steps: still 9.81 N, read {got:.4}"
        );
    }
    for iters in [1_usize, 2, 4, 8, 16] {
        let (mut w, h, _) = hanging(JointKind::Pin, 1.0, f32::INFINITY, f32::INFINITY);
        w.set_solver_iterations(iters);
        settle(&mut w, 120);
        let got = w.joint_load(h).expect("load").force;
        assert!(
            (got - 9.81).abs() < 0.02,
            "{iters} solver iterations: still 9.81 N, read {got:.4}"
        );
    }
}

/// **An overloaded joint gives way; an unbreakable one never does.**
///
/// The wave's whole claim, with its own control in the same gate — "the load
/// fell" is satisfied just as well by a joint that was never built, so the same
/// rig with `∞` has to keep holding.
///
/// Mutation: `accumulate_and_break` never calling `set_enabled(false)` — the
/// heavy load hangs at −1.00 m forever and the first half goes red.
#[test]
fn an_overloaded_joint_gives_way_and_an_unbreakable_one_never_does() {
    // 10 kg on a rope rated for 50 N. It carries 98.1 N, so it parts.
    let (mut w, h, load) = hanging(JointKind::Rope, 10.0, 50.0, f32::INFINITY);
    settle(&mut w, 120);
    assert_eq!(
        w.joint_is_enabled(h),
        Some(false),
        "98.1 N on a 50 N rope has to part it"
    );
    let y = w.body_pose(load).expect("pose").translation.y;
    assert!(
        y < -3.0,
        "a parted rope stops holding: the load has to keep falling, got y = {y:.2}"
    );

    // The control: the same 10 kg, unbreakable.
    let (mut w, h, load) = hanging(JointKind::Rope, 10.0, f32::INFINITY, f32::INFINITY);
    settle(&mut w, 120);
    assert_eq!(w.joint_is_enabled(h), Some(true), "infinity never breaks");
    let y = w.body_pose(load).expect("pose").translation.y;
    assert!(
        (y + 1.0).abs() < 0.05,
        "an unbreakable rope holds its load at -1.00 m, got {y:.2}"
    );
}

/// **A break DISABLES the joint — it does not delete it.**
///
/// The joint stays in the world with its anchors and its parameters, which is
/// what makes a Reset bring it back (nothing about the break is authored) and
/// what keeps a break from destroying the artist's work.
#[test]
fn a_break_disables_the_joint_it_does_not_delete_it() {
    let (mut w, h, _) = hanging(JointKind::Rope, 10.0, 50.0, f32::INFINITY);
    settle(&mut w, 120);
    assert_eq!(w.joint_count(), 1, "the joint is still there");
    assert_eq!(w.joint_is_enabled(h), Some(false));
    assert!(
        w.joint_anchors(h).is_some(),
        "and it still knows where it attaches"
    );
    assert_eq!(
        w.joint_break_thresholds(h),
        Some((50.0, f32::INFINITY)),
        "and what it was rated for"
    );
}

/// **The break is an EVENT, with a place and the load it gave at.**
///
/// Reported once, in the tick it happens — a moment later the joint reads a load
/// of zero, because it is not holding anything, so the state that follows cannot
/// carry this.
#[test]
fn the_break_is_an_event_carrying_where_and_how_hard() {
    let (mut w, h, _) = hanging(JointKind::Rope, 10.0, 50.0, f32::INFINITY);
    let mut seen = None;
    for _ in 0..120 {
        w.step();
        if let Some(b) = w.joint_breaks().first() {
            seen = Some(*b);
            break;
        }
    }
    let b = seen.expect("the rope has to report its own break");
    assert_eq!(b.joint, h);
    assert!(
        b.force > 50.0,
        "the event carries the load it gave at, and it is above the threshold: {:.1} N",
        b.force
    );
    // The rope hangs straight down from the origin, so the break is somewhere on
    // that line, about a metre below the hook.
    assert!(
        b.point[0].abs() < 0.2 && b.point[1] < 0.0,
        "the break happened on the rope, at {:?}",
        b.point
    );
    // And it is reported ONCE: the next tick's list is empty (the joint is gone
    // from the comparison entirely, being disabled).
    w.step();
    assert!(
        w.joint_breaks().is_empty(),
        "a break is a transition, not a state that keeps re-reporting"
    );
}

/// **A TORQUE threshold fires on a hinge at its stop** — the one place a torque
/// is both produced and observable.
///
/// MEASURED (`measure_joint_break.rs`): a hinge at its limit and a servo both
/// report `m·g·r` exactly, and a Weld's locked angular axis reports `0.0000`
/// while holding the same moment. The sibling half of that fact — that the panel
/// therefore offers the row on a Pin alone — is gated in the Inspector.
#[test]
fn a_torque_threshold_fires_on_a_hinge_at_its_stop() {
    // A 1 kg, 1 m plank pinned at its left end, with the hinge pinched shut: it
    // has to hold `m·g·0.5` = 4.905 N·m, which a 2 N·m hinge cannot.
    for (rating, should_break) in [(2.0_f32, true), (f32::INFINITY, false)] {
        let mut w = PhysicsWorld::new();
        let hub = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
            None,
        );
        let plank = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.5,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.05,
            },
            Some(1.0),
        );
        let (la, lb) = w
            .world_to_local_anchors(hub, plank, [0.0, 0.0], [0.0, 0.0])
            .expect("anchors");
        let h = w
            .spawn_joint(
                hub,
                plank,
                JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: la,
                    anchor_b: lb,
                    limits: Some([-0.001, 0.001]),
                    // The FORCE threshold is out of reach on purpose (the plank's
                    // weight is 9.81 N), so only the torque can be what fired.
                    break_force: f32::INFINITY,
                    break_torque: rating,
                    ..JointDesc::default()
                },
            )
            .expect("joint");
        settle(&mut w, 120);
        assert_eq!(
            w.joint_is_enabled(h),
            Some(!should_break),
            "a hinge rated {rating} N.m holding 4.905"
        );
    }
}

/// **The thresholds ride INSIDE the joint, so a checkpoint carries them.**
///
/// They live in rapier's `user_data`, which the checkpoint ring deep-clones with
/// the rest of `ImpulseJointSet`. A side table would need its own capture, and
/// the failure of forgetting is a joint that scrubs back to being unbreakable.
///
/// Mutation: `spawn_joint` not writing `user_data` — the joint reads `(∞, ∞)`
/// and never breaks, so this and the overload gate both go red.
#[test]
fn the_thresholds_ride_inside_the_joint_and_survive_a_checkpoint() {
    let (mut w, h, _) = hanging(JointKind::Rope, 1.0, 50.0, 20.0);
    settle(&mut w, 30);
    let cp = w.checkpoint();
    assert_eq!(w.joint_break_thresholds(h), Some((50.0, 20.0)));
    // Wind on, then restore: the thresholds have to come back with the world.
    settle(&mut w, 30);
    w.restore(&cp);
    assert_eq!(
        w.joint_break_thresholds(h),
        Some((50.0, 20.0)),
        "a restored world remembers what its joints were rated for"
    );
}

/// **A joint that was never told a threshold is unbreakable.**
///
/// Every scene that predates W-J7 is exactly this: `JointDesc::default()` carries
/// `∞` on both, so nothing is ever compared and nothing is ever disabled — which
/// is what keeps those scenes byte-identical. Stated as the artist would see it
/// (a 50 kg load on a plain joint still hangs) and as the data would (the
/// thresholds the joint reports back), because the first alone would also pass on
/// a build where the whole comparison was deleted.
///
/// Mutation: `to_slot` mapping `∞` to something finite — the default joint parts
/// under its own load and both halves go red.
#[test]
fn a_joint_that_was_never_told_a_threshold_is_unbreakable() {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        0.0,
        ShapeDesc::Ball { radius: 0.05 },
        None,
    );
    let load = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        -1.0,
        ShapeDesc::Ball { radius: 0.2 },
        Some(50.0),
    );
    let (la, lb) = w
        .world_to_local_anchors(hook, load, [0.0, 0.0], [0.0, -1.0])
        .expect("anchors");
    // ⚠️ The break fields are NOT mentioned — that is the whole point of the gate.
    // A ROPE and not a Pin, so the load hangs a metre down where it can be SEEN
    // to still be held: a pin's two anchors are the same point, so its load ends
    // up AT the hook and "still holding" and "fell to the hook" look alike.
    let h = w
        .spawn_joint(
            hook,
            load,
            JointDesc {
                kind: JointKind::Rope,
                anchor_a: la,
                anchor_b: lb,
                ..JointDesc::default()
            },
        )
        .expect("joint");
    assert_eq!(
        w.joint_break_thresholds(h),
        Some((f32::INFINITY, f32::INFINITY)),
        "a joint nobody rated takes anything"
    );
    settle(&mut w, 120);
    assert_eq!(w.joint_is_enabled(h), Some(true));
    let y = w.body_pose(load).expect("pose").translation.y;
    assert!(
        (y + 1.0).abs() < 0.05,
        "490 N on an unrated joint still hangs at -1.00 m, got {y:.2}"
    );
}
