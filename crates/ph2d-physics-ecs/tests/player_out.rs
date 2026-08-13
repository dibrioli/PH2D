//! **A SAÍDA do player** — o readout contínuo e o canal de transições
//! (`W-PlayerOut`, A1+A2).
//!
//! ⚠️ **O gate que carrega a wave é o
//! [`a_dispatch_that_owes_three_ticks_delivers_the_events_of_the_middle_ones`]**:
//! ele é o único que separa *o evento nasce por TIQUE* de *o evento nasce por
//! QUADRO*, e as duas leituras são indistinguíveis em todos os outros gates
//! deste arquivo, porque todos eles dão um tique por dispatch.

#[path = "platform_scene.rs"]
mod scene;

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, PhysicsBridge, PlatformPlayer, RigidBody,
};
use ph2d_platformer::{DashConfig, FootingKind, JumpKind, PlayerEvent, PlayerInput};
use scene::{FLOAT_HEIGHT, scene};

/// Um toque no botão de pulo — um tique preso, um solto: a lei lê a BORDA.
fn tap(sim: &mut SimWorld, bridge: &mut PhysicsBridge, p: Entity, tick: &mut u64) {
    for held in [true, false] {
        bridge.set_player_input(
            p,
            PlayerInput {
                jump: held,
                ..PlayerInput::default()
            },
        );
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
    }
}

/// `n` tiques sem entrada nenhuma, devolvendo todos os eventos vistos pelo
/// caminho (a lista é por DISPATCH, então quem quer a corrida inteira acumula).
fn coast(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    p: Entity,
    tick: &mut u64,
    n: u64,
) -> Vec<PlayerEvent> {
    let mut seen = Vec::new();
    for _ in 0..n {
        bridge.set_player_input(p, PlayerInput::default());
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
        seen.extend(bridge.player_events().iter().map(|(_, e)| *e));
    }
    seen
}

// ── O READOUT ────────────────────────────────────────────────────────────────

/// **O readout diz `Ground` só quando a LEI diz `Ground`** — e diz **`Steep`**
/// numa rampa que ela recusa.
///
/// ⚠️ **É o gate que impede o re-colapso.** A W9 des-colapsou *no ar* de
/// *encostado numa encosta íngreme demais* porque as duas pedem coisas OPOSTAS
/// da caminhada; um readout com um `grounded: bool` juntaria outra vez o que ela
/// separou, e o consumidor não teria como voltar a distinguir.
#[test]
fn the_readout_says_ground_only_when_the_law_says_ground() {
    // Plano: apoiado.
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    for t in 1..=30u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(
        bridge.player_view(p).map(|v| v.footing),
        Some(FootingKind::Ground),
        "num plano a lei apoia"
    );

    // Uma rampa que a lei RECUSA (o default aceita até 45°).
    let steep = PlatformPlayer::default().max_slope_deg.to_radians() + 0.25;
    let (mut sim, mut bridge, p) = scene(steep, 0.0);
    let mut saw_steep = false;
    for t in 1..=90u64 {
        bridge.dispatch(&mut sim, true, t);
        if bridge.player_view(p).map(|v| v.footing) == Some(FootingKind::Steep) {
            saw_steep = true;
        }
    }
    assert!(
        saw_steep,
        "escorregando por uma encosta recusada o readout tem de dizer Steep, \
         nunca Ground nem Airborne"
    );
}

