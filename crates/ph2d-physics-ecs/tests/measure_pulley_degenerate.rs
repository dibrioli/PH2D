//! **SONDA: o que o artista vê quando a rota da corda DEGENERA.**
//!
//! O plano (§10) pede *"os guardas de degeneração — âncora dentro de uma roldana,
//! roldanas sobrepostas, `|C₂−C₁| < |r₁±r₂|` — cada um com decisão explícita em vez
//! de `NaN` silencioso"*. O `NaN` **já está barrado** no kernel
//! (`rope_route::tangent` recusa quando `inner <= 0`, e `route` pula a corda
//! inteira), e o overlay já desenha uma reta em vez de inventar geometria.
//!
//! Então a pergunta que sobra não é o `NaN`: é **o que acontece a jusante da
//! recusa** — e sobretudo o que acontece na VOLTA, quando o artista desfaz o gesto
//! que degenerou a rota.
//!
//! Isto NÃO é um gate. Rode com
//! `cargo test -p ph2d-physics-ecs --test measure_pulley_degenerate -- --nocapture`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics::world::rope_route;
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody,
};

/// O elevador das outras sondas: carga de 3 kg, contrapeso de 1 kg, duas roldanas.
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

fn t_of(sim: &mut SimWorld, name: &str) -> Vec2 {
    let e = entity_of(sim, name);
    sim.world().get::<Transform>(e).expect("t").translation
}

fn joint(sim: &mut SimWorld) -> (f32, bool) {
    let e = entity_of(sim, "Rope");
    let j = sim.world().get::<PhysicsJoint>(e).expect("j");
    (j.max_length, j.anchored)
}

/// A rota existe? E que comprimento ela mede?
fn route_of(bridge: &PhysicsBridge) -> Option<f32> {
    let v = bridge.joint_views().next()?;
    let arena = bridge.pulley_wheel_arena();
    let start = v.wheel_start as usize;
    let wheels = &arena[start..start + v.wheel_count as usize];
    let mut segs = Vec::new();
    rope_route::route(v.anchor_a, v.anchor_b, wheels, &mut segs).map(|r| r.length)
}

fn set_wheel(sim: &mut SimWorld, name: &str, centre: Vec2, radius: f32) {
    let e = entity_of(sim, name);
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = centre;
    }
    if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(e) {
        w.radius = radius;
    }
}

/// Roda 60 tiques e devolve `(y da carga, tudo finito?)`.
fn run(sim: &mut SimWorld, bridge: &mut PhysicsBridge) -> (f32, bool) {
    for t in 1..=60u64 {
        bridge.dispatch(sim, true, t);
    }
    let (l, c) = (t_of(sim, "Load"), t_of(sim, "Counter"));
    (
        l.y,
        l.x.is_finite() && l.y.is_finite() && c.x.is_finite() && c.y.is_finite(),
    )
}

/// Os três gestos que degeneram a rota, cada um aplicado a um rig limpo.
fn degenerate(sim: &mut SimWorld, which: &str) {
    match which {
        // A roldana ENGOLE a âncora da carga (que está em (-1.5, 2)).
        "ancora dentro da roldana" => set_wheel(sim, "Rope Wheel 1", Vec2::new(-1.5, 2.2), 1.0),
        // As duas roldanas se SOBREPOEM.
        "roldanas sobrepostas" => set_wheel(sim, "Rope Wheel 2", Vec2::new(-1.2, 6.0), 0.3),
        // Uma roldana DENTRO da outra.
        "roldana dentro da outra" => {
            set_wheel(sim, "Rope Wheel 1", Vec2::new(-1.5, 6.0), 2.0);
            set_wheel(sim, "Rope Wheel 2", Vec2::new(-1.4, 6.0), 0.2);
        }
        _ => unreachable!(),
    }
}

