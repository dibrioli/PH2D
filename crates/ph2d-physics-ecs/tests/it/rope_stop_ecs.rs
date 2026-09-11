//! **O LIMITADOR na fronteira do ECS** (W-RopeStop) — as duas portas que a UI
//! consome: onde a marca fica, e que roldana está sob o cursor.
//!
//! ⚠️ **A roldana tem RAIO nesta fixture, ao contrário da do `rope_pick`**, e a
//! razão é a mesma pela qual aquela usa raio zero: ali o oráculo é um ponto SOBRE
//! a rota, que com raio deixa de ser derivável no papel; aqui o oráculo é o ARO
//! e a folga de TANGÊNCIA — grandezas que **só existem** quando há raio. A
//! fixture tem de conter o fenômeno que o gate mede.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform, stable_name_id};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge, PhysicsJoint, PulleyWheel,
    RigidBody, RopeStops, stop_at_point, stop_mark,
};

/// O raio da roldana — grande o bastante para que o aro e o centro sejam alvos
/// **distintos**, que é o que separa *"clicou no anel"* de *"clicou no disco"*.
const R: f32 = 1.0;

/// Uma corda vertical com uma roldana no alto: carga em `(0, 2)`, ponta morta em
/// `(3, 6)`, eixo em `(0, 6)`.
fn rig() -> (SimWorld, PhysicsBridge, Entity, Entity) {
    let mut sim = SimWorld::new();
    let mut ball = |name: &str, p: [f32; 2], kind: BodyKind| {
        sim.world_mut().spawn((
            Name::new(name),
            RigidBody { kind },
            Collider {
                shape: ColliderShape::Ball { radius: 0.2 },
                density: 1.0,
                ..Collider::default()
            },
            Transform::from_translation(Vec2::new(p[0], p[1])),
        ));
    };
    ball("Load", [0.0, 2.0], BodyKind::Dynamic);
    ball("Dead", [3.0, 6.0], BodyKind::Static);
    sim.world_mut().spawn((
        Name::new("Rope"),
        PhysicsJoint {
            body_a: stable_name_id("Load"),
            body_b: stable_name_id("Dead"),
            kind: JointKind::Pulley,
            ..PhysicsJoint::of_kind(JointKind::Pulley)
        },
        Transform::from_translation(Vec2::new(0.0, 2.0)),
    ));
    sim.world_mut().spawn((
        Name::new("Sheave"),
        PulleyWheel {
            rope: stable_name_id("Rope"),
            order: 0,
            radius: R,
            ..Default::default()
        },
        Transform::from_translation(Vec2::new(0.0, 6.0)),
    ));
    let mut bridge = PhysicsBridge::new();
    ph2d_physics_ecs::resolve_body_names(sim.world_mut());
    bridge.dispatch(&mut sim, false, 0);
    let named = |sim: &mut SimWorld, n: &str| {
        let mut q = sim.world_mut().query::<(Entity, &Name)>();
        q.iter(sim.world())
            .find(|(_, x)| x.as_str() == n)
            .map(|(e, _)| e)
            .expect("entidade viva")
    };
    let rope = named(&mut sim, "Rope");
    let wheel = named(&mut sim, "Sheave");
    (sim, bridge, rope, wheel)
}

/// **A ponta A tem perna, e a marca de zero pousa no ponto de TANGÊNCIA.**
///
/// É o que faz um limitador desligado ficar encostado na roldana — onde a corda
/// de fato já podia chegar — e o que dá ao gesto um lugar de partida sem painel e
/// sem armar nada.
#[test]
fn the_zero_mark_sits_where_the_rope_touches_the_wheel() {
    let (_sim, bridge, rope, _) = rig();
    let leg = bridge.rope_stop_legs(rope)[0].expect("a ponta A tem roldana");
    let mark = stop_mark(&leg, 0.0);
    assert!(
        (mark[0] - leg.touch[0]).abs() < 1e-5 && (mark[1] - leg.touch[1]).abs() < 1e-5,
        "a marca de zero saiu em {mark:?}, e a tangência é {:?}",
        leg.touch
    );
    // E a folga que ela mede é a de TANGENTE, não a distância ao centro: com o
    // eixo 4 m acima e raio 1, `√(16 − 1) = 3,873`.
    assert!(
        (leg.len - 15.0f32.sqrt()).abs() < 1e-3,
        "a folga de tangente saiu {} e a geometria diz {}",
        leg.len,
        15.0f32.sqrt()
    );
}