/// **O `facing` segue o eixo com o ARRANQUE DESLIGADO.**
///
/// ⚠️ O campo morava dentro do `DashState`, e este gate é o que prova que a
/// mudança de casa não foi cosmética: com o arranque desarmado — o que shipa —
/// ele continua a ser mantido, porque quem o ESCREVE é a caminhada.
#[test]
fn facing_follows_the_axis_with_the_dash_disarmed() {
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    assert!(
        !DashConfig::STARTING_POINT.armed(),
        "o ponto de partida do arranque tem de estar DESLIGADO, senão este \
         gate não prova nada"
    );

    for t in 1..=20u64 {
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: -1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(bridge.player_view(p).map(|v| v.facing), Some(-1.0));

    for t in 21..=40u64 {
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
    }
    assert_eq!(bridge.player_view(p).map(|v| v.facing), Some(1.0));

    // ⚠️ E um eixo NEUTRO não o apaga: parar de andar não é virar-se para lugar
    // nenhum.
    let _ = coast(&mut sim, &mut bridge, p, &mut 40, 10);
    assert_eq!(bridge.player_view(p).map(|v| v.facing), Some(1.0));
}

/// **Uma descontinuidade apaga o readout**, em vez de o deixar a descrever uma
/// corrida que acabou.
///
/// É a mesma lei dos contatos e das marcas dos sensores: sem passo não há lei, e
/// publicar *"no chão, a 4 m/s"* com a física desarmada é um número errado
/// apresentado como certo.
#[test]
fn a_discontinuity_clears_the_readout() {
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    for t in 1..=30u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    assert!(bridge.player_view(p).is_some());

    // O toggle Physics desmarcado.
    bridge.hold(&mut sim, 30);
    assert!(
        bridge.player_view(p).is_none(),
        "com a física desarmada não há leitura a publicar"
    );
}

// ── OS EVENTOS ───────────────────────────────────────────────────────────────

/// **Uma aterragem produz EXACTAMENTE um `Landed`**, e a `speed` dele é a de
/// APROXIMAÇÃO — não a de depois do tique, que a mola já apagou.
#[test]
fn a_landing_produces_exactly_one_landed_at_the_approach_speed() {
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    let mut tick = 0u64;
    let _ = coast(&mut sim, &mut bridge, p, &mut tick, 30);

    tap(&mut sim, &mut bridge, p, &mut tick);
    let seen = coast(&mut sim, &mut bridge, p, &mut tick, 120);

    let landings: Vec<f32> = seen
        .iter()
        .filter_map(|e| match e {
            PlayerEvent::Landed { speed } => Some(*speed),
            _ => None,
        })
        .collect();
    assert_eq!(
        landings.len(),
        1,
        "um salto, uma aterragem — não uma por tique apoiado: {seen:?}"
    );
    assert!(
        landings[0] > 0.5,
        "a velocidade publicada é a de APROXIMAÇÃO; depois do tique a mola já a \
         teria apagado: {}",
        landings[0]
    );
}

/// ⚠️ **O GATE QUE CARREGA A WAVE.**
///
/// Um dispatch que deve **três** tiques, com um pulo a sair num deles, entrega
/// o evento do MEIO. A mutação é derivar por diff de QUADRO — ela vê só os dois
/// extremos, e o pulo que sai e é consumido dentro do mesmo dispatch some. É o
/// defeito exacto que o `W-TickContacts` mediu no canal de contatos.
#[test]
fn a_dispatch_that_owes_three_ticks_delivers_the_events_of_the_middle_ones() {
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    let mut tick = 0u64;
    let _ = coast(&mut sim, &mut bridge, p, &mut tick, 30);

    // O botão fica PRESO durante os três tiques — a borda cai no primeiro deles,
    // que é um tique do MEIO do dispatch e não o endpoint.
    bridge.set_player_input(
        p,
        PlayerInput {
            jump: true,
            ..PlayerInput::default()
        },
    );
    tick += 3;
    bridge.dispatch(&mut sim, true, tick);

    let seen: Vec<PlayerEvent> = bridge.player_events().iter().map(|(_, e)| *e).collect();
    assert!(
        seen.contains(&PlayerEvent::Jumped {
            kind: JumpKind::Ground
        }),
        "o pulo saiu num tique do meio e o canal tem de o entregar: {seen:?}"
    );
}

/// **O `Jumped.kind` distingue os três** — e a fixture inclui o de PAREDE, que é
/// onde um palpite de fora erra.
///
/// ⚠️ Um adivinho perguntaria *"ele estava no chão no tique anterior?"*, que
/// responde **não** para o pulo do ar E para o de parede: ele não os distingue,
/// e chama os dois de a mesma coisa.
#[test]
fn the_jump_kind_distinguishes_the_three() {
    // ── CHÃO ──
    let (mut sim, mut bridge, p) = scene(0.0, 0.0);
    let mut tick = 0u64;
    let _ = coast(&mut sim, &mut bridge, p, &mut tick, 30);
    // ⚠️ O toque gasta DOIS dispatches e a lista é por dispatch, então o gate
    // acumula em vez de olhar só o último — olhar o último é como um gate destes
    // fica verde por não ver nada.
    let mut ground = Vec::new();
    for held in [true, false] {
        bridge.set_player_input(
            p,
            PlayerInput {
                jump: held,
                ..PlayerInput::default()
            },
        );
        tick += 1;
        bridge.dispatch(&mut sim, true, tick);
        ground.extend(bridge.player_events().iter().map(|(_, e)| *e));
    }
    assert!(
        ground.contains(&PlayerEvent::Jumped {
            kind: JumpKind::Ground
        }),
        "o pulo do chão tem de sair nomeado: {ground:?}"
    );

    // ── PAREDE ──
    let (mut sim, mut bridge, p) = wall_rig();
    // Cair encostado na parede até a lei o agarrar, depois pular.
    let mut saw_wall = false;
    for t in 1..=180u64 {
        bridge.set_player_input(
            p,
            PlayerInput {
                drive: 1.0,
                jump: t % 2 == 0 && t > 40,
                ..PlayerInput::default()
            },
        );
        bridge.dispatch(&mut sim, true, t);
        if bridge.player_events().iter().any(|(_, e)| {
            matches!(
                e,
                PlayerEvent::Jumped {
                    kind: JumpKind::Wall
                }
            )
        }) {
            saw_wall = true;
            break;
        }
    }
    assert!(
        saw_wall,
        "um pulo de PAREDE tem de sair nomeado como tal — é o caso que um \
         palpite pelo apoio anterior confunde com o pulo do ar"
    );
}

/// Uma parede alta com um player a cair encostado nela.
fn wall_rig() -> (SimWorld, PhysicsBridge, Entity) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 6.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(1.0, 0.0)),
    ));
    let player = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: BodyKind::Dynamic,
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: 0.3,
                radius: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
        PlatformPlayer {
            float_height: FLOAT_HEIGHT,
            // A parede só existe para a lei quando ela está ARMADA.
            wall_slide_speed: 2.0,
            wall_jump_height: 1.2,
            wall_jump_push: 4.0,
            ..PlatformPlayer::default()
        },
        Transform::from_translation(Vec2::new(0.3, 4.0)),
    )).id();
    (sim, PhysicsBridge::new(), player)
}
