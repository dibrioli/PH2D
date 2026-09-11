//! **W3 — joints at the engine seam.**
//!
//! Every gate here is stated as something the artist can watch happen, and
//! every one carries its own control: "the bodies are held together" is
//! satisfied just as well by "nothing moved at all", so each claim about a
//! constraint is paired with a claim about the motion it must still allow.

use ph2d_physics::{
    BodyDesc, JointDesc, JointKind, MotorDesc, MotorMode, PhysicsWorld, RigidBodyType, ShapeDesc,
};

/// Distance between two points.
fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

fn body(
    world: &mut PhysicsWorld,
    kind: RigidBodyType,
    x: f32,
    y: f32,
    shape: ShapeDesc,
) -> ph2d_physics::RigidBodyHandle {
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
        mass_override: None,
        dominance: 0,
        material: Default::default(),
        damping: None,
        one_way: false,
        effector: None,
        offset: [0.0, 0.0],
    })
}

fn pose(world: &PhysicsWorld, h: ph2d_physics::RigidBodyHandle) -> [f32; 2] {
    let p = world.body_pose(h).expect("body alive");
    [p.translation.x, p.translation.y]
}

/// Attach `a` to `b`, with the anchors given in **WORLD** coordinates — the
/// frame an artist points in.
///
/// The engine takes LOCAL anchors (so that a rebuild reproduces the identical
/// constraint), and the conversion is `world_to_local_anchors`. Every fixture
/// below goes through this helper rather than converting inline, because that
/// is exactly what the product does: one conversion, at authoring time.
fn join(
    w: &mut PhysicsWorld,
    a: ph2d_physics::RigidBodyHandle,
    b: ph2d_physics::RigidBodyHandle,
    desc: JointDesc,
) -> Option<ph2d_physics::ImpulseJointHandle> {
    let (la, lb) = w.world_to_local_anchors(a, b, desc.anchor_a, desc.anchor_b)?;
    w.spawn_joint(
        a,
        b,
        JointDesc {
            anchor_a: la,
            anchor_b: lb,
            ..desc
        },
    )
}

/// **The pendulum, and the anchor is where the ARTIST put it.**
///
/// A hook (static) sits at `(0, 6)`. A plank 1 m long lies to the right with
/// its centre at `(0.5, 5)`, so its **left end** is at `(0, 5)`. The pin goes
/// at `(0, 5)` — the plank's end, a metre below the hook and half a metre from
/// the plank's centre. Neither body's centre is at the anchor, which is the
/// whole point of the fixture.
///
/// What must happen: the plank hangs from that point and swings. What must NOT
/// happen: the plank's *centre* ending up glued under the hook — which is what
/// a version that quietly used body centres for the anchors would produce, and
/// it is a difference of a whole metre.
#[test]
fn a_pin_holds_the_point_the_artist_chose_not_the_bodies_centres() {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let plank = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.5,
        5.0,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    let j = join(
        &mut w,
        hook,
        plank,
        JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, 5.0],
            anchor_b: [0.0, 5.0],
            ..JointDesc::default()
        },
    )
    .expect("two distinct live bodies");

    // ⚠️ **Every assertion here is over the TRAJECTORY, never the final
    // frame.** An undamped pendulum is periodic: it comes back to where it
    // started, so sampling one instant proves nothing. The first version of
    // this gate asserted "the plank ends up below the pin" and failed on a
    // pendulum that was working perfectly — it was caught near the top of its
    // swing. (The scrub gate of W1.5 paid for this same lesson.)
    let mut lowest = f32::MAX;
    for _ in 0..240 {
        w.step();

        // Rigidity is an INVARIANT, so it is checked every step rather than
        // once at the end — a joint that comes apart mid-swing and drifts back
        // would pass an endpoint check.
        let (a1, a2) = w.joint_anchors(j).expect("joint alive");
        assert!(
            dist(a1, a2) < 0.01,
            "the pin came apart: anchor1 {a1:?} vs anchor2 {a2:?}"
        );
        assert!(
            dist(a1, [0.0, 5.0]) < 0.01,
            "the pin moved off the authored anchor (0, 5): it is at {a1:?}"
        );
        let c = pose(&w, plank);
        let from_pin = dist(c, [0.0, 5.0]);
        assert!(
            (from_pin - 0.5).abs() < 0.02,
            "the plank's centre must stay 0.50 m from the pin (it is a rigid \
             plank pinned at its end); it is {from_pin:.3} m away, at {c:?}"
        );
        lowest = lowest.min(c[1]);
    }

    // Somewhere in that swing it hung straight down: 0.5 m below the pin.
    assert!(
        lowest < 4.6,
        "the plank never swung down — the lowest its centre ever got was \
         y={lowest:.3}, against a pin at y=5.0. A pendulum that does not fall \
         is not pinned, it is frozen"
    );
}