/// **A marca ANDA na corda, e a ida e a volta batem** — a lei do seed==sample na
/// fronteira que a UI de fato usa.
#[test]
fn dragging_the_mark_along_the_rope_authors_the_number() {
    let (_sim, bridge, rope, _) = rig();
    let leg = bridge.rope_stop_legs(rope)[0].expect("a ponta A tem roldana");
    for s in [0.0f32, 1.0, 2.5, leg.len] {
        let back = stop_at_point(&leg, stop_mark(&leg, s));
        assert!((back - s).abs() < 1e-3, "{s} voltou como {back}");
    }
}

/// **O número autorado CHEGA ao solver** — provado pelo COMPORTAMENTO, e não por
/// ler a tabela: uma alça que autora um número que o solver nunca lê é um gesto
/// bonito e morto.
///
/// A roldana ganha motor (é um guincho), então a carga sobe. Sem limitador ela
/// entra na roldana; com ele, para.
#[test]
fn the_authored_stop_reaches_the_solver_and_the_load_stops() {
    let gap = |sim: &SimWorld, e: Entity| -> f32 {
        let t = sim.world().get::<Transform>(e).expect("pose");
        let d = t.translation.x.hypot(t.translation.y - 6.0);
        (d * d - R * R).max(0.0).sqrt()
    };
    let run = |stop: f32| -> f32 {
        let (mut sim, mut bridge, rope, wheel) = rig();
        if let Some(mut w) = sim.world_mut().get_mut::<PulleyWheel>(wheel) {
            w.motor_speed = 0.5; // rate = ω·r = 0,5 m/s
        }
        if stop > 0.0 {
            sim.world_mut()
                .entity_mut(rope)
                .insert(RopeStops { a: stop, b: 0.0 });
        }
        let load = {
            let mut q = sim.world_mut().query::<(Entity, &Name)>();
            q.iter(sim.world())
                .find(|(_, n)| n.as_str() == "Load")
                .map(|(e, _)| e)
                .expect("a carga")
        };
        let mut lowest = f32::INFINITY;
        for tick in 1..600u64 {
            bridge.dispatch(&mut sim, true, tick);
            lowest = lowest.min(gap(&sim, load));
        }
        lowest
    };
    let free = run(0.0);
    assert!(
        free < 0.05,
        "o CONTROLE tem de encostar na roldana (folga mínima {free:.4} m)"
    );
    let held = run(1.5);
    assert!(
        held > 1.4,
        "o limitador de 1,5 m autorado no ECS não chegou ao solver (folga mínima {held:.4} m)"
    );
}

/// **O clique acha a roldana pelo ARO e pelo CUBO, nunca pelo DISCO.**
///
/// ⚠️ Reclamar o disco faria uma roldana grande engolir o clique de tudo o que
/// ela emoldura — e no rig do Enio a corda passa por dentro de três delas.
#[test]
fn the_click_finds_the_wheel_by_its_ring_and_hub_not_its_disc() {
    let (_sim, bridge, _, wheel) = rig();
    let c = [0.0, 6.0];
    let tol = 0.15;
    // O cubo.
    assert_eq!(bridge.wheel_at_world(c, tol), Some(wheel));
    // O aro, pelos quatro lados.
    for d in [[R, 0.0], [-R, 0.0], [0.0, R], [0.0, -R]] {
        assert_eq!(
            bridge.wheel_at_world([c[0] + d[0], c[1] + d[1]], tol),
            Some(wheel),
            "o aro em {d:?} não respondeu"
        );
    }
    // **O MIOLO do disco não é alvo** — é o controle desta lei.
    assert_eq!(
        bridge.wheel_at_world([c[0] + R * 0.5, c[1]], tol),
        None,
        "o meio do disco reclamou o clique"
    );
    // E longe também não.
    assert_eq!(bridge.wheel_at_world([c[0] + R * 3.0, c[1]], tol), None);
}
