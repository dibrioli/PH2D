//! **LIFT MOMENTUM** (W10) — os gates de COMPORTAMENTO, com o rapier de verdade.
//!
//! A doença que esta wave cura não é do solver: o corpo SEMPRE manteve a
//! velocidade que a plataforma lhe deu (isso é conservação de momento, e o
//! rapier a faz de graça). Quem a apagava era a **assistência**: o controle
//! aéreo mira `drive × speed` *relativo ao chão* ([`ph2d_platformer::walk`]), e
//! no ar o chão valia zero — então, no tique em que o pé sai de um vagão a
//! 4 m/s, o alvo salta para o referencial do MUNDO e o controle aéreo começa a
//! frear justamente o que a física acabou de dar.
//!
//! ⚠️ **O oráculo é o DESLOCAMENTO horizontal durante o voo**, não a velocidade
//! num instante: a velocidade oscila com o passo do solver, e o que o jogador vê
//! é o quanto ele avançou.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, LockRotation, MassOverride,
    PhysicsBridge, PlatformPlayer, PlayerInput, RigidBody,
};
use scene_fixture::{FLOAT_HEIGHT, pose};

/// A velocidade do vagão desta fixture.
const WAGON: f32 = 4.0;

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

/// Um vagão que anda a [`WAGON`] m/s e um personagem em pé nele.
///
/// ⚠️ O vagão é **dinâmico com gravidade zero e massa enorme**, e não cinemático:
/// um corpo cinemático é dirigido por uma pose por tique (o `SceneAtTick` da
/// timeline), e esta fixture não tem timeline. Com massa 1000 kg a reação do
/// personagem (a 3ª lei, W6) não o move de forma mensurável nos dois segundos
/// que o gate observa.
///
/// `moving = false` dá a MESMA cena com o vagão parado — o controle que separa
/// *"a memória funciona"* de *"a memória mudou alguma coisa"*.
fn rig(lift: f32, moving: bool) -> Rig {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Wagon"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        LockRotation,
        GravityScale(0.0),
        MassOverride(1000.0),
        InitialVelocity {
            linvel: [if moving { WAGON } else { 0.0 }, 0.0],
            angvel: 0.0,
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
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
                float_height: FLOAT_HEIGHT,
                lift_momentum: lift,
                // ⚠️ A quina fora do caminho: esta cena não tem teto, mas
                // desligá-la explicitamente é o que mantém o gate a medir UMA
                // wave de cada vez.
                corner_reach: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, 0.5 + FLOAT_HEIGHT)),
        ))
        .id();
    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

/// Anda alguns tiques em pé, pula, e devolve **o quanto o personagem avançou
/// horizontalmente durante o voo** — relativo ao chão de onde ele saiu.
fn ride_and_jump(r: &mut Rig) -> f32 {
    let mut tick = 0_u64;
    // Meio segundo em pé: a mola assenta e ele passa a viajar com o vagão.
    for _ in 0..30 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
    }
    let (x0, _) = pose(&r.sim);
    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
            dash: false,
        },
    );
    for k in 0..40 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
        if k == 2 {
            r.bridge.set_player_input(r.player, PlayerInput::default());
        }
    }
    let (x1, _) = pose(&r.sim);
    x1 - x0
}