/// **The control for the gate above: the pin still lets the plank ROTATE.**
///
/// A constraint that welds everything solid would satisfy "the anchors
/// coincide" perfectly. The plank starts horizontal and must not still be
/// horizontal once gravity has had a go at it.
#[test]
fn a_pin_still_lets_the_bodies_rotate() {
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let plank = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.5,
        5.0,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    join(
        &mut w,
        hook,
        plank,
        JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, 5.0],
            anchor_b: [0.0, 5.0],
            ..JointDesc::default()
        },
    )
    .expect("joint");
    // The RANGE over the swing, not the endpoint — see the sibling gate: this
    // pendulum returns to where it started, so `end - start` is near zero on a
    // perfectly good hinge.
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for _ in 0..240 {
        w.step();
        let a = w.body_pose(plank).unwrap().rotation.angle();
        lo = lo.min(a);
        hi = hi.max(a);
    }
    assert!(
        hi - lo > 1.0,
        "the plank only ever turned through {:.3} rad ({lo:.3} .. {hi:.3}). A \
         hinge that does not hinge is a weld",
        hi - lo
    );
}

/// **Limits stop the swing, and the number they stop at is the one authored.**
///
/// Same pendulum, but the hinge may only open 30°. It must come to rest near
/// that angle — not at the bottom (no limit) and not at zero (frozen).
#[test]
fn a_limited_pin_stops_at_the_authored_angle() {
    let limit = 30f32.to_radians();
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let plank = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.5,
        5.0,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    join(
        &mut w,
        hook,
        plank,
        JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, 5.0],
            anchor_b: [0.0, 5.0],
            limits: Some([-limit, limit]),
            ..JointDesc::default()
        },
    )
    .expect("joint");
    for _ in 0..600 {
        w.step();
    }
    let angle = w.body_pose(plank).unwrap().rotation.angle();
    assert!(
        angle < -limit + 0.05 && angle > -limit - 0.05,
        "the plank should hang against its -30° stop; it settled at {:.1}°",
        angle.to_degrees()
    );
}

/// **A motor drives the hinge to the speed it was given.**
///
/// The fixture is a **wheel** — a plank pinned at its own centre — and that is
/// a measurement decision, not a convenience. Pinned at its *end* the plank is
/// a driven pendulum: gravity alternately helps and fights the motor and the
/// speed is genuinely chaotic (measured: the same input landing on -8.5, +9.1
/// and +2.0 rad/s across neighbouring parameters). Pinned at its centre gravity
/// exerts no torque about the pivot, so "did the motor reach its speed?" has
/// one answer.
#[test]
fn a_motor_reaches_the_speed_it_was_given() {
    let target = 4.0; // rad/s
    let mut w = PhysicsWorld::new();
    let hub = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        6.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let disc = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        6.0,
        ShapeDesc::Cuboid {
            half_x: 0.5,
            half_y: 0.1,
        },
    );
    join(
        &mut w,
        hub,
        disc,
        JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, 6.0],
            anchor_b: [0.0, 6.0],
            motor: Some(MotorDesc {
                mode: MotorMode::Velocity,
                speed: target,
                target: 0.0,
                max_force: 100.0,
            }),
            ..JointDesc::default()
        },
    )
    .expect("joint");
    for _ in 0..300 {
        w.step();
    }
    let spin = w.bodies().get(disc).unwrap().angvel();
    assert!(
        (spin - target).abs() < 0.1,
        "the motor was told {target} rad/s and is turning at {spin:.4}"
    );
}

