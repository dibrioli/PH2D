#![forbid(unsafe_code)]
//! `physics-ecs-c9` — the ECS-bridged cross-OS determinism harness.
//!
//! Sibling of `ph2d_physics_c9` (which drives the raw wrapper). This one
//! drives the same falling-circles fixture **through the ECS bridge** (plus
//! one non-uniformly scaled ball → an ELLIPSE collider, W6, one
//! `GravityScale(0.5)` ball → a per-body gravity multiplier, W8, and one
//! non-uniformly scaled CAPSULE → a STADIUM hull, and one LAUNCHED ball →
//! an `InitialVelocity`, one CCD ball launched at a thin wall → the CCD
//! solver's sweep, W-CCD, one ROTATION-LOCKED spinning box → the frozen
//! angular DOF, W-LockRot, one OFFSET collider → the collider translation,
//! W-Offset, one X-LOCKED launched ball → the frozen translation DOF,
//! W-LockPos, one HEAVY ball with a manual mass override → the collider mass
//! property, W-Mass, one LIGHT high-dominance ball plowing a heavy one → the
//! contact-solver dominance path, W-Dominance, one MAX-combine superball
//! bouncing off the dead floor → the restitution combine rule in the contact
//! solver, W-Material, one DAMPED launched ball → the per-body drag fold in the
//! integrator, W-Damping, and one ball launched UP through a ONE-WAY platform → the
//! contact-modification hook, W-OneWay, and one ball falling through a WIND COLUMN →
//! the force-zone impulse read from the narrow phase's intersection graph, W-Area, and
//! one ball sinking through a DRAG POOL → the zone's decay applied outside rapier's own
//! damping point, W-AreaDrag, and one crate FLOATING in a pool → o recorte de polígono
//! do empuxo e o impulso aplicado num PONTO, W-Buoyancy): entities carry
//! `RigidBody`/`Collider`, the bridge spawns rapier bodies, steps at the
//! tick, and reads poses back into `Transform`. The hash is over those
//! readback `Transform`s — so it proves OUR code (iteration order, the
//! meters↔rapier boundary, the readback, the libm ellipse tessellation) is
//! bit-identical cross-OS, not just rapier's internal state (ADR-0131 D7).
//!
//! CI runs this on Linux/macOS/Windows and compares the three hashes
//! (`.github/workflows/spike.yml`). Output format (stable, parsed by CI):
//! ```text
//! physics-ecs-c9 step_count: 120
//! physics-ecs-c9 body_count: 83
//! physics-ecs-c9 hash: <hex64>
//! ```

mod zones;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Ccd, Collider, ColliderShape, CombineRule, DampMode, DampingOverride, Dominance,
    GravityScale, InitialVelocity, JointKind, LockPositionX, LockRotation, MassOverride,
    MaterialCombine, MotorMode, OneWayPlatform, PhysicsBridge, PhysicsJoint, RigidBody,
};

const STEPS: u64 = 120; // 2 s @ 60 Hz — long enough for collisions to develop.
const N_DYNAMIC: u32 = 50;

