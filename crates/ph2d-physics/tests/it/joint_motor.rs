//! **The motor drives whatever the joint left free** (W-J6) — the hinge of a
//! Pin, the rail of a Slider, the distance of a Rope — and it can be told either
//! a RATE or a PLACE.
//!
//! Until this wave a motor existed only on the Pin and only in velocity mode.
//! The two halves are separable and both are gated here: `motor_axis` decides
//! WHICH kinds are driven (and the Spring's exclusion is mechanical — the last
//! two gates are about that), `MotorMode` decides WHAT the instruction is.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyType, ShapeDesc,
};

type Handle = ph2d_physics::RigidBodyHandle;

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    rotation: f32,
    shape: ShapeDesc,
) -> Handle {
    world.spawn_body(BodyDesc {
        body_type: kind,
        x,
        y,
        rotation,
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
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

/// Attach with WORLD anchors, converting once.
///
/// ⚠️ `JointDesc`'s anchors are body-**LOCAL** (the caller converts, exactly
/// once, and stores the pair). Handing it a world point puts the joint metres
/// from where the fixture reads — which is how the first servo sweep in this
/// wave came to measure a rig nobody had built.
fn join(w: &mut PhysicsWorld, a: Handle, b: Handle, desc: JointDesc) {
    let (la, lb) = w
        .world_to_local_anchors(a, b, desc.anchor_a, desc.anchor_b)
        .expect("anchors");
    w.spawn_joint(
        a,
        b,
        JointDesc {
            anchor_a: la,
            anchor_b: lb,
            ..desc
        },
    )
    .expect("joint");
}

/// A hook at `(0, 6)` and a 0.2 kg, 1 m arm hanging straight down from it — the
/// rig the `MOTOR_TRACKING` and `SERVO_*` tables are measured on. Not joined
/// here: what the joint IS is the thing each gate varies.
fn hook_and_arm(w: &mut PhysicsWorld) -> (Handle, Handle) {
    let hook = body(
        w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        0.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let arm = body(
        w,
        RigidBodyType::Dynamic,
        0.0,
        5.5,
        -std::f32::consts::FRAC_PI_2,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    (hook, arm)
}

/// A motor with everything but the mode and its one number left neutral.
fn motor(mode: MotorMode, speed: f32, target: f32) -> Option<MotorDesc> {
    Some(MotorDesc {
        mode,
        speed,
        target,
        max_force: 10.0,
    })
}

/// **A servo HOLDS its target against gravity; a rate motor cannot.**
///
/// The arm starts hanging straight down (−90°) and is told to be at **+45°**,
/// which gravity spends the whole run pulling it away from. A servo arrives and
/// stays. The control is the SAME rig with the motor asked for a rate of zero —
/// the closest thing velocity mode has to *"be somewhere"*, and it is not a
/// place, so the arm hangs.
///
/// ⚠️ The oracle is the trajectory's TAIL, not one sample: *arrives* and *stays*
/// are different claims, and a servo oscillating through 45° would satisfy the
/// first alone.
///
/// Mutation: the `Position` arm passing `0.0` for the stiffness (velocity's
/// value) — RED, the arm hangs.
#[test]
fn a_servo_holds_its_target_against_gravity_and_a_rate_motor_cannot() {
    let target = std::f32::consts::FRAC_PI_4;
    let run = |m: Option<MotorDesc>| -> Vec<f32> {
        let mut w = PhysicsWorld::new();
        let (hook, arm) = hook_and_arm(&mut w);
        join(
            &mut w,
            hook,
            arm,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 6.0],
                anchor_b: [0.0, 6.0],
                motor: m,
                ..JointDesc::default()
            },
        );
        let mut tail = Vec::new();
        for step in 0..300 {
            w.step();
            if step >= 240 {
                tail.push(w.body_pose(arm).unwrap().rotation.angle().to_degrees());
            }
        }
        tail
    };

    for a in run(motor(MotorMode::Position, 0.0, target)) {
        assert!(
            (a - 45.0).abs() < 1.0,
            "the servo was told to hold +45 deg and its tail reads {a:.2}"
        );
    }
    // The control. Without it "the arm ends at 45" would also be satisfied by a
    // rig where gravity never reached the arm in the first place.
    let hanging = run(motor(MotorMode::Velocity, 0.0, target));
    let last = *hanging.last().expect("tail");
    assert!(
        last < -80.0,
        "a rate of zero is not a place: the arm has to hang, got {last:.2}"
    );
}

/// **A Slider's rail is driven, and in METRES.**
///
/// The rail is vertical (`axis = +Y`), so gravity pulls the carriage down it and
/// a motor that works has to fight something. Velocity mode lifts it at a rate;
/// position mode parks it at a place and holds.
///
/// Mutation: `motor_axis` answering `None` for a Slider leaves the carriage at
/// the bottom in both modes — RED twice.
#[test]
fn a_sliders_rail_is_driven_in_metres() {
    let run = |m: Option<MotorDesc>, steps: u32| -> f32 {
        let mut w = PhysicsWorld::new();
        let post = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            3.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let car = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            3.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.2,
                half_y: 0.2,
            },
        );
        join(
            &mut w,
            post,
            car,
            JointDesc {
                kind: JointKind::Slider,
                anchor_a: [0.0, 3.0],
                anchor_b: [0.0, 3.0],
                // A VERTICAL rail: gravity acts along it, so a motor that does
                // nothing is visibly different from one that does.
                axis_a: [0.0, 1.0],
                axis_b: [0.0, 1.0],
                motor: m,
                ..JointDesc::default()
            },
        );
        for _ in 0..steps {
            w.step();
        }
        w.body_pose(car).unwrap().translation.y
    };

    // Unpowered, the carriage slides down the rail — the control.
    let free = run(None, 120);
    assert!(
        free < 2.0,
        "an unpowered carriage slides down, got {free:.3}"
    );
    // 0.5 m/s for 2 s from the anchor: about a metre up, and not falling.
    //
    // ⚠️ **Not exactly a metre, and the gap is a measured property rather than
    // slop.** A velocity motor is a damping term, so lifting against gravity it
    // settles `g / MOTOR_TRACKING` short — 0.0097 m/s at the shipped 1000, which
    // is why the band is centred a hair under 4.0 rather than on it. (At the old
    // tracking of 100 it was 0.098 m/s and the carriage landed at 3.80: that
    // measurement is what re-picked the constant.)
    let lifted = run(motor(MotorMode::Velocity, 0.5, 0.0), 120);
    assert!(
        (lifted - 3.98).abs() < 0.06,
        "0.5 m/s for 2 s should land just under y = 4.0, got {lifted:.3}"
    );
    // And a servo parks at a stated place, in metres from the anchor.
    let parked = run(motor(MotorMode::Position, 0.0, 0.75), 300);
    assert!(
        (parked - 3.75).abs() < 0.05,
        "a servo told 0.75 m up the rail should hold y = 3.75, got {parked:.3}"
    );
}

/// **A Rope's motor is a WINCH — it reels the load in, and it is linear.**
///
/// The load hangs 2 m under the hook on a 2 m rope (taut). A servo told 0.5 m
/// hauls it up to 0.5 m below the hook and holds it there against its weight.
///
/// ⚠️ This is the gate that makes `motor_in_metres` and `limits_in_metres` worth
/// being separate questions: a Rope has **no limit range at all** and still has a
/// linear motor, so one door for both would have given the winch a target in
/// degrees.
///
/// Mutation: `motor_axis` answering `None` for a Rope leaves the load at y = 4 —
/// RED.
#[test]
fn a_ropes_motor_is_a_winch_that_reels_the_load_in() {
    let run = |m: Option<MotorDesc>| -> f32 {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            6.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let load = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            4.0,
            0.0,
            ShapeDesc::Ball { radius: 0.15 },
        );
        join(
            &mut w,
            hook,
            load,
            JointDesc {
                kind: JointKind::Rope,
                anchor_a: [0.0, 6.0],
                anchor_b: [0.0, 4.0],
                max_length: 2.0,
                motor: m,
                ..JointDesc::default()
            },
        );
        for _ in 0..300 {
            w.step();
        }
        w.body_pose(load).unwrap().translation.y
    };

    // Unwinched, the load hangs at the rope's full length — the control.
    let slack = run(None);
    assert!(
        (slack - 4.0).abs() < 0.1,
        "a passive 2 m rope hangs its load at y = 4.0, got {slack:.3}"
    );
    let reeled = run(motor(MotorMode::Position, 0.0, 0.5));
    assert!(
        (reeled - 5.5).abs() < 0.15,
        "a winch told 0.5 m should hold the load at y = 5.5, got {reeled:.3}"
    );
    assert!(
        reeled > slack + 1.0,
        "the winch has to have LIFTED it: {slack:.3} -> {reeled:.3}"
    );
}