/// **Jointed bodies do not collide with each other.**
///
/// The gate above cannot pass without this, and that is why it exists
/// separately: rapier's default is `contacts_enabled: true`, and the wheel's
/// hub sits *inside* the plank it is pinned to. With contacts on, the solver
/// spends every step shoving two permanently overlapping bodies apart, and the
/// motor told to turn at 4 rad/s was measured at **-80**.
///
/// Box2D (`collideConnected`) and Unity (`enableCollision`) both default to
/// off for the canonical reason: the links of a chain overlap at their pins.
#[test]
fn bodies_a_joint_connects_do_not_collide_with_each_other() {
    let mut w = PhysicsWorld::new();
    // Two balls at the SAME place — fully overlapping. Nothing but a disabled
    // contact keeps them from exploding apart.
    let a = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        5.0,
        ShapeDesc::Ball { radius: 0.5 },
    );
    let b = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        5.0,
        ShapeDesc::Ball { radius: 0.5 },
    );
    join(
        &mut w,
        a,
        b,
        JointDesc {
            kind: JointKind::Pin,
            anchor_a: [0.0, 5.0],
            anchor_b: [0.0, 5.0],
            ..JointDesc::default()
        },
    )
    .expect("joint");
    for _ in 0..120 {
        w.step();
    }
    let p = pose(&w, b);
    assert!(
        dist(p, [0.0, 5.0]) < 0.01,
        "two fully-overlapping bodies pinned together were shoved apart to \
         {p:?} — the contact between jointed bodies is still enabled, and a \
         chain of overlapping links would tear itself apart"
    );
}