#[test]
fn measure_what_a_degenerate_route_does() {
    let cases = [
        "ancora dentro da roldana",
        "roldanas sobrepostas",
        "roldana dentro da outra",
    ];

    println!("\n=== 0. CONTROLE ===");
    let mut sim = rig();
    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    let (l0, anchored) = joint(&mut sim);
    let route = route_of(&bridge);
    let (y, finite) = run(&mut sim, &mut bridge);
    println!("  rota={route:?}  L0={l0:.4} anchored={anchored}  carga y={y:+.3} finito={finite}");

    println!("\n=== 1. DEGENERAR uma cena que JA foi semeada (o gesto do artista) ===");
    for which in cases {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let seeded = joint(&mut sim).0;
        degenerate(&mut sim, which);
        bridge.dispatch(&mut sim, false, 0);
        let route = route_of(&bridge);
        let (l0, anchored) = joint(&mut sim);
        let (y, finite) = run(&mut sim, &mut bridge);
        println!(
            "  {which:<26} rota={:<9} L0={l0:.4} (semeado {seeded:.4}) anchored={anchored} \
             carga y={y:+.3} finito={finite}",
            route.map_or("None".to_string(), |r| format!("{r:.4}"))
        );
    }

    println!("\n=== 2. E a VOLTA: desfazer o gesto e ver se a corda volta a segurar ===");
    for which in cases {
        let mut sim = rig();
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let seeded = joint(&mut sim).0;
        degenerate(&mut sim, which);
        bridge.dispatch(&mut sim, false, 0);
        // O artista arrasta de volta: as duas roldanas voltam para onde nasceram.
        set_wheel(&mut sim, "Rope Wheel 1", Vec2::new(-1.5, 6.0), 0.3);
        set_wheel(&mut sim, "Rope Wheel 2", Vec2::new(1.5, 6.0), 0.3);
        bridge.dispatch(&mut sim, false, 0);
        let route = route_of(&bridge);
        let (l0, _) = joint(&mut sim);
        let (y, finite) = run(&mut sim, &mut bridge);
        println!(
            "  {which:<26} rota={:<9} L0={l0:.4} (semeado {seeded:.4})  carga y={y:+.3} \
             finito={finite}",
            route.map_or("None".to_string(), |r| format!("{r:.4}"))
        );
    }

    println!("\n=== 3. E o caso FEIO: a cena NASCE degenerada (load de projeto, undo) ===");
    for which in cases {
        let mut sim = rig();
        degenerate(&mut sim, which);
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let (l0_sealed, anchored) = joint(&mut sim);
        let route_bad = route_of(&bridge);
        // E agora o artista conserta a geometria.
        set_wheel(&mut sim, "Rope Wheel 1", Vec2::new(-1.5, 6.0), 0.3);
        set_wheel(&mut sim, "Rope Wheel 2", Vec2::new(1.5, 6.0), 0.3);
        bridge.dispatch(&mut sim, false, 0);
        let route = route_of(&bridge);
        let (l0, _) = joint(&mut sim);
        let (y, finite) = run(&mut sim, &mut bridge);
        println!(
            "  {which:<26} degenerada: rota={:<6} L0={l0_sealed:.4} anchored={anchored}\n  \
             {:<28} consertada: rota={:<9} L0={l0:.4}  carga y={y:+.3} finito={finite}",
            route_bad.map_or("None".to_string(), |r| format!("{r:.4}")),
            "",
            route.map_or("None".to_string(), |r| format!("{r:.4}"))
        );
    }
    println!();
}

/// **O resíduo do §3: um rig MENOR que o default de 1 m, nascido degenerado.**
///
/// O piso só levanta o `L0`; ele nunca o abaixa (senão clobbaria a row
/// `Rope Length`). Então se o selo bogus (1,0 m) for MAIOR que a rota verdadeira, a
/// corda fica folgada para sempre — e a pergunta é quanta folga, e se dá para ver.
#[test]
fn measure_the_residual_of_a_rig_smaller_than_the_default() {
    /// Um elevador de bolso: 0,8 m entre as roldanas.
    fn small(degenerate: bool) -> SimWorld {
        let mut sim = SimWorld::new();
        let mut body = |name: &str, x: f32, y: f32, kind: BodyKind| {
            sim.world_mut().spawn((
                Name::new(name),
                RigidBody { kind },
                Collider {
                    shape: ColliderShape::Ball { radius: 0.05 },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(x, y)),
            ));
        };
        body("Floor", 0.0, -1.0, BodyKind::Static);
        body("Load", -0.4, 0.3, BodyKind::Dynamic);
        body("Counter", 0.4, 0.3, BodyKind::Dynamic);
        sim.world_mut().spawn((
            Name::new("Rope"),
            PhysicsJoint {
                body_a: stable_name_id("Load"),
                body_b: stable_name_id("Counter"),
                kind: JointKind::Pulley,
                ..PhysicsJoint::of_kind(JointKind::Pulley)
            },
            Transform::from_translation(Vec2::new(-0.4, 0.3)),
        ));
        for (i, x) in [-0.4f32, 0.4].into_iter().enumerate() {
            // Nascida degenerada: as duas roldanas no MESMO lugar, uma dentro da
            // outra.
            let (cx, r) = if degenerate {
                (0.0, if i == 0 { 0.3 } else { 0.05 })
            } else {
                (x, 0.05)
            };
            sim.world_mut().spawn((
                Name::new(format!("Rope Wheel {}", i + 1)),
                PulleyWheel {
                    rope: stable_name_id("Rope"),
                    order: u16::try_from(i).expect("duas"),
                    radius: r,
                    ..Default::default()
                },
                Transform::from_translation(Vec2::new(cx, 0.8)),
            ));
        }
        sim
    }

    println!("\n=== O RESIDUO: rig de bolso (rota ~1,0 m), default 1,0 m ===");
    for born_degenerate in [false, true] {
        let mut sim = small(born_degenerate);
        let mut bridge = PhysicsBridge::new();
        bridge.dispatch(&mut sim, false, 0);
        let sealed = joint(&mut sim).0;
        if born_degenerate {
            // O artista conserta a geometria.
            set_wheel(&mut sim, "Rope Wheel 1", Vec2::new(-0.4, 0.8), 0.05);
            set_wheel(&mut sim, "Rope Wheel 2", Vec2::new(0.4, 0.8), 0.05);
            bridge.dispatch(&mut sim, false, 0);
        }
        let route = route_of(&bridge).unwrap_or(f32::NAN);
        let l0 = joint(&mut sim).0;
        let (y, finite) = run(&mut sim, &mut bridge);
        println!(
            "  nasceu degenerada={born_degenerate:<5} selo={sealed:.4}  rota={route:.4} \
             L0={l0:.4}  FOLGA={:+.4}  carga y={y:+.3} finito={finite}",
            l0 - route
        );
    }
    println!();
}

