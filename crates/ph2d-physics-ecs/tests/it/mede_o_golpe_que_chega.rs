//! **SONDA da W2 da VIDA E DANO** (plano 28 §4 — *«abre com a medição do `Began` de um corpo que
//! nasce sobreposto»*). Imprime a tabela E afirma o que ela mediu (2026-09-23).
//!
//! ⭐ **O que ela decidiu no desenho** (plano 28 §8): nascer sobreposto NÃO perde o golpe (A–D, tudo
//! no 1.º tique), mas ⛔ **uma bala nunca ENCOSTA no alvo** (E, F): o projéctil é um mover que pára
//! rente ao obstáculo, logo o solver não reporta toque nenhum e o alvo nem se mexe. ⇒ a vida lê
//! também **o que o próprio mover bateu**, não só os contactos.
//!
//! ⛔ E a G achou um defeito do #14: um corpo **só-sensor** não anda com mover nenhum
//! (`move_character_from` devolve `none` quando não há forma SÓLIDA), logo uma bala-sensor fica
//! parada onde nasceu, em silêncio.
//!
//! Cada caso responde a UMA pergunta sobre os canais que a ponte já tem, pela API pública:
//! o golpe chega no 1.º tique, a meio da corrida, e numa bala rápida?

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ContactPhase, GravityScale, PhysicsBridge, ProjectileMotion,
    RigidBody,
};

fn caixa(hx: f32, hy: f32, sensor: bool) -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: hx,
            half_y: hy,
        },
        density: 1.0,
        is_sensor: sensor,
        ..Collider::default()
    }
}

fn corpo(sim: &mut SimWorld, nome: &str, kind: BodyKind, col: Collider, x: f32, y: f32) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new(nome),
            RigidBody { kind },
            col,
            GravityScale(0.0),
            Transform::from_translation(Vec2::new(x, y)),
        ))
        .id()
}

/// Quantos `Began` de contacto sólido e quantas entradas em sensor viu cada tique, entre `a` e `b`.
fn corre(sim: &mut SimWorld, a: Entity, b: Entity, ticks: u64) -> (Vec<u64>, Vec<u64>) {
    let mut ponte = PhysicsBridge::new();
    let mut solidos = Vec::new();
    let mut sensores = Vec::new();
    for t in 0..=ticks {
        ponte.dispatch(sim, true, t);
        let par = |x: Entity, y: Entity| (x == a && y == b) || (x == b && y == a);
        if ponte
            .contact_events()
            .iter()
            .any(|e| e.phase == ContactPhase::Began && par(e.a, e.b))
        {
            solidos.push(t);
        }
        if ponte
            .trigger_events()
            .iter()
            .any(|e| par(e.sensor, e.other))
        {
            sensores.push(t);
        }
    }
    (solidos, sensores)
}

