//! **SONDA: o que acontece quando o artista muda o RAIO de uma roldana.**
//!
//! Report do Enio (2026-07-29): *"aumentar o diâmetro da polia afasta a ponta da
//! corda dos objetos e na simulação ocorre aqueles saltos explosivos"*.
//!
//! Isto NÃO é um gate — é a medição que decide o desenho. Rode com
//! `cargo test -p ph2d-physics-ecs --test measure_pulley_radius -- --nocapture`.
//!
//! Três perguntas, nesta ordem:
//!
//! 1. **O `L0` acompanha a rota?** O comprimento da corda é semeado UMA vez
//!    (`!joint.anchored`) e depois `anchored = true` o congela. Crescer o raio
//!    cresce a rota (o abraço é maior), e se o `L0` ficar parado a restrição
//!    `L(rota) ≤ L0` nasce **violada**.
//! 2. **Quanto vale essa violação em metros?** É ela que o solver tem de comer,
//!    e é o candidato do *salto explosivo*.
//! 3. **Onde a corda DESENHADA começa e acaba?** O report diz que a ponta se
//!    afasta dos objetos, e o desenho vai da âncora à tangente.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// Um elevador simples: carga e contrapeso por duas roldanas no alto.
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

/// O comprimento que a rota de fato DESENHA agora, pela mesma função do solver.
fn route_len(bridge: &PhysicsBridge) -> f32 {
    let v = bridge.joint_views().next().expect("uma joint");
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).map_or(f32::NAN, |r| r.length)
}

/// Onde a corda desenhada COMEÇA, contra a âncora que devia ser o começo dela.
fn first_gap(bridge: &PhysicsBridge) -> f32 {
    let v = bridge.joint_views().next().expect("uma joint");
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    if rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).is_none() {
        return f32::NAN;
    }
    let from = segs.first().expect("uma perna").from;
    (from[0] - v.anchor_a[0]).hypot(from[1] - v.anchor_a[1])
}

fn set_radius(sim: &mut SimWorld, name: &str, r: f32) {
    let e = entity_of(sim, name);
    if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        w.radius = r;
    }
}

#[test]
fn measure_what_growing_the_radius_does() {
    println!("\n=== 1. EM REPOUSO, o L0 é semeado da rota ===");
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let (l0, len) = (rope_l0(&mut sim), route_len(&bridge));
    println!(
        "  L0 = {l0:.4}  rota = {len:.4}  violacao = {:+.4}",
        len - l0
    );
    println!("  vao desenho->ancora = {:.6}", first_gap(&bridge));

    println!("\n=== 2. O ARTISTA CRESCE O RAIO (0,30 -> 0,90), ainda em repouso ===");
    for r in [0.30f32, 0.45, 0.60, 0.90, 1.50] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let seeded = rope_l0(&mut sim);
        set_radius(&mut sim, "Rope Wheel 1", r);
        bridge.dispatch(&mut sim, false, 0);
        let (l0, len) = (rope_l0(&mut sim), route_len(&bridge));
        println!(
            "  r={r:.2}  L0={l0:.4} (semeado {seeded:.4})  rota={len:.4}  \
             VIOLACAO={:+.4} m  vao={:.6}",
            len - l0,
            first_gap(&bridge)
        );
    }

    println!("\n=== 2b. A MESMA COISA, PELA PORTA DE AUTORIA ===");
    for r in [0.30f32, 0.45, 0.60, 0.90, 1.50] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        set_radius(&mut sim, "Rope Wheel 1", r);
        let w = entity_of(&mut sim, "Rope Wheel 1");
        ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), w);
        bridge.dispatch(&mut sim, false, 0);
        let (l0, len) = (rope_l0(&mut sim), route_len(&bridge));
        println!(
            "  r={r:.2}  L0={l0:.4}  rota={len:.4}  VIOLACAO={:+.4} m",
            len - l0
        );
    }

    println!("\n=== 3. E O QUE A SIM FAZ COM ISSO (60 ticks) ===");
    for (r, door) in [
        (0.30f32, false),
        (0.60, false),
        (0.90, false),
        (1.50, false),
        (0.60, true),
        (0.90, true),
        (1.50, true),
    ] {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        set_radius(&mut sim, "Rope Wheel 1", r);
        if door {
            let w = entity_of(&mut sim, "Rope Wheel 1");
            ph2d_physics_ecs::reseat_wheel_geometry(sim.world_mut(), w);
        }
        bridge.dispatch(&mut sim, false, 0);
        let before = (y_of(&mut sim, "Load"), y_of(&mut sim, "Counter"));
        let mut worst = 0.0f32;
        let mut prev = before;
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
            let now = (y_of(&mut sim, "Load"), y_of(&mut sim, "Counter"));
            let step = (now.0 - prev.0).abs().max((now.1 - prev.1).abs());
            worst = worst.max(step);
            prev = now;
        }
        println!(
            "  r={r:.2} porta={door:<5}  carga {:+.3} -> {:+.3}  contrapeso {:+.3} -> {:+.3}  \
             MAIOR SALTO num tick = {worst:.4} m",
            before.0, prev.0, before.1, prev.1
        );
    }
    println!();
}