/// **A Spring never receives a motor, and the reason is mechanical.**
///
/// rapier models a spring *as* a motor on the coupled linear axis, so writing a
/// second one there would overwrite the stiffness and damping the artist
/// authored — the spring would stop being a spring and become a rate-driven rod,
/// with both knobs still on screen.
///
/// The oracle is byte-level: the SAME spring, with and without a `MotorDesc`
/// attached, has to settle at the identical pose. A weaker *"still bouncy"*
/// assertion would pass on a spring whose gains had merely been dented.
///
/// Mutation: `motor_axis` answering `Some(JointAxis::LinX)` for a Spring — RED.
#[test]
fn a_springs_motor_is_never_written_or_it_would_eat_the_spring() {
    let run = |m: Option<MotorDesc>| -> f32 {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            6.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let ball = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            5.0,
            0.0,
            ShapeDesc::Ball { radius: 0.15 },
        );
        join(
            &mut w,
            hook,
            ball,
            JointDesc {
                kind: JointKind::Spring,
                anchor_a: [0.0, 6.0],
                anchor_b: [0.0, 5.0],
                rest_length: 1.0,
                motor: m,
                ..JointDesc::default()
            },
        );
        for _ in 0..300 {
            w.step();
        }
        w.body_pose(ball).unwrap().translation.y
    };
    let plain = run(None);
    let with_motor = run(motor(MotorMode::Velocity, 3.0, 0.0));
    assert_eq!(
        plain.to_bits(),
        with_motor.to_bits(),
        "a spring handed a motor must be byte-identical to one without: \
         {plain} vs {with_motor}"
    );
    // The control: the spring is doing something at all, or the equality above
    // would be two copies of a joint that never moved.
    assert!(
        (plain - 5.0).abs() > 0.01,
        "the spring has to have moved the ball, got {plain:.4}"
    );
}

