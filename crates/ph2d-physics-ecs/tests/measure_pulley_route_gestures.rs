//! **SONDA: que outros gestos mudam a ROTA da corda sem avisar ninguém?**
//!
//! A wave do raio (2026-07-29) fechou a explosão por uma PORTA
//! (`reseat_wheel_geometry`), chamada por três gestos: arrastar o centro, arrastar
//! o aro, e digitar o raio na §13. Mas o `L0` é derivado da rota, e a rota tem
//! mais entradas que essas três — e *uma condição que enumera seus leitores
//! apodrece*.
//!
//! Isto NÃO é um gate — é a medição que decide o desenho. Rode com
//! `cargo test -p ph2d-physics-ecs --test measure_pulley_route_gestures -- --nocapture`.
//!
//! Quatro gestos, nenhum deles passando pela porta hoje:
//!
//! 1. **ACRESCENTAR** uma roldana (o botão "Add Wheel" da §12);
//! 2. **MOVER** o centro de uma roldana digitando Position (o commit da §0);
//! 3. **APAGAR** uma roldana (o delete genérico da Hierarquia — que nunca vai
//!    saber o que é uma corda);
//! 4. **AUTORAR um comprimento curto demais** na row `Rope Length (m)`, que é
//!    editável numa polia.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// O MESMO elevador da sonda do raio: carga de 3 kg, contrapeso de 1 kg, duas
/// roldanas no alto.
fn rig() -> SimWorld {
    let mut sim = SimWorld::new();
    let mut body = |name: &str, x: f32, y: f32, kind: BodyKind, mass: f32| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: mass / (std::f32::consts::PI * 0.04),
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(x, y)),
        ));
    };
    body("Floor", 0.0, -4.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 3.0);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 1.0);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Counter"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(-1.5, 2.0)),
    ));
    for (i, x) in [-1.5f32, 1.5].into_iter().enumerate() {
        sim.world_mut().spawn((
            Name::new(format!("Rope Wheel {}", i + 1)),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: u16::try_from(i).expect("duas roldanas"),
                radius: 0.3,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }
    sim
}

fn entity_of(sim: &mut SimWorld, name: &str) -> Entity {
    let mut q = sim.world_mut().query::<(Entity, &Name)>();
    q.iter(sim.world())
        .find(|(_, n)| n.as_str() == name)
        .map(|(e, _)| e)
        .expect("entidade viva")
}

fn y_of(sim: &mut SimWorld, name: &str) -> f32 {
    let e = entity_of(sim, name);
    sim.world().get::<Transform>(e).expect("t").translation.y
}

fn rope_l0(sim: &mut SimWorld) -> f32 {
    let e = entity_of(sim, "Rope");
    sim.world().get::<PhysicsJoint>(e).expect("j").max_length
}

/// O comprimento que a rota de fato desenha AGORA, pela porta do solver.
fn route_len(bridge: &PhysicsBridge) -> f32 {
    let Some(v) = bridge.joint_views().next() else {
        return f32::NAN;
    };
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).map_or(f32::NAN, |r| r.length)
}

/// Roda 60 ticks e devolve `(maior salto num tick, y final da carga)`.
fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> (f32, f32) {
    let mut prev = (y_of(sim, "Load"), y_of(sim, "Counter"));
    let mut worst = 0.0f32;
    for t in 1..=60u64 {
        bridge.dispatch(sim, true, t);
        let now = (y_of(sim, "Load"), y_of(sim, "Counter"));
        worst = worst.max((now.0 - prev.0).abs().max((now.1 - prev.1).abs()));
        prev = now;
    }
    (worst, prev.0)
}

fn report(label: &str, sim: &mut SimWorld, bridge: &mut PhysicsBridge) {
    let (l0, len) = (rope_l0(sim), route_len(bridge));
    let (worst, end_y) = run(sim, bridge);
    println!(
        "  {label:<34} L0={l0:7.4}  rota={len:7.4}  VIOLACAO={:+8.4}  \
         maior salto={worst:8.4}  carga y={end_y:+7.3}",
        len - l0
    );
}

#[test]
fn measure_which_gestures_move_the_route_behind_the_doors_back() {
    println!("\n=== 0. CONTROLE: ninguém tocou em nada ===");
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    report("controle", &mut sim, &mut bridge);

    println!("\n=== 1. ACRESCENTAR uma roldana (o botao Add Wheel) ===");
    for (label, radius) in [("add wheel r=0.30", 0.30f32), ("add wheel r=0.60", 0.60)] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        // O que `add_pulley_wheel` do shell faz: spawna a 3ª roldana no meio do
        // ultimo trecho, herdando o raio da anterior.
        sim.world_mut().spawn((
            Name::new("Rope Wheel 3"),
            PulleyWheel {
                rope: stable_name_id("Rope"),
                order: 2,
                radius,
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(0.0, 6.0)),
        ));
        bridge.dispatch(&mut sim, false, 0);
        report(label, &mut sim, &mut bridge);
    }

    println!("\n=== 2. MOVER o centro de uma roldana (commit de Position) ===");
    for (label, to) in [
        ("move wheel +2 em y", Vec2::new(-1.5, 8.0)),
        ("move wheel -2 em y", Vec2::new(-1.5, 4.0)),
        ("move wheel -3 em x", Vec2::new(-4.5, 6.0)),
    ] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let w = entity_of(&mut sim, "Rope Wheel 1");
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(w) {
            t.translation = to;
        }
        bridge.dispatch(&mut sim, false, 0);
        report(label, &mut sim, &mut bridge);
    }

    println!("\n=== 3. APAGAR uma roldana (o delete da Hierarquia) ===");
    {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let w = entity_of(&mut sim, "Rope Wheel 2");
        let _ = sim.world_mut().despawn(w);
        bridge.dispatch(&mut sim, false, 0);
        report("apagar a 2a roldana", &mut sim, &mut bridge);
    }

    println!("\n=== 4. AUTORAR um comprimento CURTO na row Rope Length ===");
    for l in [8.0f32, 5.0, 1.0] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let e = entity_of(&mut sim, "Rope");
        if let Some(mut j) = sim.world_mut().get_mut::<PhysicsJoint>(e) {
            j.max_length = l;
        }
        bridge.dispatch(&mut sim, false, 0);
        report(&format!("Rope Length = {l:.1}"), &mut sim, &mut bridge);
    }
    println!();
}