/// **A weak motor stalls instead of winning.**
///
/// The control for the speed gate, and the reason `max_force` exists: an arm
/// hanging straight down has to be *lifted* past horizontal to keep turning. A
/// motor capped at 0.1 N·m cannot, and must not.
///
/// The oracle is radians **travelled** (unwrapped), because an arm that swings
/// up and falls back has an impressive peak angle and has gone nowhere.
#[test]
fn a_motor_that_is_too_weak_cannot_lift_its_own_arm() {
    fn travelled(max_force: f32) -> f32 {
        let mut w = PhysicsWorld::new();
        let hook = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            6.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let arm = w.spawn_body(BodyDesc {
            body_type: RigidBodyType::Dynamic,
            x: 0.0,
            y: 5.5,
            rotation: std::f32::consts::FRAC_PI_2, // hanging straight down
            density: 1.0,
            shape: ShapeDesc::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
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
        });
        join(
            &mut w,
            hook,
            arm,
            JointDesc {
                kind: JointKind::Pin,
                anchor_a: [0.0, 6.0],
                anchor_b: [0.0, 6.0],
                motor: Some(MotorDesc {
                    mode: MotorMode::Velocity,
                    speed: 4.0,
                    target: 0.0,
                    max_force,
                }),
                ..JointDesc::default()
            },
        )
        .expect("joint");
        let mut total = 0.0f32;
        let mut prev = w.body_pose(arm).unwrap().rotation.angle();
        for _ in 0..300 {
            w.step();
            let a = w.body_pose(arm).unwrap().rotation.angle();
            let mut d = a - prev;
            while d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            }
            while d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            // ⚠️ **SOMA COM SINAL, não `d.abs()`** — ver o bloco abaixo do `travelled`.
            total += d;
            prev = a;
        }
        total.abs()
    }
    // 4 rad/s for 5 s is 20 rad if the motor keeps its word.
    //
    // ⛔⛔ **A régua deste gate somava `d.abs()`, e isso é CAMINHO PERCORRIDO — enquanto a
    // afirmação dele («não consegue levantar o braço para lá da horizontal») é sobre POSIÇÃO.**
    // Um braço parado a tremer acumula caminho sem sair do sítio, e foi assim que o gate passou
    // durante anos: com o `sleep_linear_threshold` em `0,4` o braço **adormecia**, o tremor parava
    // de somar, e o número ficava pequeno **por o corpo ter congelado, não por o motor ser fraco**.
    //
    // Medido em 2026-08-29, ao baixar o limiar de sono para o valor MEDIDO (`0,05`, ver
    // `BodyDefaults::ours`), com o braço agora acordado durante os 5 s:
    //
    // | motor | caminho `Σ|d|` | rotação líquida `|Σd|` |
    // |---|---|---|
    // | `100 N·m` | `19,97 rad` | `19,97` (dá ~3 voltas) |
    // | ⛔ `0,1 N·m` | **`1,2288 rad`** | ⭐ **`0,0016 rad` = `0,1°`** |
    //
    // ⇒ **O tecto de força SEMPRE foi respeitado** — o braço não se mexe `0,1°`. O que estava
    // errado era a régua. ⭐ *Um gate que passa porque o corpo adormeceu está a medir o sono, não
    // a lei que diz medir.*
    //
    // A soma **com sinal** responde às duas perguntas com uma régua só: o tremor cancela-se
    // (é simétrico), e uma volta completa **não** volta a zero, porque `d` já vem desembrulhado.
    // ⚠️ `Σ|d|` não servia para o fraco e `Σd` embrulhado não serviria para o forte.
    let strong = travelled(100.0);
    let weak = travelled(0.1);
    assert!(
        strong > 18.0,
        "a motor with force to spare should have turned ~20 rad; it managed {strong:.2}"
    );
    // Barra: `0,1 rad` = `5,7°`. É **60×** a medição (`0,0016`) e **15×** mais apertada que a
    // afirmação que defende (levantar «para lá da horizontal» são `1,57 rad` a partir de pendurado)
    // — ⛔ e a mutação que ela apanha é a de sempre: um tecto de força ignorado põe o braço a dar
    // voltas, e `|Σd|` passa de `0,0016` para `~20`.
    assert!(
        weak < 0.1,
        "a motor capped at 0.1 N·m must not be able to lift a 0.2 kg arm past \
         horizontal, but it turned {weak:.4} rad NET — the force ceiling is \
         decorative and every motor is secretly infinite"
    );
}

/// **A rope is a ceiling on distance, not a rod.**
///
/// Both halves in one fixture, because either alone is trivially satisfiable:
/// the ball must fall FREELY while the rope is slack (a rod would hold it at
/// its start), and must then STOP at the rope's length (no length at all would
/// let it keep going).
#[test]
fn a_rope_lets_the_ball_fall_until_it_runs_out() {
    let max = 2.0;
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        10.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let ball = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        9.5,
        ShapeDesc::Ball { radius: 0.25 },
    );
    join(
        &mut w,
        hook,
        ball,
        JointDesc {
            kind: JointKind::Rope,
            anchor_a: [0.0, 10.0],
            anchor_b: [0.0, 9.5],
            max_length: max,
            ..JointDesc::default()
        },
    )
    .expect("joint");

    // Half a metre down, the rope is still slack: the ball is in free fall.
    for _ in 0..12 {
        w.step();
    }
    let early = pose(&w, ball);
    assert!(
        early[1] < 9.5,
        "while the rope is slack the ball must fall; it is at {early:?}"
    );

    for _ in 0..600 {
        w.step();
    }
    let hang = dist(pose(&w, ball), [0.0, 10.0]);
    assert!(
        hang <= max + 0.05,
        "the ball fell past the end of a {max} m rope: it is {hang:.3} m away"
    );
    assert!(
        hang > max - 0.2,
        "the ball is hanging {hang:.3} m from a {max} m rope — that is a short \
         rod, not a rope run out"
    );
}

