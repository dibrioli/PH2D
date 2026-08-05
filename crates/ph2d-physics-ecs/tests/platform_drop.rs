//! **DESCER DA PLATAFORMA** (W12) — os gates de comportamento, com o rapier de
//! verdade.
//!
//! O mecanismo de plataforma jump-through existe desde a W-OneWay: ela é sólida
//! por cima e atravessável por baixo. O que faltava era a outra metade do
//! idioma — **sair dela por baixo de propósito** —, e o plano 06 §4 a agendou
//! como *"o mecanismo existe; a feature é o gesto"*.
//!
//! # ⚠️ O que estes gates medem, e o que seria fácil medir errado
//!
//! O oráculo é **onde o personagem PARA**, nunca um instante do meio: durante a
//! travessia ele está dentro da plataforma, e qualquer amostra dali confunde
//! *"atravessou"* com *"foi cuspido"*. Deixar a cena assentar e perguntar em que
//! altura ele descansa responde à pergunta inteira — passou, e passou até ao
//! chão de baixo.
//!
//! ⚠️ **E cada gate tem o seu CONTROLE**, porque *"o personagem desceu"* é
//! satisfeito por um bug que simplesmente o deixa cair: o controle é a mesma
//! cena, o mesmo número de tiques, e o gesto que NÃO deve derrubá-lo.

#[path = "platform_scene.rs"]
mod scene_fixture;

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, OneWayPlatform, PhysicsBridge, PlatformPlayer,
    PlayerInput, RigidBody,
};
use scene_fixture::{FLOAT_HEIGHT, pose};

/// Meia-altura da plataforma fina.
const PLANK_HALF_Y: f32 = 0.1;
/// O topo da plataforma jump-through.
const PLANK_TOP: f32 = PLANK_HALF_Y;
/// O topo do chão sólido lá em baixo.
const FLOOR_TOP: f32 = -2.0;

/// A altura em que o personagem descansa sobre uma superfície de topo `top`.
fn rest_over(top: f32) -> f32 {
    top + FLOAT_HEIGHT
}

struct Rig {
    sim: SimWorld,
    bridge: PhysicsBridge,
    player: ph2d_ecs::Entity,
}

/// **Uma prancha jump-through a 2,5 m do chão, e um personagem em pé nela.**
///
/// ⚠️ **A separação vertical é ESCOLHIDA pela lei do fim da descida** (ver
/// `bridge::player::retire_drops`): ela acaba quando a caixa do personagem está
/// inteiramente abaixo da caixa da prancha, então a fixture tem de dar espaço
/// para isso ACONTECER. Com o chão em `FLOOR_TOP` o personagem descansa com o
/// topo meio metro abaixo da prancha — margem suficiente para o gate medir a
/// travessia, e não a aritmética de encaixe.
///
/// `solid = true` dá a MESMA cena com a prancha SÓLIDA: é o controle que separa
/// *"o gesto atravessa uma plataforma jump-through"* de *"o gesto atravessa
/// qualquer coisa"*.
fn rig(solid: bool) -> Rig {
    let mut sim = SimWorld::new();
    // O chão de baixo, sólido e comum: é ele que prova que o personagem PAROU
    // em vez de simplesmente cair para fora do mundo.
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
        Transform::from_translation(Vec2::new(0.0, FLOOR_TOP - 0.5)),
    ));

    let plank = (
        Name::new("Plank"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 40.0,
                half_y: PLANK_HALF_Y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    );
    if solid {
        sim.world_mut().spawn(plank);
    } else {
        sim.world_mut().spawn((plank, OneWayPlatform));
    }

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
                // ⚠️ A quina e a memória do chão fora do caminho: esta cena não
                // tem teto nem plataforma móvel, e desligá-las explicitamente é
                // o que mantém o gate a medir UMA wave.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.0, rest_over(PLANK_TOP))),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}

/// Assenta a cena e devolve o tique em que ela parou.
fn settle(r: &mut Rig, ticks: u64, from: u64) -> u64 {
    let mut t = from;
    for _ in 0..ticks {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
    }
    t
}

/// Segura um gesto por alguns tiques e depois solta — a forma de um aperto de
/// verdade, e o que a lei do buffer/borda espera ver.
fn press(r: &mut Rig, input: PlayerInput, hold: u64, then: u64, from: u64) -> u64 {
    r.bridge.set_player_input(r.player, input);
    let t = settle(r, hold, from);
    r.bridge.set_player_input(r.player, PlayerInput::default());
    settle(r, then, t)
}

/// **O GATE DA WAVE.** Baixo + pulo sobre uma plataforma jump-through faz o
/// personagem DESCER através dela e pousar no chão de baixo.
///
/// ⚠️ **O controle é o pulo SOZINHO**, na mesma cena e no mesmo número de
/// tiques: sem ele, *"o personagem acabou lá em baixo"* seria satisfeito por
/// qualquer defeito que o deixasse cair — inclusive por uma plataforma que
/// nunca foi sólida.
#[test]
fn down_and_jump_drops_through_a_one_way_platform() {
    // O gesto: baixo + pulo.
    let mut dropping = rig(false);
    let t = settle(&mut dropping, 30, 0);
    press(
        &mut dropping,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: true,
        },
        4,
        90,
        t,
    );
    let (_, dropped_y) = pose(&dropping.sim);

    // O CONTROLE: só o pulo, e ele volta para a prancha.
    let mut jumping = rig(false);
    let t = settle(&mut jumping, 30, 0);
    press(
        &mut jumping,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
        },
        4,
        90,
        t,
    );
    let (_, jumped_y) = pose(&jumping.sim);

    assert!(
        (dropped_y - rest_over(FLOOR_TOP)).abs() < 0.15,
        "com baixo+pulo ele tem de pousar no CHAO ({:.3}), e parou em {dropped_y:.3}",
        rest_over(FLOOR_TOP)
    );
    assert!(
        (jumped_y - rest_over(PLANK_TOP)).abs() < 0.15,
        "so com o pulo ele volta para a PRANCHA ({:.3}), e parou em {jumped_y:.3}",
        rest_over(PLANK_TOP)
    );
}