#[test]
fn sonda_o_golpe_que_chega() {
    eprintln!("\n=== A. dois DINÂMICOS sólidos que nascem sobrepostos ===");
    let mut sim = SimWorld::new();
    let a = corpo(
        &mut sim,
        "A",
        BodyKind::Dynamic,
        caixa(0.5, 0.5, false),
        0.0,
        0.0,
    );
    let b = corpo(
        &mut sim,
        "B",
        BodyKind::Dynamic,
        caixa(0.5, 0.5, false),
        0.3,
        0.0,
    );
    let (s, t) = corre(&mut sim, a, b, 5);
    eprintln!("  Began sólido nos tiques {s:?} · sensor {t:?}");
    assert_eq!(
        s,
        vec![1],
        "A: o toque de quem nasce sobreposto chega no 1.º tique"
    );

    eprintln!("\n=== B. SENSOR estático a nascer sobre um CINEMÁTICO ===");
    let mut sim = SimWorld::new();
    let a = corpo(
        &mut sim,
        "S",
        BodyKind::Static,
        caixa(0.5, 0.5, true),
        0.0,
        0.0,
    );
    let b = corpo(
        &mut sim,
        "K",
        BodyKind::Kinematic,
        caixa(0.3, 0.3, false),
        0.2,
        0.0,
    );
    let (s, t) = corre(&mut sim, a, b, 5);
    eprintln!("  Began sólido {s:?} · entrada no sensor nos tiques {t:?}");
    assert_eq!(
        t,
        vec![1],
        "B: o sensor vê quem nasce dentro dele no 1.º tique"
    );

    eprintln!("\n=== C. SENSOR cinemático a nascer sobre um ESTÁTICO sólido ===");
    let mut sim = SimWorld::new();
    let a = corpo(
        &mut sim,
        "S",
        BodyKind::Kinematic,
        caixa(0.3, 0.3, true),
        0.0,
        0.0,
    );
    let b = corpo(
        &mut sim,
        "W",
        BodyKind::Static,
        caixa(0.5, 0.5, false),
        0.2,
        0.0,
    );
    let (s, t) = corre(&mut sim, a, b, 5);
    eprintln!("  Began sólido {s:?} · entrada no sensor nos tiques {t:?}");
    assert_eq!(
        t,
        vec![1],
        "C: um sensor cinemático vê um estático sólido no 1.º tique"
    );

    eprintln!("\n=== D. corpo que NASCE A MEIO da corrida dentro de um sensor ===");
    let mut sim = SimWorld::new();
    let s_ = corpo(
        &mut sim,
        "S",
        BodyKind::Static,
        caixa(0.5, 0.5, true),
        0.0,
        0.0,
    );
    let mut ponte = PhysicsBridge::new();
    let mut viu = Vec::new();
    let mut k = None;
    for t in 0..=40 {
        if t == 30 {
            k = Some(corpo(
                &mut sim,
                "K",
                BodyKind::Kinematic,
                caixa(0.3, 0.3, false),
                0.1,
                0.0,
            ));
        }
        ponte.dispatch(&mut sim, true, t);
        if let Some(k) = k
            && ponte
                .trigger_events()
                .iter()
                .any(|e| e.sensor == s_ && e.other == k)
        {
            viu.push(t);
        }
    }
    eprintln!("  nasceu no dispatch 30 · entrada no sensor nos tiques {viu:?}");
    assert_eq!(
        viu,
        vec![30],
        "D: quem nasce a meio da corrida é visto no dispatch em que nasce"
    );

    eprintln!("\n=== E. PROJÉCTIL (cinemático, sólido) contra um DINÂMICO ===");
    eprintln!("  vel(m/s) | Began sólido | fim do voo      | alvo andou(m)");
    for v in [6.0_f32, 30.0, 60.0, 120.0] {
        let mut sim = SimWorld::new();
        let bala = sim
            .world_mut()
            .spawn((
                Name::new("Bala"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                caixa(0.1, 0.1, false),
                GravityScale(0.0),
                Transform::from_translation(Vec2::new(-3.0, 0.0)),
                ProjectileMotion {
                    initial_speed: v,
                    range: 0.0,
                    ..ProjectileMotion::default()
                },
            ))
            .id();
        let alvo = corpo(
            &mut sim,
            "Alvo",
            BodyKind::Dynamic,
            caixa(0.4, 0.4, false),
            0.0,
            0.0,
        );
        let mut ponte = PhysicsBridge::new();
        let mut began = Vec::new();
        let mut fim = None;
        for t in 0..=90 {
            ponte.dispatch(&mut sim, true, t);
            if ponte.contact_events().iter().any(|e| {
                e.phase == ContactPhase::Began
                    && ((e.a == bala && e.b == alvo) || (e.a == alvo && e.b == bala))
            }) {
                began.push(t);
            }
            if fim.is_none()
                && let Some((_, porque)) = ponte.projectile_done().iter().find(|(e, _)| *e == bala)
            {
                fim = Some((t, *porque));
            }
        }
        let x = sim
            .world()
            .get::<Transform>(alvo)
            .map_or(0.0, |t| t.translation.x);
        eprintln!("  {v:8.1} | {began:?} | {fim:?} | {x:.4}");
        // ⛔ O achado que muda o desenho: a bala pára rente ao alvo e o solver não vê toque.
        assert!(
            began.is_empty(),
            "E: a bala passou a encostar no alvo ({began:?}) — reler §8"
        );
        assert!(fim.is_some(), "E: o voo tem de acabar ao bater");
    }

    eprintln!("\n=== F. PROJÉCTIL contra uma parede ESTÁTICA sólida ===");
    let mut sim = SimWorld::new();
    let bala = sim
        .world_mut()
        .spawn((
            Name::new("Bala"),
            RigidBody {
                kind: BodyKind::Kinematic,
            },
            caixa(0.1, 0.1, false),
            GravityScale(0.0),
            Transform::from_translation(Vec2::new(-3.0, 0.0)),
            ProjectileMotion {
                initial_speed: 30.0,
                range: 0.0,
                ..ProjectileMotion::default()
            },
        ))
        .id();
    let muro = corpo(
        &mut sim,
        "Muro",
        BodyKind::Static,
        caixa(0.4, 0.4, false),
        0.0,
        0.0,
    );
    let (s, t) = corre(&mut sim, bala, muro, 60);
    eprintln!("  Began sólido {s:?} · sensor {t:?}");
    assert!(
        s.is_empty() && t.is_empty(),
        "F: cinemático contra estático não gera toque"
    );

    eprintln!("\n=== G. BALA-SENSOR (cinemática) a atravessar um alvo CINEMÁTICO de 0,8 m ===");
    eprintln!("  vel(m/s) | m/tique | entrada no sensor | a bala passou? x final");
    for v in [6.0_f32, 30.0, 60.0, 120.0, 240.0] {
        let mut sim = SimWorld::new();
        let bala = sim
            .world_mut()
            .spawn((
                Name::new("Bala"),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                caixa(0.1, 0.1, true),
                GravityScale(0.0),
                Transform::from_translation(Vec2::new(-3.0, 0.0)),
                ProjectileMotion {
                    initial_speed: v,
                    range: 0.0,
                    ..ProjectileMotion::default()
                },
            ))
            .id();
        let alvo = corpo(
            &mut sim,
            "Alvo",
            BodyKind::Kinematic,
            caixa(0.4, 0.4, false),
            0.0,
            0.0,
        );
        let (s, t) = corre(&mut sim, bala, alvo, 90);
        let x = sim
            .world()
            .get::<Transform>(bala)
            .map_or(0.0, |t| t.translation.x);
        eprintln!(
            "  {v:8.1} | {:7.3} | {t:?} (sólido {s:?}) | {x:.3}",
            v / 60.0
        );
    }
}