/// **A spring stretches under load and springs back.**
///
/// This is also where [`JointDesc::DEFAULT_STIFFNESS`] and `DEFAULT_DAMPING`
/// earn their values: at the defaults a hanging 1 kg body must sag *visibly*
/// past the rest length (a spring that does not sag reads as a rod) and must
/// still be **under-damped** — it overshoots and comes back, which is the thing
/// an artist reaches for a spring to see.
#[test]
fn a_spring_sags_under_gravity_and_overshoots_on_the_way() {
    let rest = 1.0;
    let mut w = PhysicsWorld::new();
    let hook = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        10.0,
        ShapeDesc::Ball { radius: 0.05 },
    );
    let ball = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        9.0,
        ShapeDesc::Ball { radius: 0.25 },
    );
    join(
        &mut w,
        hook,
        ball,
        JointDesc {
            kind: JointKind::Spring,
            anchor_a: [0.0, 10.0],
            anchor_b: [0.0, 9.0],
            rest_length: rest,
            ..JointDesc::default()
        },
    )
    .expect("joint");

    let mut lowest = f32::MAX;
    let mut after_lowest_rise = 0.0f32;
    for _ in 0..600 {
        w.step();
        let y = pose(&w, ball)[1];
        if y < lowest {
            lowest = y;
        }
        after_lowest_rise = after_lowest_rise.max(y - lowest);
    }
    let settled = dist(pose(&w, ball), [0.0, 10.0]);
    assert!(
        settled > rest + 0.01,
        "a 1 kg body hanging on the default spring settled at {settled:.3} m \
         against a rest length of {rest} — it is not stretching, so the \
         default stiffness reads as a rod"
    );
    assert!(
        after_lowest_rise > 0.01,
        "the ball sank to its lowest point and never came back up ({after_lowest_rise:.4} m \
         of rebound) — the default damping is not under-damped, and a spring \
         that cannot bounce is a rod with extra steps"
    );
}

/// **A body cannot be joined to itself.**
///
/// Refused, not clamped: the constraint would be trivially satisfied forever,
/// so it would look like a joint the artist could tune and be a no-op.
#[test]
fn a_body_cannot_be_pinned_to_itself() {
    let mut w = PhysicsWorld::new();
    let b = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        5.0,
        ShapeDesc::Ball { radius: 0.25 },
    );
    assert!(w.spawn_joint(b, b, JointDesc::default()).is_none());
    assert_eq!(w.joint_count(), 0, "a refused joint must not be inserted");
}

/// **A jointed world is deterministic** — the same scene twice, bit for bit.
///
/// The cross-OS half runs in CI (`physics-ecs-c9`); this is the same-process
/// sanity check that joint insertion has not introduced an unordered container
/// or a hashed iteration into the sim (HR-5).
#[test]
fn a_jointed_world_hashes_the_same_twice() {
    fn chain() -> [u8; 32] {
        let mut w = PhysicsWorld::new();
        let mut prev = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            10.0,
            ShapeDesc::Ball { radius: 0.05 },
        );
        let mut y = 10.0;
        for _ in 0..8 {
            y -= 0.5;
            let link = body(
                &mut w,
                RigidBodyType::Dynamic,
                0.0,
                y,
                ShapeDesc::Ball { radius: 0.2 },
            );
            join(
                &mut w,
                prev,
                link,
                JointDesc {
                    kind: JointKind::Pin,
                    anchor_a: [0.0, y + 0.25],
                    anchor_b: [0.0, y + 0.25],
                    ..JointDesc::default()
                },
            )
            .expect("joint");
            prev = link;
        }
        for _ in 0..120 {
            w.step();
        }
        w.deterministic_hash()
    }
    assert_eq!(chain(), chain());
}

