//! **SONDA — o que o modo CINEMÁTICO existe para curar, medido HOJE.**
//!
//! ⚠️ **Esta sonda existe porque o plano 07 envelheceu antes de ser executado.**
//! Ele nomeia dois defeitos do corpo dinâmico (a deriva de rampa e a penetração
//! no impacto) e cita `0,164 m` para o primeiro. A `W-Landing` levou o
//! amortecimento ao TETO no mesmo dia, e o teto sempre zerou a deriva — então o
//! número do plano descreve um produto que não existe mais.
//!
//! **A regra é a do `CLAUDE.md` §0 aplicada a mim mesmo:** *quem move o número
//! que justificava uma wave tem de reconferir a nota antes de a construir.*
//!
//! Rodar: `cargo test -p ph2d-physics-ecs --release --test measure_kinematic_case -- --ignored --nocapture`

#[path = "platform_scene.rs"]
mod platform;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::RideConfig;

use platform::{FLOAT_HEIGHT, pose};

/// Quanto o personagem PARADO viaja numa rampa de `deg` graus em `secs`, com o
/// amortecimento em `damping`.
fn idle_travel(deg: f32, secs: u64, damping: f32) -> f32 {
    let slope = deg.to_radians();
    let (mut sim, mut bridge, who) = platform::scene(slope, 0.0);
    {
        let mut e = sim.world_mut().entity_mut(who);
        let mut p = e.get_mut::<PlatformPlayer>().expect("player");
        p.spring_damping = damping;
    }
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let (x0, _) = pose(&sim);
    for t in 121..=(120 + secs * 60) {
        bridge.dispatch(&mut sim, true, t);
    }
    (pose(&sim).0 - x0).abs()
}

/// **A DERIVA DE RAMPA, hoje** — e a tabela diz se o plano ainda descreve o
/// produto.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_ramp_creep_today() {
    let ceiling = RideConfig::MAX_DAMPING;
    println!("\n=== DERIVA DE RAMPA (parado 10 s, corpo DINAMICO) ===");
    println!("{:<34} {:>10}", "amortecimento", "viajou (m)");
    for (label, d) in [
        (
            "o DEFAULT que shipa",
            RideConfig::STARTING_POINT.spring_damping,
        ),
        ("o teto", ceiling),
        ("meio curso", 0.5 * ceiling),
        ("um quarto", 0.25 * ceiling),
    ] {
        println!("{label:<34} {:>10.4}", idle_travel(30.0, 10, d));
    }
    println!(
        "\n  (o plano 07 cita 0,164 m para o default; o default de hoje e' {:.2} = o teto)",
        RideConfig::STARTING_POINT.spring_damping
    );
}

/// **A PENETRAÇÃO no impacto, hoje** — o segundo defeito que o plano nomeia.
///
/// ⚠️ O oráculo é a distância do PÉ ao topo do chão: um personagem que paira a
/// `float_height` tem de manter essa folga, e o que a queda lhe tira é o número
/// que o modo cinemático promete zerar.
#[test]
#[ignore = "sonda de medição"]
fn measure_the_impact_penetration_today() {
    println!("\n=== PENETRACAO NO IMPACTO (corpo DINAMICO) ===");
    println!(
        "{:>10} {:>9} {:>14} {:>14}",
        "queda (m)", "damping", "pior folga", "perdido (mm)"
    );
    // ⚠️ **A coluna do amortecimento é o CONTROLE, e sem ela a tabela mente por
    // vácuo:** no teto o boost mata a velocidade relativa inteira em UM tique, e
    // o personagem é apanhado no instante em que o raio o vê. Uma tabela só do
    // default diria *"não há penetração"* quando o que ela mede é *"o knob está
    // no teto"* — e é essa distinção que decide se o modo cinemático compra algo.
    let ceiling = RideConfig::MAX_DAMPING;
    for (drop, damping) in [0.5_f32, 2.0, 5.0, 10.0]
        .into_iter()
        .flat_map(|d| [(d, ceiling), (d, 0.25 * ceiling)])
    {
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
                float_height: FLOAT_HEIGHT,
                spring_damping: damping,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT_HEIGHT + drop)),
        ));
        let mut bridge = PhysicsBridge::new();
        let mut worst = f32::INFINITY;
        for t in 1..=600u64 {
            bridge.dispatch(&mut sim, true, t);
            worst = worst.min(pose(&sim).1);
        }
        println!(
            "{drop:>10.1} {damping:>9.2} {worst:>14.4} {:>14.1}",
            (FLOAT_HEIGHT - worst) * 1000.0
        );
    }
}
