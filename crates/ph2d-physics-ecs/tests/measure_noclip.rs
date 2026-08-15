//! **O QUE JÁ EXISTE NO LUGAR DO NOCLIP** — a sonda que decide o item **H** da
//! fila (`MOVE_Flying` / noclip).
//!
//! A auditoria dá ao item um veredito de **valor baixo** com uma razão escrita:
//! *"útil para percorrer um nível grande no editor, valor de produto baixo num
//! editor que já tem câmera livre"*. A §0 manda medir a premissa antes de a
//! aceitar OU de a recusar, e a premissa tem duas metades:
//!
//! 1. **o artista já consegue pôr o personagem em qualquer lugar?** — inclusive
//!    DENTRO de geometria, que é o que *noclip* significa;
//! 2. **e a simulação retoma de onde ele o largou?** — porque um teleporte que a
//!    física desfaz no primeiro tique não é uma forma de chegar a lado nenhum.
//!
//! ⚠️ **A fixture nasceu ERRADA e a primeira tabela MENTIU.** O gesto do toggle
//! **Physics** desmarcado é [`PhysicsBridge::hold`], e **não**
//! `dispatch(playing = false, …)`: aquela porta, com o alvo a CRESCER, entra no
//! braço `Greater` e **DÁ PASSO** — o doc dela chama-lhe *"um scrub para a FRENTE
//! enquanto pausado"*, porque o estado da sim é função do TIQUE e não do botão de
//! play. Com ela a pose escrita à mão era devolvida pelo `readback` do passo
//! seguinte, e a tabela dizia *"o artista não consegue pô-lo em lado nenhum"*
//! sobre um produto em que ele consegue.
//!
//! Rodar:
//! `ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_noclip -- --ignored --nocapture --test-threads=1`

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

const FLOAT: f32 = 0.9;

/// Uma cena com o personagem, um chão e uma PAREDE grossa em `x ∈ [4, 8]`.
fn scene() -> (SimWorld, ph2d_ecs::Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 2.0,
                half_y: 4.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(6.0, 4.0)),
    ));
    let p = sim
        .world_mut()
        .spawn((
            Name::new("Player"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Capsule {
                    half_height: 0.3,
                    radius: 0.2,
                },
                ..Collider::default()
            },
            LockRotation,
            PlatformPlayer {
                float_height: FLOAT,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
        ))
        .id();
    (sim, p)
}

#[test]
#[ignore = "sonda"]
fn measure_whether_the_artist_can_already_place_him_anywhere() {
    let (mut sim, p) = scene();
    let mut bridge = PhysicsBridge::new();
    for i in 1..=30u64 {
        bridge.dispatch(&mut sim, true, i);
    }
    println!("\n=== O ARTISTA JA' CONSEGUE PO-LO ONDE QUISER? ===");
    println!("  (parede SOLIDA em x [4, 8], y [0, 8]; o toggle Physics DESMARCADO)\n");

    // O gesto que o artista faz: desmarca Physics e arrasta o `Transform`.
    let mut tick = 30u64;
    for (name, at) in [
        ("no meio da PAREDE", Vec2::new(6.0, 4.0)),
        ("do outro LADO dela", Vec2::new(12.0, FLOAT)),
        ("bem la' em CIMA", Vec2::new(6.0, 20.0)),
    ] {
        sim.world_mut()
            .get_mut::<Transform>(p)
            .expect("transform")
            .translation = at;
        for _ in 0..10 {
            tick += 1;
            bridge.hold(&mut sim, tick);
        }
        let t = sim.world().get::<Transform>(p).expect("transform");
        println!(
            "  {name:<20} pedido ({:>5.1}, {:>5.1})  ->  ficou ({:>7.4}, {:>7.4})",
            at.x, at.y, t.translation.x, t.translation.y
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_whether_the_sim_resumes_from_where_he_was_left() {
    let (mut sim, p) = scene();
    let mut bridge = PhysicsBridge::new();
    for i in 1..=30u64 {
        bridge.dispatch(&mut sim, true, i);
    }
    // Largado do outro lado da parede, com o toggle Physics desmarcado.
    sim.world_mut()
        .get_mut::<Transform>(p)
        .expect("transform")
        .translation = Vec2::new(12.0, 6.0);
    for i in 31..=40u64 {
        bridge.hold(&mut sim, i);
    }
    let left_at = sim
        .world()
        .get::<Transform>(p)
        .expect("transform")
        .translation;
    // E o Play: a sim retoma DALI?
    for i in 41..=120u64 {
        bridge.dispatch(&mut sim, true, i);
    }
    let after = sim
        .world()
        .get::<Transform>(p)
        .expect("transform")
        .translation;
    println!("\n=== E A SIM RETOMA DALI? ===");
    println!(
        "  largado em ({:.4}, {:.4})  ->  depois de 80 tiques de Play: ({:.4}, {:.4})",
        left_at.x, left_at.y, after.x, after.y
    );
    println!(
        "  (deriva lateral {:.4} m -- ele cai NO LUGAR, do outro lado da parede)",
        (after.x - left_at.x).abs()
    );
}

/// ⚠️ **O CONTROLE:** sem ele as duas tabelas acima podiam descrever um mundo em
/// que a parede não existe, e *"ele atravessou"* seria verdade por vácuo.
#[test]
#[ignore = "sonda"]
fn measure_that_the_wall_is_really_solid() {
    let (mut sim, p) = scene();
    let mut bridge = PhysicsBridge::new();
    for i in 1..=30u64 {
        bridge.dispatch(&mut sim, true, i);
    }
    // Empurrado contra a parede com o relogio a andar: ele TEM de ser barrado.
    for i in 31..=240u64 {
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
    }
    let x = sim
        .world()
        .get::<Transform>(p)
        .expect("transform")
        .translation
        .x;
    println!("\n=== CONTROLE: a parede e' SOLIDA ===");
    println!("  correndo para a direita 3,5 s, ele parou em x = {x:.4} (a parede comeca em 4,0)");
    assert!(
        x < 4.0,
        "se ele passou de x=4 a parede nao esta' solida e as tabelas acima nao dizem nada"
    );
}