/// **O baixo SOZINHO não derruba ninguém**, e é a metade do gesto que protege o
/// dia em que existir um agachar.
///
/// ⚠️ Segurar baixo enquanto se anda é uma coisa que o jogador faz o tempo todo;
/// se ela bastasse para atravessar o chão, a plataforma jump-through deixaria de
/// ser chão.
#[test]
fn holding_down_alone_does_not_drop() {
    let mut r = rig(false);
    let t = settle(&mut r, 30, 0);
    press(
        &mut r,
        PlayerInput {
            drive: 0.0,
            jump: false,
            down: true,
        },
        60,
        30,
        t,
    );
    let (_, y) = pose(&r.sim);
    assert!(
        (y - rest_over(PLANK_TOP)).abs() < 0.15,
        "so o baixo nao atravessa nada: ele tem de continuar em {:.3}, e esta em {y:.3}",
        rest_over(PLANK_TOP)
    );
}

/// **Sobre chão SÓLIDO o mesmo gesto PULA**, e não faz nada de estranho.
///
/// ⚠️ É o gate que impede a descida de roubar o pulo: o botão só muda de
/// significado onde a descida é possível, e em toda outra superfície ele
/// continua a ser o botão de pulo. Sem ele, um personagem agachado num chão
/// comum deixaria de conseguir pular — meio controle morto, em silêncio.
#[test]
fn the_same_gesture_on_solid_ground_is_still_a_jump() {
    let mut r = rig(true);
    let t = settle(&mut r, 30, 0);
    let (_, before) = pose(&r.sim);

    r.bridge.set_player_input(
        r.player,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: true,
        },
    );
    // O ÁPICE, não o repouso: um pulo acaba onde começou, então medir depois de
    // assentar não distinguiria pular de não pular.
    let mut peak = before;
    let mut t = t;
    for _ in 0..40 {
        t += 1;
        r.bridge.dispatch(&mut r.sim, true, t);
        let (_, y) = pose(&r.sim);
        peak = peak.max(y);
    }
    assert!(
        peak > before + 1.0,
        "sobre chao solido o gesto continua a ser um PULO: pico {peak:.3} contra {before:.3}"
    );
}

/// **A plataforma volta a ser sólida depois da travessia** — o personagem pula
/// de baixo e POUSA nela.
///
/// ⚠️ **Este é o gate do FIM da descida**, e ele é o que separa a wave de um
/// interruptor que nunca desliga: sem a retirada, a prancha ficaria
/// permanentemente atravessável para este personagem e ele nunca mais
/// conseguiria ficar em cima dela — um defeito que o gate da descida sozinho
/// julgaria um sucesso.
#[test]
fn the_platform_is_solid_again_once_he_is_through() {
    let mut r = rig(false);
    let t = settle(&mut r, 30, 0);
    // Desce.
    let t = press(
        &mut r,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: true,
        },
        4,
        90,
        t,
    );
    let (_, down_y) = pose(&r.sim);
    assert!(
        (down_y - rest_over(FLOOR_TOP)).abs() < 0.15,
        "premissa deste gate: ele tem de ter chegado ao chao, e esta em {down_y:.3}"
    );

    // E volta, sem o baixo.
    press(
        &mut r,
        PlayerInput {
            drive: 0.0,
            jump: true,
            down: false,
        },
        6,
        120,
        t,
    );
    let (_, up_y) = pose(&r.sim);
    assert!(
        (up_y - rest_over(PLANK_TOP)).abs() < 0.15,
        "a prancha tem de estar SOLIDA outra vez: esperado {:.3}, e ele parou em {up_y:.3}",
        rest_over(PLANK_TOP)
    );
}