/// **The Slider slides along the axis it was GIVEN — a diagonal one** (W-J5).
///
/// A carriage sits on a static rail body. The axis is 45°, and gravity is
/// straight down, so the only way the carriage can reach a diagonal position is
/// for the constraint to be aiming there: on a default `+X` axis it would be
/// held at the same height and slide sideways (or not at all), and on a free
/// body it would fall straight down.
///
/// ⚠️ The oracle is the RATIO `dy/dx`, not the distance: how *far* it gets in a
/// second is a fact about gravity's component along the rail, but the DIRECTION
/// is the axis, and the direction is what the joint is for.
///
/// Mutation: drop `.local_axis1/.local_axis2` (so `PrismaticJointBuilder::new`'s
/// single axis stays) → still diagonal (`new` takes the same vector), so the real
/// mutation is passing `[1, 0]`: the carriage then travels `dy/dx = 0.000` and
/// this goes red.
#[test]
fn a_slider_travels_along_the_axis_it_was_given() {
    let mut w = PhysicsWorld::new();
    let rail = body(
        &mut w,
        RigidBodyType::Fixed,
        0.0,
        5.0,
        ShapeDesc::Ball { radius: 0.1 },
    );
    let car = body(
        &mut w,
        RigidBodyType::Dynamic,
        0.0,
        5.0,
        ShapeDesc::Ball { radius: 0.2 },
    );
    // 45° down-and-to-the-right: gravity has a component along it, so the
    // carriage runs DOWN the rail on its own.
    let d = std::f32::consts::FRAC_1_SQRT_2;
    join(
        &mut w,
        rail,
        car,
        JointDesc {
            kind: JointKind::Slider,
            anchor_a: [0.0, 5.0],
            anchor_b: [0.0, 5.0],
            axis_a: [d, -d],
            axis_b: [d, -d],
            ..Default::default()
        },
    )
    .expect("the slider builds");
    for _ in 0..60 {
        w.step();
    }
    let p = pose(&w, car);
    let (dx, dy) = (p[0] - 0.0, p[1] - 5.0);
    assert!(
        dx > 0.05,
        "the carriage has to actually travel along the rail, got dx {dx}"
    );
    assert!(
        (dy / dx + 1.0).abs() < 0.05,
        "it must travel ALONG the 45 degree axis (dy/dx = -1), got dy {dy} dx {dx} \
         ratio {}",
        dy / dx
    );
}

/// **And it STOPS at the stroke limits** — the range is in metres.
///
/// Same rail, axis straight down, so gravity drives the carriage the whole way.
/// With a `[-0.5, 0.5]` stroke it may fall exactly half a metre and no further;
/// unlimited it keeps going. The unlimited case is the control — without it a
/// carriage that never moved would satisfy the limit.
///
/// Mutation: skip `builder.limits(...)` → the carriage runs past 0.5 m and this
/// goes red.
#[test]
fn a_limited_slider_stops_at_the_authored_stroke() {
    fn travelled(limits: Option<[f32; 2]>) -> f32 {
        let mut w = PhysicsWorld::new();
        let rail = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.1 },
        );
        let car = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.2 },
        );
        join(
            &mut w,
            rail,
            car,
            JointDesc {
                kind: JointKind::Slider,
                anchor_a: [0.0, 5.0],
                anchor_b: [0.0, 5.0],
                // Straight DOWN, so the stroke is measured along gravity.
                axis_a: [0.0, -1.0],
                axis_b: [0.0, -1.0],
                limits,
                ..Default::default()
            },
        )
        .expect("the slider builds");
        for _ in 0..120 {
            w.step();
        }
        5.0 - pose(&w, car)[1]
    }
    let free = travelled(None);
    let capped = travelled(Some([-0.5, 0.5]));
    assert!(
        free > 1.0,
        "the control: an unlimited slider keeps falling, got {free} m"
    );
    assert!(
        (capped - 0.5).abs() < 0.05,
        "a [-0.5, 0.5] stroke stops the carriage half a metre down, got {capped} m"
    );
}