/// **O caso que a pose de REPOUSO não vê: a rota degenera DURANTE o play.**
///
/// O solver resolve contra a pose VIVA (`pulley.rs:310`), então um contrapeso que
/// ergue a carga até ENCOSTAR na roldana degenera a rota no meio da corrida — e o
/// `continue` do passe pula a corda em silêncio. A pose de repouso, contra a qual o
/// reconcile e o overlay resolvem, continua dizendo que está tudo bem.
#[test]
fn measure_a_route_that_degenerates_mid_play() {
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
    // Contrapeso PESADO: ele desce e ergue a carga leve ate a roldana.
    body("Floor", 0.0, -8.0, BodyKind::Static, 1.0);
    body("Load", -1.5, 2.0, BodyKind::Dynamic, 0.5);
    body("Counter", 1.5, 2.0, BodyKind::Dynamic, 6.0);
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
                order: u16::try_from(i).expect("duas"),
                radius: 0.5,
                // O GUINCHO: a roldana 1 enrola, encurtando o L0 -- e um guincho
                // que nao para e o unico jeito de a corda ARRASTAR a ancora para
                // dentro de uma roldana durante o play.
                motor_speed: if i == 0 { 4.0 } else { 0.0 },
                ..Default::default()
            },
            Transform::from_translation(Vec2::new(x, 6.0)),
        ));
    }

    let mut bridge = PhysicsBridge::new();
    bridge.dispatch(&mut sim, false, 0);
    println!("\n=== A rota degenerando DURANTE o play (a carga sobe ate a roldana) ===");
    println!("  rota em repouso = {:?}", route_of(&bridge));
    println!("  tique |  carga y  | contra y  | rota VIVA (do solver)");
    let mut last_live = true;
    let mut closest = f32::INFINITY;
    for t in 1..=240u64 {
        bridge.dispatch(&mut sim, true, t);
        // A rota VIVA: as âncoras onde os corpos ESTÃO agora.
        let live = {
            let v = bridge.joint_views().next().expect("view");
            let arena = bridge.pulley_wheel_arena();
            let s = v.wheel_start as usize;
            let mut segs = Vec::new();
            rope_route::route(
                v.anchor_a,
                v.anchor_b,
                &arena[s..s + v.wheel_count as usize],
                &mut segs,
            )
            .is_some()
        };
        // A DISTANCIA da ancora ao centro da roldana 1: a rota degenera quando
        // ela cai abaixo do raio (0,5), e este numero diz se a sim chega perto.
        {
            let v = bridge.joint_views().next().expect("view");
            let arena = bridge.pulley_wheel_arena();
            let c = arena[v.wheel_start as usize].centre;
            let d = (v.anchor_a[0] - c[0]).hypot(v.anchor_a[1] - c[1]);
            closest = closest.min(d);
        }
        if t % 40 == 0 || live != last_live {
            println!(
                "  {t:>5} | {:>9.3} | {:>9.3} | {}",
                t_of(&mut sim, "Load").y,
                t_of(&mut sim, "Counter").y,
                if live { "ok" } else { "DEGENERADA" }
            );
            last_live = live;
        }
    }
    println!(
        "  distancia MINIMA da ancora ao centro da roldana: {closest:.4} m \
         (a rota degenera abaixo de 0,5)"
    );
    println!();
}