/// **A Weld ignores a motor too** — the sibling of the spring gate, for a
/// different reason: it has no free axis at all, so there is nothing to drive.
///
/// ⚠️ **Documented survivor.** The mutation that gives a Weld a motor axis
/// (`motor_axis` → `Some(LinX)`) leaves this GREEN, and that is honest rather
/// than a hole: rapier has all six of a fixed joint's axes locked, so a motor
/// written to one is inert by the solver's own construction. The property is
/// defended twice and only the outer layer is ours — the gate pins the property,
/// not our share of it ([[feedback_layered_defenses_need_per_layer_gates]]). Its
/// spring sibling above is the one that bleeds, because there the second layer
/// does not exist.
#[test]
fn a_weld_ignores_a_motor_because_it_has_no_free_axis() {
    let run = |m: Option<MotorDesc>| -> f32 {
        let mut w = PhysicsWorld::new();
        let post = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            5.0,
            0.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let bar = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.5,
            5.0,
            0.0,
            ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
        );
        join(
            &mut w,
            post,
            bar,
            JointDesc {
                kind: JointKind::Weld,
                anchor_a: [0.0, 5.0],
                anchor_b: [0.0, 5.0],
                motor: m,
                ..JointDesc::default()
            },
        );
        for _ in 0..180 {
            w.step();
        }
        w.body_pose(bar).unwrap().rotation.angle()
    };
    assert_eq!(
        run(None).to_bits(),
        run(motor(MotorMode::Velocity, 4.0, 0.0)).to_bits(),
        "a weld handed a motor must be byte-identical to one without"
    );
}