/// **A degenerate axis does not poison the solver.**
///
/// Normalising a zero vector is `NaN`, and a `NaN` axis does not fail loudly — it
/// flows into the pose, the readback and the determinism hash. A joint whose axis
/// never got authored (or arrived non-finite from a project file) gets the
/// horizontal rail an unrotated joint means.
///
/// Mutation: normalise without the guard → the pose reads `NaN` and this goes red.
///
/// ⚠️ **Este gate cobre METADE da porta `unit_or_x`** — o ramo do RECUO. A outra
/// metade (o ramo que NORMALIZA) é a que o irmão abaixo passou a medir; leia lá
/// por que ela deixou de ser de graça.
#[test]
fn a_degenerate_slider_axis_falls_back_instead_of_poisoning_the_pose() {
    for axis in [[0.0, 0.0], [f32::NAN, 1.0], [f32::INFINITY, 0.0]] {
        let mut w = PhysicsWorld::new();
        let rail = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.1 },
        );
        let car = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.2 },
        );
        join(
            &mut w,
            rail,
            car,
            JointDesc {
                kind: JointKind::Slider,
                anchor_a: [0.0, 5.0],
                anchor_b: [0.0, 5.0],
                axis_a: axis,
                axis_b: axis,
                ..Default::default()
            },
        )
        .expect("the slider builds");
        for _ in 0..30 {
            w.step();
        }
        let p = pose(&w, car);
        assert!(
            p[0].is_finite() && p[1].is_finite(),
            "axis {axis:?} poisoned the pose: {p:?}"
        );
    }
}

