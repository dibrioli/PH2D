//! **A sonda headless da cena 50** (W-J8) — os três rigs da smoke, sem janela.
//!
//! Existe porque duas cenas desta jornada afirmaram números que a medição
//! desmentiu: uma mensagem de smoke é uma AFIRMAÇÃO sobre o que o artista vai
//! ver, e ela tem de ser medida antes de ser escrita. Roda com
//! `cargo test -p ph2d-physics-ecs --test measure_smoke_50 -- --ignored --nocapture`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, RigidBody,
};

fn statik(sim: &mut SimWorld, name: &str, at: Vec2, shape: ColliderShape) {
    body(sim, name, BodyKind::Static, at, shape);
}

fn body(sim: &mut SimWorld, name: &str, kind: BodyKind, at: Vec2, shape: ColliderShape) {
    sim.world_mut().spawn((
        Name::new(name),
        RigidBody { kind },
        Collider {
            shape,
            ..Collider::default()
        },
        Transform::from_translation(at),
    ));
}

fn joint(sim: &mut SimWorld, name: &str, j: PhysicsJoint, at: Vec2) {
    sim.world_mut()
        .spawn((Name::new(name), j, Transform::from_translation(at)));
}

fn named(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entity exists")
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let e = named(sim, name);
    sim.world().get::<Transform>(e).unwrap().translation.y
}

/// The scene's floor: a wide static slab at y = 0, like `spawn_floor`.
fn floor(sim: &mut SimWorld) {
    statik(
        sim,
        "Floor",
        Vec2::new(0.0, -0.25),
        ColliderShape::Cuboid {
            half_x: 40.0,
            half_y: 0.25,
        },
    );
}

fn run(sim: &mut SimWorld, ticks: u64) {
    let mut bridge = PhysicsBridge::default();
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
}

/// Settle the rig, then hold the clock STILL — which is the state the anchor dot
/// is drawn in at all (`sync_joint_pivots` is rest-only: during play the overlay
/// draws the live solver anchors, so writing a display value every frame would be
/// work for no reader). Returns the bridge so the caller can keep holding.
fn run_then_hold(sim: &mut SimWorld, ticks: u64) -> PhysicsBridge {
    let mut bridge = PhysicsBridge::default();
    for t in 1..=ticks {
        bridge.dispatch(sim, true, t);
    }
    for _ in 0..3 {
        bridge.dispatch(sim, false, ticks);
    }
    bridge
}

#[test]
#[ignore = "measurement harness for the smoke message"]
fn where_the_smoke_50_rigs_settle() {
    // --- Rig A: the two arms, one disarmed. ---
    let mut sim = SimWorld::new();
    floor(&mut sim);
    for (hook, arm, x, active) in [
        ("Arm Hook On", "Arm On", -7.5, true),
        ("Arm Hook Off", "Arm Off", -4.5, false),
    ] {
        statik(
            &mut sim,
            hook,
            Vec2::new(x, 8.0),
            ColliderShape::Ball { radius: 0.08 },
        );
        body(
            &mut sim,
            arm,
            BodyKind::Dynamic,
            Vec2::new(x + 0.7, 8.0),
            ColliderShape::Cuboid {
                half_x: 0.7,
                half_y: 0.15,
            },
        );
        joint(
            &mut sim,
            &format!("{hook} : {arm}"),
            PhysicsJoint {
                body_a: stable_name_id(hook),
                body_b: stable_name_id(arm),
                kind: JointKind::Pin,
                active,
                ..PhysicsJoint::default()
            },
            Vec2::new(x, 8.0),
        );
    }
    run(&mut sim, 300);
    println!("\n== Rig A (Active) ==");
    println!("  Arm On  y = {:.2}", y_of(&mut sim, "Arm On"));
    println!("  Arm Off y = {:.2}", y_of(&mut sim, "Arm Off"));

    // --- Rig B: the two shelves, one that lets the crate through. ---
    let mut sim = SimWorld::new();
    floor(&mut sim);
    for (shelf, krate, x, collide) in [
        ("Shelf Through", "Crate Through", -1.0, false),
        ("Shelf Rest", "Crate Rest", 2.5, true),
    ] {
        statik(
            &mut sim,
            shelf,
            Vec2::new(x, 5.0),
            ColliderShape::Cuboid {
                half_x: 1.0,
                half_y: 0.25,
            },
        );
        body(
            &mut sim,
            krate,
            BodyKind::Dynamic,
            Vec2::new(x, 7.5),
            ColliderShape::Cuboid {
                half_x: 0.4,
                half_y: 0.4,
            },
        );
        joint(
            &mut sim,
            &format!("{shelf} : {krate}"),
            PhysicsJoint {
                body_a: stable_name_id(shelf),
                body_b: stable_name_id(krate),
                kind: JointKind::Rope,
                max_length: 4.0,
                collide_connected: collide,
                ..PhysicsJoint::default()
            },
            Vec2::new(x, 5.0),
        );
    }
    run(&mut sim, 300);
    println!("\n== Rig B (Collide) ==");
    println!("  Crate Through y = {:.2}", y_of(&mut sim, "Crate Through"));
    println!("  Crate Rest    y = {:.2}", y_of(&mut sim, "Crate Rest"));

    // --- Rig C: the rope whose ends get swapped. The claim to check is that
    //     the LOAD does not move, and the display pivot does.
    let mut sim = SimWorld::new();
    floor(&mut sim);
    statik(
        &mut sim,
        "Rope Hook",
        Vec2::new(6.5, 8.0),
        ColliderShape::Ball { radius: 0.08 },
    );
    body(
        &mut sim,
        "Rope Load",
        BodyKind::Dynamic,
        Vec2::new(6.5, 6.0),
        ColliderShape::Cuboid {
            half_x: 0.4,
            half_y: 0.4,
        },
    );
    let j = PhysicsJoint {
        body_a: stable_name_id("Rope Hook"),
        body_b: stable_name_id("Rope Load"),
        kind: JointKind::Rope,
        max_length: 2.0,
        ..PhysicsJoint::default()
    };
    joint(&mut sim, "Rope Hook : Rope Load", j, Vec2::new(6.5, 8.0));
    let mut bridge = run_then_hold(&mut sim, 300);
    let before = y_of(&mut sim, "Rope Load");
    let pivot_before = y_of(&mut sim, "Rope Hook : Rope Load");

    // Swap in place, as the §12 button does, and let the bridge reconcile — with
    // the clock STILL, because that is when the artist is looking at the dot.
    let e = named(&mut sim, "Rope Hook : Rope Load");
    let swapped = sim.world().get::<PhysicsJoint>(e).unwrap().swapped();
    *sim.world_mut().get_mut::<PhysicsJoint>(e).unwrap() = swapped;
    for _ in 0..3 {
        bridge.dispatch(&mut sim, false, 300);
    }
    println!("\n== Rig C (Swap) ==");
    println!(
        "  Rope Load y  {before:.4} -> {:.4}   (a carga NAO se mexe)",
        y_of(&mut sim, "Rope Load")
    );
    println!(
        "  pivot     y  {pivot_before:.4} -> {:.4}   (o ponto ambar SALTA de ponta)",
        y_of(&mut sim, "Rope Hook : Rope Load")
    );
}