/// **O gate da wave.** Pular de um vagão a 4 m/s não apaga os 4 m/s.
///
/// ## Medido (2026-08-04, vagão a 4 m/s, 40 tiques de voo, `drive = 0`)
///
/// | `lift_momentum` | avanço no voo |
/// |---|---|
/// | 0,00 | 0,291 m — **11%** do balístico |
/// | 0,50 | 2,291 m — 86% |
/// | 1,50 (o default) | 2,667 m — **100%** |
///
/// ⚠️ **O oráculo é a RAZÃO contra o voo balístico** (`4 m/s × 0,67 s ≈ 2,7 m`),
/// não um número absoluto: o que a assistência não pode fazer é FREAR o que a
/// física deu, e é isso que a comparação diz.
#[test]
fn jumping_off_a_wagon_keeps_the_wagons_speed() {
    let forgetful = ride_and_jump(&mut rig(0.0, true));
    let carried = ride_and_jump(&mut rig(1.5, true));

    assert!(
        carried > forgetful + 0.3,
        "com a memoria o personagem avanca MAIS: {carried:.3} vs {forgetful:.3}"
    );
    // E o avanço é da ordem do voo balístico — a assistência deixou de trabalhar
    // contra a velocidade herdada, em vez de a substituir por outra.
    let ballistic = WAGON * 40.0 / 60.0;
    assert!(
        carried > ballistic * 0.8,
        "o avanco tem de ser da ordem do voo balistico ({ballistic:.2} m): {carried:.3}"
    );
    assert!(
        forgetful < ballistic * 0.8,
        "controle: SEM a memoria o controle aereo freia ({ballistic:.2} m): {forgetful:.3}"
    );
}

/// ⚠️ **Em chão PARADO a memória é inerte** — e não por um guard, mas porque a
/// velocidade lembrada é `[0, 0]`.
///
/// É o que torna o default LIGADO honesto: ele não move um byte de nenhuma cena
/// que não tenha uma plataforma em movimento.
///
/// **Mutação que deve sangrar:** lembrar a velocidade do CORPO em vez da do
/// chão.
#[test]
fn on_still_ground_the_memory_changes_nothing() {
    let off = ride_and_jump(&mut rig(0.0, false));
    let on = ride_and_jump(&mut rig(1.5, false));
    assert_eq!(
        off, on,
        "com o vagao parado os dois voos tem de ser IDENTICOS: {off} vs {on}"
    );
}

/// **`lift_momentum = 0` é o mundo de antes desta wave**, medido no vagão em
/// movimento — onde a diferença existe.
#[test]
fn zero_window_is_the_world_before_this_wave() {
    let a = ride_and_jump(&mut rig(0.0, true));
    let b = ride_and_jump(&mut rig(0.0, true));
    assert_eq!(a, b, "a cena e' determinista");
}

/// **A SONDA da janela** — quanto cada valor entrega, contra o voo balístico.
#[test]
#[ignore = "sonda de medicao"]
fn measure_what_the_window_delivers() {
    let ballistic = WAGON * 40.0 / 60.0;
    println!("\n=== LIFT MOMENTUM: o avanco no voo (balistico = {ballistic:.3} m) ===");
    println!(
        "{:>10} | {:>10} | {:>8}",
        "janela (s)", "avanco (m)", "fracao"
    );
    for step in 0..=8 {
        let w = 0.25 * step as f32;
        let d = ride_and_jump(&mut rig(w, true));
        println!("{w:>10.2} | {d:>10.3} | {:>8.2}", d / ballistic);
    }
}

/// **Quanto tempo um pulo COMPLETO fica no ar** — o número de que a janela
/// default tem de ser função.
#[test]
#[ignore = "sonda de medicao"]
fn measure_how_long_a_default_jump_lasts() {
    let mut r = rig(0.0, false);
    let mut tick = 0_u64;
    for _ in 0..30 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
    }
    let (_, rest) = pose(&r.sim);
    // Segura o botao: altura CHEIA (sem o corte).
    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
            dash: false,
        },
    );
    let mut airborne = 0;
    let mut peak = rest;
    for _ in 0..300 {
        tick += 1;
        r.bridge.dispatch(&mut r.sim, true, tick);
        let (_, y) = pose(&r.sim);
        peak = peak.max(y);
        if y > rest + 0.02 {
            airborne += 1;
        } else if airborne > 0 {
            break;
        }
    }
    println!(
        "\n=== O PULO default: pico {:.3} m, {} tiques no ar = {:.2} s ===",
        peak - rest,
        airborne,
        airborne as f32 / 60.0
    );
}
