//! **O CONTROLE AÉREO PARADO** — a sonda que abre os itens **H** e **I** da fila.
//!
//! A auditoria descreve o **I** (`AirControlBoostMultiplier`) por um SINTOMA —
//! *"tira a sensação de 'não consigo sair do lugar' no topo de um pulo
//! vertical"* — e a §0 manda medir o fenómeno antes de escrever a cura. As
//! perguntas, por esta ordem:
//!
//! 1. **o sintoma existe no NOSSO produto?** Um pulo vertical parado, com o
//!    direcional apertado à saída: quanto ele anda de lado até aterrar?
//! 2. **ele é sobre o ARRANQUE ou sobre o teto?** Se a lateral final já é a
//!    velocidade de cruzeiro, não há nada a acelerar — o número que falta seria
//!    o do tempo de voo, não o do controle.
//! 3. **quanto é que um multiplicador compraria?** — a resposta que decide se a
//!    wave é uma feature ou uma nota.
//!
//! Rodar:
//! `ph2d-run cargo test -p ph2d-physics-ecs --release --test measure_air_control -- --ignored --nocapture`

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

const FLOAT: f32 = 0.9;

/// O que um pulo devolve.
struct Jump {
    /// Deslocamento lateral total, do salto até aterrar.
    drift: f32,
    /// A velocidade lateral no ápice — *"quão preso ele está lá em cima"*.
    apex_speed: f32,
    /// Quantos tiques ele passou no ar.
    airborne: u64,
    /// A altura máxima acima do repouso.
    peak: f32,
}

/// Pula parado e segura o direcional a partir do tique `hold_from`.
///
/// ⚠️ **O `hold_from` é o parâmetro que separa as duas perguntas:** segurar
/// desde o chão mede *o arranque no ar mais o que o chão já deu*; segurar só
/// depois de sair mede o controle aéreo SOZINHO, que é o que o sintoma descreve.
fn vertical_jump(air_accel: f32, hold_from: u64) -> Jump {
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
    let player = sim
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
                air_acceleration: air_accel,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, FLOAT)),
        ))
        .id();

    let mut bridge = PhysicsBridge::new();
    // Assenta.
    for i in 1..=30u64 {
        bridge.set_player_input(player, PlayerInput::default());
        bridge.dispatch(&mut sim, true, i);
    }
    let x0 = sim
        .world()
        .get::<Transform>(player)
        .expect("transform")
        .translation
        .x;

    let mut peak = 0.0f32;
    let mut apex_speed = 0.0f32;
    let mut airborne = 0u64;
    let mut prev_x = x0;
    let mut rising = true;
    for i in 31..=240u64 {
        // Segura o pulo os primeiros tiques (altura cheia), depois o direcional.
        let t = i - 30;
        bridge.set_player_input(
            player,
            PlayerInput {
                jump: t <= 20,
                drive: if t >= hold_from { 1.0 } else { 0.0 },
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, i);
        let tr = sim.world().get::<Transform>(player).expect("transform");
        let h = tr.translation.y - FLOAT;
        if h > 0.05 {
            airborne += 1;
        }
        if h > peak {
            peak = h;
            // No ápice, a velocidade lateral é a diferença do último tique.
            apex_speed = (tr.translation.x - prev_x).abs() * 60.0;
        } else if rising && h < peak - 0.01 {
            rising = false;
        }
        prev_x = tr.translation.x;
        // Aterrou depois de ter subido: acabou.
        if !rising && h < 0.02 && airborne > 5 {
            break;
        }
    }
    let x1 = sim
        .world()
        .get::<Transform>(player)
        .expect("transform")
        .translation
        .x;
    Jump {
        drift: x1 - x0,
        apex_speed,
        airborne,
        peak,
    }
}

#[test]
#[ignore = "sonda"]
fn measure_whether_the_symptom_exists_here() {
    println!("\n=== O SINTOMA EXISTE? (pulo vertical parado, direcional apertado) ===");
    let cfg = PlatformPlayer::default();
    println!(
        "  config de partida: air_acceleration = {}, speed = {}\n",
        cfg.air_acceleration, cfg.speed
    );
    println!("  segura a partir do tique   deriva (m)   vel no apice   tiques no ar   pico (m)");
    for hold in [1u64, 5, 10, 20] {
        let j = vertical_jump(cfg.air_acceleration, hold);
        println!(
            "  {hold:>24}   {:>10.4}   {:>12.4}   {:>12}   {:>8.4}",
            j.drift, j.apex_speed, j.airborne, j.peak
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_what_a_boost_would_buy() {
    println!("\n=== E O QUE UM MULTIPLICADOR COMPRARIA? ===");
    println!("  (o `AirControlBoostMultiplier` do Unreal nasce em 2)\n");
    let base = PlatformPlayer::default().air_acceleration;
    println!("  air_accel   x base   deriva (m)   vel no apice");
    for mult in [0.5f32, 1.0, 2.0, 4.0, 8.0] {
        let j = vertical_jump(base * mult, 1);
        println!(
            "  {:>9.1}   {mult:>6.1}   {:>10.4}   {:>12.4}",
            base * mult,
            j.drift,
            j.apex_speed
        );
    }
}

#[test]
#[ignore = "sonda"]
fn measure_that_he_jumps_at_all() {
    // ⚠️ O CONTROLE: sem ele as tabelas acima podiam estar a medir um personagem
    // que nunca saiu do chão, e toda a deriva seria zero por vácuo.
    let j = vertical_jump(PlatformPlayer::default().air_acceleration, 1);
    println!(
        "\n=== CONTROLE: ele PULA ===\n  pico {:.4} m, {} tiques no ar, deriva {:.4} m",
        j.peak, j.airborne, j.drift
    );
    assert!(j.peak > 0.5, "ele tem de subir, senao a sonda mede o nada");
    assert!(j.airborne > 20, "e tem de ficar no ar tempo de manobrar");
}

#[test]
#[ignore = "sonda"]
fn measure_where_the_symptom_would_live() {
    // ⚠️ A tabela acima diz que o sintoma nao existe na config de partida. Esta
    // pergunta a metade que decide se o item e' uma FEATURE ou uma NOTA: existe
    // ALGUMA config em que ele exista? E se existe, qual e' o knob que a cura?
    println!("\n=== E COM O CONTROLE AEREO FRACO? ===");
    println!("  (o `AirControl` do Unreal e' uma FRACAO da velocidade -- 5% por");
    println!("   default -- e e' esse regime que o boost dele resgata)\n");
    println!("  air_accel   deriva (m)   vel no apice   fracao do cruzeiro");
    for accel in [0.5f32, 1.0, 2.0, 5.0, 10.0, 20.0] {
        let j = vertical_jump(accel, 1);
        println!(
            "  {accel:>9.1}   {:>10.4}   {:>12.4}   {:>17.0}%",
            j.drift,
            j.apex_speed,
            100.0 * j.apex_speed / 6.0
        );
    }
}