fn main() {
    let mut sim = SimWorld::new();

    // Static floor: cuboid at y=0, half-thickness 0.1, half-width 50.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 50.0,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));

    // 50 dynamic circles in a 10×5 grid above the floor (same layout as
    // the raw wrapper's c9 fixture).
    for i in 0..N_DYNAMIC {
        let row = (i / 10) as f32;
        let col = (i % 10) as f32;
        sim.world_mut().spawn((
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.25 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(col * 0.6 - 2.7, 5.0 + row * 0.6)),
        ));
    }

    // One NON-uniformly scaled ball (W6): its `Ball` resolves to an ELLIPSE
    // (`scaled_shape`), built from `ellipse_vertices` via libm. Its poses feed
    // the hash, so CI comparing the three OSes proves the ellipse tessellation
    // is bit-identical cross-platform — the only ML-free transcendental on the
    // physics path. A `f32::sin_cos` here would diverge in the last ulps and
    // this hash would split across OSes.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        Transform {
            translation: Vec2::new(0.3, 8.0),
            rotation: 0.0,
            scale: Vec2::new(1.6, 2.4),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));

    // One ball with a per-body gravity scale (W8): `GravityScale(0.5)` makes it
    // fall at half rate. Its poses feed the hash, so the multiplier travels the
    // deterministic path (an f32 fold in the bridge's substep integration) and
    // CI proves it is bit-identical across the three OSes — the same guarantee
    // the ellipse gives the tessellation. Set apart on x so it lands on the same
    // floor without touching the grid.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        GravityScale(0.5),
        Transform::from_translation(Vec2::new(4.0, 6.0)),
    ));

    // One NON-uniformly scaled CAPSULE: it degrades to a `Stadium`, whose hull
    // comes from `capsule_vertices` — our own libm tessellation, exactly like
    // the ellipse above. That is what puts the capsule path on the cross-OS
    // hash: an `f32::sin_cos` in there would split the three OSes in the last
    // ulps and this is what would catch it. (A *uniform* capsule is rapier's
    // own exact shape and carries no transcendental of ours.)
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.20,
                radius: 0.15,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform {
            translation: Vec2::new(-4.0, 7.0),
            rotation: 0.0,
            scale: Vec2::new(1.7, 1.1),
            skew_x: 0.0,
            skew_y: 0.0,
        },
    ));

    // One LAUNCHED ball (W9): a nonzero `InitialVelocity` applied at spawn. Its
    // poses feed the hash, so the launch travels the deterministic path (an f32
    // set on the rigid body at build) and CI proves it is bit-identical across
    // the three OSes — same guarantee gravity scale gets. Aimed up-and-left into
    // empty space so it does not perturb the grid.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [-1.5, 3.0],
            angvel: 2.0,
        },
        Transform::from_translation(Vec2::new(-4.0, 4.0)),
    ));

    // One CCD ball (W-CCD): launched fast at a thin static wall so the CCD SOLVER
    // actually runs its conservative-advancement / time-of-impact sweep. That puts
    // the CCD path on the cross-OS hash — an `f32` divergence in the sweep would
    // split the three OSes here, the same guarantee the ellipse gives the
    // tessellation. A body without the `Ccd` marker would tunnel and never invoke
    // the solver, so the marker is load-bearing for this coverage. Placed far off
    // in +x, its own lane, so it does not perturb the grid.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.02,
                half_y: 1.0,
            },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(20.0, 8.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [80.0, 0.0],
            angvel: 0.0,
        },
        Ccd,
        Transform::from_translation(Vec2::new(19.0, 8.0)),
    ));

    // One ROTATION-LOCKED box (W-LockRot): spun at t=0 but pinned by the marker,
    // so `LockedAxes::ROTATION_LOCKED` travels the deterministic path (an `f32`
    // fold in the solver's DOF handling) and CI proves it is bit-identical across
    // the three OSes — the same guarantee gravity scale gets. Off in its own lane.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.25,
                half_y: 0.25,
            },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [0.0, 0.0],
            angvel: 5.0,
        },
        LockRotation,
        Transform::from_translation(Vec2::new(-8.0, 8.0)),
    ));

    // One body with an OFFSET collider (W-Offset): its collider sits off the body
    // centre, so `BodyDesc.offset` folds into rapier's collider translation and
    // changes where the body rests — an `f32` on the deterministic path that CI
    // proves is bit-identical cross-OS. Off in its own lane so it settles alone.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            offset: [0.0, 0.4],
            ..Collider::default()
        },
        // Far left, clear of the CCD ball's rightward flight and the spinning box.
        Transform::from_translation(Vec2::new(-15.0, 3.0)),
    ));

    // One X-LOCKED launched ball (W-LockPos): fired sideways but pinned on X by the
    // `LockPositionX` marker, so `LockedAxes::TRANSLATION_LOCKED_X` travels the
    // deterministic path (an `f32` fold in the solver's DOF handling) and CI proves
    // it is bit-identical cross-OS — the same guarantee the rotation lock gets. The
    // launch would carry a free body away; the lock cancels the X component while it
    // still falls, so its Y descent under the pinned X exercises the fold. Its own
    // lane, far left, so it settles alone.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [6.0, 0.0],
            angvel: 0.0,
        },
        LockPositionX,
        Transform::from_translation(Vec2::new(-22.0, 6.0)),
    ));

    // One HEAVY ball with a manual mass override (W-Mass): `MassOverride(20.0)` sets
    // the collider mass to 20 kg, ignoring density, so `ColliderBuilder::mass` (not
    // `.density`) travels the deterministic path. It falls onto the floor next to a
    // light one; the mass changes the contact impulse (an `f32` fold in the solver's
    // mass matrix), so CI proves it is bit-identical cross-OS — the same guarantee
    // gravity scale gets. Off in its own lane so it settles beside the light control.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        MassOverride(20.0),
        Transform::from_translation(Vec2::new(-28.0, 5.0)),
    ));

    // One LIGHT high-dominance ball (W-Dominance): `Dominance(5)` launched into a
    // heavy neutral ball, so the dominance path (rapier's `relative_dominance` in the
    // contact solver) travels the deterministic hash — an `f32` fold CI proves is
    // bit-identical cross-OS, the same guarantee mass gets. It plows through the heavy
    // one instead of bouncing. Its own lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [5.0, 0.0],
            angvel: 0.0,
        },
        Dominance(5),
        Transform::from_translation(Vec2::new(-34.0, 5.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.3 },
            density: 1.0,
            ..Collider::default()
        },
        MassOverride(30.0),
        Transform::from_translation(Vec2::new(-32.0, 5.0)),
    ));

    // One MAX-combine superball (W-Material): restitution 1.0 with a
    // `MaterialCombine{restitution: Max}`, dropped onto the floor. rapier combines
    // the pair's restitution with `rule1.max(rule2)`, so the `Max` rule travels the
    // contact solver (an `f32` fold in the restitution the impulse uses) — and CI
    // proves it is bit-identical cross-OS, the same guarantee dominance gets. Its
    // bounce off the dead floor is where the fold shows; without the component it
    // would average and settle differently. Its own lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            restitution: 1.0,
            ..Collider::default()
        },
        MaterialCombine {
            restitution: CombineRule::Max,
            friction: CombineRule::Average,
        },
        Transform::from_translation(Vec2::new(-40.0, 5.0)),
    ));

    // One DAMPED launched ball (W-Damping): a `DampingOverride` with heavy linear +
    // angular drag, launched into empty space. The drag decays both velocities each
    // step (an `f32` fold in the integrator), so the override travels the deterministic
    // hash — CI proves it is bit-identical cross-OS, the same guarantee dominance gets.
    // It is also re-stamped each dispatch by the bridge's pass, on the same path. Its
    // own lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [4.0, 0.0],
            angvel: 6.0,
        },
        DampingOverride {
            linear: 2.5,
            angular: 1.5,
            mode: DampMode::Combine,
        },
        Transform::from_translation(Vec2::new(-46.0, 6.0)),
    ));

    // One ONE-WAY platform + the ball launched UP through it (W-OneWay): the contact
    // modification hook runs inside the narrow phase and CLEARS solver contacts on the
    // forbidden side, so both the pass-through and the landing that follows are folds
    // on the deterministic path — CI proves the hook is bit-identical cross-OS, the
    // same guarantee the dominance solver path gets. Its own lane, far left.
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.5,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        OneWayPlatform,
        Transform::from_translation(Vec2::new(-52.0, 2.0)),
    ));
    sim.world_mut().spawn((
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.25 },
            density: 1.0,
            ..Collider::default()
        },
        InitialVelocity {
            linvel: [0.0, 8.0],
            angvel: 0.0,
        },
        Transform::from_translation(Vec2::new(-52.0, 0.0)),
    ));

    // Um SLIDER com curso (W-J5): a primeira JOINT no hash, e ela entra porque o
    // prismatic e um caminho de solver PROPRIO (a restricao de um grau de
    // liberdade de translacao, mais os batentes do curso) e porque o eixo dele
    // cruza `libm::sincosf` no caminho determinista. Um trilho a 45 graus de
    // proposito: no horizontal ou no vertical o seno e o cosseno sao 0 ou 1
    // exatos, e uma diagonal e o unico angulo que de fato exercita a trigonometria.
    //
    // ⚠️ Lane propria, longe de tudo, e com os DOIS corpos nomeados -- uma joint
    // referencia os corpos por hash de `Name`, entao um corpo sem nome e um que
    // ela nao consegue apontar.
    sim.world_mut().spawn((
        Name::new("C9 Rail"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-58.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Car"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.2 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-58.0, 6.0)),
    ));
    let mut rail_t = Transform::from_translation(Vec2::new(-58.0, 6.0));
    // -45 graus: para baixo e para a direita, com a gravidade puxando ao longo.
    rail_t.rotation = -std::f32::consts::FRAC_PI_4;
    sim.world_mut().spawn((
        Name::new("C9 Slider"),
        PhysicsJoint {
            body_a: stable_name_id("C9 Rail"),
            body_b: stable_name_id("C9 Car"),
            kind: JointKind::Slider,
            limits_enabled: true,
            limit_min: -PhysicsJoint::DEFAULT_STROKE,
            limit_max: PhysicsJoint::DEFAULT_STROKE,
            ..PhysicsJoint::default()
        },
        rail_t,
    ));

    // Um SERVO (W-J6): a mesma dobradica do resto do repo, mas mirando um LUGAR
    // em vez de uma taxa. Entra no hash porque o motor de POSICAO e um caminho de
    // solver proprio -- `set_motor` com stiffness diferente de zero, resolvido
    // junto com os contatos -- e porque ele SEGURA (o corpo nao dorme), entao
    // toda divergencia de plataforma continua acumulando ate o fim dos passos em
    // vez de ser congelada pelo sono no primeiro segundo.
    sim.world_mut().spawn((
        Name::new("C9 Servo Hook"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Ball { radius: 0.05 },
            density: 1.0,
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(-64.0, 6.0)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Servo Arm"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 0.1,
            },
            density: 1.0,
            ..Collider::default()
        },
        // Pendurado: a gravidade puxa para LONGE do alvo o tempo todo, que e o
        // que torna o servo observavel em vez de coincidente.
        Transform::from_translation(Vec2::new(-64.0, 5.5)),
    ));
    sim.world_mut().spawn((
        Name::new("C9 Servo"),
        PhysicsJoint {
            body_a: stable_name_id("C9 Servo Hook"),
            body_b: stable_name_id("C9 Servo Arm"),
            kind: JointKind::Pin,
            motor_enabled: true,
            motor_mode: MotorMode::Position,
            // 1 rad, um angulo que nao e nem 0 nem um multiplo de pi/4.
            motor_target: 1.0,
            ..PhysicsJoint::default()
        },
        Transform::from_translation(Vec2::new(-64.0, 6.0)),
    ));

    // As oito lanes da familia das ZONAS -- irmao proprio pelo cap de 700 LOC.
    zones::spawn(&mut sim);

    let mut bridge = PhysicsBridge::new();
    // Drive it exactly as the shell does on play: one tick forward per
    // frame, sequential.
    for tick in 1..=STEPS {
        bridge.dispatch(&mut sim, true, tick);
    }

    let hash = bridge.deterministic_hash(&sim);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();

    println!("physics-ecs-c9 step_count: {STEPS}");
    println!("physics-ecs-c9 body_count: {}", bridge.body_count());
    println!("physics-ecs-c9 hash: {hex}");
}