/// ⭐⭐ **UM EIXO NÃO-UNITÁRIO É NORMALIZADO — a metade que o TIPO segurava de graça.**
///
/// Até à `rapier2d` 0.31 o eixo de um slider era um `UnitVector2<f32>`: um *newtype* em que
/// «isto está normalizado» fazia parte do **tipo**, e o builder **exigia-o** — entregar um vetor
/// cru não compilava. Na 0.35 ele toma um `Vector` (`glam::Vec2`) e nada no compilador impede que
/// lá chegue um vetor por normalizar.
///
/// ⇒ A garantia mudou de dono: era do tipo, passou a ser da porta `unit_or_x`. **E uma garantia
/// que muda de dono precisa de um gate, senão ela desaparece na primeira refactoração** — foi
/// exactamente isso que a migração de 2026-08-29 achou: o gate irmão acima cobria só o recuo, e o
/// único teste com eixo diagonal usava `[1/√2, −1/√2]`, que **já é unitário** e passaria igual se
/// a chamada a `normalize` fosse apagada.
///
/// # Como este mede o que aquele não media
///
/// O eixo autorado é `[3, 4]` — norma **5**, escolhido por ser um terno pitagórico, de modo que a
/// direção normalizada `(0,6 · 0,8)` é exacta em `f32` e o gate não depende de tolerância de
/// arredondamento. Um slider deixa o carro correr **ao longo** do eixo e prende-o em tudo o resto;
/// com o eixo por normalizar, o trilho leva uma escala embutida e o carro percorre a mesma
/// distância física por uma quantidade diferente do parâmetro do limite.
///
/// ⇒ A régua é o **limite de curso**: com `limits: [-0,5; 0,5]` e um eixo unitário, o carro pára a
/// meio metro da âncora. Se o eixo chegar com norma 5, o mesmo limite passa a valer 5× — e a
/// distância percorrida denuncia-o sem que nada fique `NaN`.
///
/// ⚠️ **A prova de que ele morde:** apagar o `.normalize()` do ramo verdadeiro de `unit_or_x`
/// deixa este gate VERMELHO e o irmão acima VERDE — que é a definição de a metade que faltava.
///
/// # ⛔⛔ E faltava-lhe o CONTROLO POSITIVO — a igualdade sozinha é satisfeita por `0,0 == 0,0`
///
/// Até 2026-08-30 este gate media *«os dois percorreram o mesmo»* e *«nenhum é `NaN`»*, e
/// **nada exigia que algum deles se mexesse**. Um carro parado na âncora satisfaz as duas
/// afirmações com a normalização por testar. O irmão
/// [`a_limited_slider_stops_at_the_authored_stroke`] sempre teve esta metade (*«o controlo:
/// um slider sem limite continua a cair»*) — este não. A linha nova é a dele.
///
/// ## Medido e provado por mutação (2026-08-30, `--profile ci-test`)
///
/// | mutação em `world/joints.rs::unit_or_x` | `d_unit` | `d_scaled` | quem apanha |
/// |---|---|---|---|
/// | — (o que shipa) | `0,500000` | `0,500000` | ⭐ verde |
/// | apagar o `.normalize()` | `0,500000` | **`0,100000`** | a IGUALDADE (a metade velha) |
/// | o eixo nunca honrado, sempre o recuo `X` | **`0,000000`** | **`0,000000`** | ⛔ só o **CONTROLO** |
///
/// ⇒ A 2ª mutação é literalmente o `0,0 == 0,0` do parágrafo acima: ela **passa** pela
/// igualdade e pelo `is_finite`, e a única linha que a vê é a que faltava.
#[test]
fn a_non_unit_slider_axis_is_normalised_before_it_reaches_the_solver() {
    // O mesmo eixo em duas escalas: um já unitário, o outro cinco vezes maior.
    let unit = [0.6_f32, 0.8];
    let scaled = [3.0_f32, 4.0];

    let travel = |axis: [f32; 2]| -> f32 {
        let mut w = PhysicsWorld::new();
        let rail = body(
            &mut w,
            RigidBodyType::Fixed,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.1 },
        );
        let car = body(
            &mut w,
            RigidBodyType::Dynamic,
            0.0,
            5.0,
            ShapeDesc::Ball { radius: 0.2 },
        );
        join(
            &mut w,
            rail,
            car,
            JointDesc {
                kind: JointKind::Slider,
                anchor_a: [0.0, 5.0],
                anchor_b: [0.0, 5.0],
                axis_a: axis,
                axis_b: axis,
                limits: Some([-0.5, 0.5]),
                ..Default::default()
            },
        )
        .expect("the slider builds");
        for _ in 0..240 {
            w.step();
        }
        let p = pose(&w, car);
        // Quanto o carro andou a partir da âncora — a gravidade empurra-o até ao limite.
        ((p[0] - 0.0).powi(2) + (p[1] - 5.0).powi(2)).sqrt()
    };

    let d_unit = travel(unit);
    let d_scaled = travel(scaled);

    println!("slider axis: unit {d_unit:.6} m, scaled {d_scaled:.6} m");

    assert!(
        d_unit.is_finite() && d_scaled.is_finite(),
        "nenhum dos dois pode envenenar a pose: {d_unit} / {d_scaled}"
    );
    // ⛔⛔ **O CONTROLO POSITIVO, e sem ele este gate era um `0,0 == 0,0`.** A igualdade
    // abaixo é satisfeita por um carro que nunca saiu da âncora — finito, igual, e com a
    // normalização por testar. O irmão `a_limited_slider_stops_at_the_authored_stroke`
    // sempre teve esta metade (*"o controlo: um slider sem limite continua a cair"*); esta
    // não. Medido 2026-08-30: `d_unit = 0,500000 m` — o carro encosta EXACTAMENTE no
    // batente de `0,5 m`, então a barra de `0,4 m` tem `20%` de folga e um carro parado
    // (`0,0`) atravessa-a por inteiro.
    assert!(
        d_unit > 0.4,
        "o controle: o carro tem de correr ate' ao batente, e andou {d_unit:.4} m. Sem esta \
         metade um carro PARADO daria `0,0 == 0,0` e a igualdade abaixo passaria com a \
         normalizacao por testar"
    );
    assert!(
        (d_unit - d_scaled).abs() < 0.02,
        "o eixo [3, 4] (norma 5) percorreu {d_scaled:.3} m contra os {d_unit:.3} m do [0,6; 0,8] \
         — o mesmo trilho, a mesma direccao, o mesmo limite de curso. A diferenca so' pode vir de \
         o eixo ter chegado ao solver POR NORMALIZAR: a `unit_or_x` deixou de normalizar, e a \
         garantia que o `UnitVector2` dava de graca ate' a` rapier 0.31 morreu sem que nada \
         ficasse NaN. Leia o doc da `unit_or_x`."
    );
}
