//! **A BEIRADA, pela porta do produto** (`W-Ledge`) — o sensor, a lei e o solver
//! juntos.
//!
//! ⚠️ **O oráculo é a POSE, e não um campo de estado:** o que o jogador vê é o
//! personagem parar com as mãos no lábio e depois ficar de pé em cima dele. Um
//! gate sobre `hanging == true` ficaria verde com o corpo a atravessar o
//! patamar.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A face vertical do bloco.
const WALL_FACE: f32 = 0.5;
/// O TOPO do bloco — o lábio.
const LIP_Y: f32 = 3.5;
/// A meia-altura da cápsula (`half_height + radius`).
const HALF_H: f32 = 0.5;
/// A meia-largura dela.
const HALF_W: f32 = 0.2;
/// A altura de flutuação — a mesma do `platform_scene`, e é ela que decide onde
/// um personagem DE PÉ assenta.
const FLOAT_HEIGHT: f32 = 0.9;

/// Chão, um bloco alto com um topo, e o personagem ao lado dele.
///
/// ⚠️ **`gap` é a distância da BORDA do corpo à face**, e ela é parte da
/// fixture: a beirada é apanhada até `grab` de distância, então uma fixture
/// colada à parede não distinguiria *o alcance funciona* de *ele estava a
/// tocar*.
fn scene(centre_y: f32, gap: f32, grab: f32) -> (SimWorld, PhysicsBridge, Entity) {
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
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let half_y = (LIP_Y - 0.5) * 0.5;
    sim.world_mut().spawn((
        Name::new("Block"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                half_y,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(WALL_FACE + 1.0, 0.5 + half_y)),
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
                ledge_grab: grab,
                ledge_speed: 3.0,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(WALL_FACE - HALF_W - gap, centre_y)),
        ))
        .id();
    (sim, PhysicsBridge::new(), player)
}

fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

/// Corre `n` tiques com uma entrada fixa.
fn run(
    sim: &mut SimWorld,
    bridge: &mut PhysicsBridge,
    player: Entity,
    tick: &mut u64,
    n: u64,
    input: PlayerInput,
) {
    for _ in 0..n {
        bridge.set_player_input(player, input);
        *tick += 1;
        bridge.dispatch(sim, true, *tick);
    }
}

/// **Agarrar-se PARA a queda, e a pose é as mãos no lábio.**
///
/// ⚠️ **O CONTROLO é a mesma cena com a capacidade desligada**, e é ele que
/// impede este gate de ficar verde por o personagem calhar de parar ali por
/// outro motivo.
///
/// ⚠️ **E o controlo NÃO é *"ele cai"***, que foi como este gate nasceu e
/// falhou: quem empurra contra uma parede sem `wall_slide_speed` **fica preso
/// pelo atrito** — medido nesta mesma fixture, ele desce **0,14 m em 1,5 s** e
/// pára 29 cm abaixo do lábio. O que a capacidade muda não é *cair ou não*, é
/// **onde ele pára**.
#[test]
fn catching_a_ledge_leaves_the_hands_on_the_lip() {
    for (grab, hangs) in [(0.0_f32, false), (0.4, true)] {
        let (mut sim, mut bridge, player) = scene(LIP_Y - HALF_H - 0.15, 0.30, grab);
        let mut tick = 0;
        run(
            &mut sim,
            &mut bridge,
            player,
            &mut tick,
            90,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        let top = pose(&sim).1 + HALF_H;
        if hangs {
            assert!(
                (top - LIP_Y).abs() < 0.02,
                "o topo do corpo tem de assentar no labio: {top:.4} contra {LIP_Y:.4}"
            );
        } else {
            assert!(
                top < LIP_Y - 0.1,
                "desligada, ele NAO chega ao labio — se chegasse, o gate acima \
                 nao provaria nada: {top:.4}"
            );
        }
    }
}

/// **E ele fica lá** — o pendurar é um regime, não um tique.
#[test]
fn the_hang_holds_for_as_long_as_the_finger_does() {
    let (mut sim, mut bridge, player) = scene(LIP_Y - HALF_H - 0.15, 0.30, 0.4);
    let mut tick = 0;
    let push = PlayerInput {
        drive: 1.0,
        ..PlayerInput::default()
    };
    run(&mut sim, &mut bridge, player, &mut tick, 60, push);
    let settled = pose(&sim).1;
    run(&mut sim, &mut bridge, player, &mut tick, 300, push);
    let later = pose(&sim).1;
    assert!(
        (later - settled).abs() < 0.01,
        "cinco segundos depois ele tem de estar onde estava: {settled:.4} -> {later:.4}"
    );
}

/// **Soltar a direção é soltar-se** — e o que acontece a seguir é cair.
#[test]
fn releasing_the_direction_lets_go() {
    let (mut sim, mut bridge, player) = scene(LIP_Y - HALF_H - 0.15, 0.30, 0.4);
    let mut tick = 0;
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        60,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let hung = pose(&sim).1;
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        60,
        PlayerInput::default(),
    );
    assert!(
        pose(&sim).1 < hung - 0.5,
        "sem o dedo ele nao esta' agarrado a nada"
    );
}

/// **O MANTLE põe-no DE PÉ em cima do patamar.**
///
/// ⚠️ **A altura de repouso é `lábio + float_height`, e não *"acima do
/// lábio"***: a perna é uma mola e o personagem PAIRA, então um gate que
/// pedisse só *"passou por cima"* ficaria verde com ele a subir para sempre.
#[test]
fn the_mantle_ends_standing_on_the_ledge() {
    let (mut sim, mut bridge, player) = scene(LIP_Y - HALF_H - 0.15, 0.30, 0.4);
    let mut tick = 0;
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        90,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    // ⚠️ **Um TOQUE, e o dedo LARGA a direção a seguir** — com o pulo preso o
    // personagem salta no tique em que a subida acaba (o pulo é mascarado na
    // entrada enquanto a beirada age, então o botão lê como aperto novo), e com
    // a direção presa ele anda até cair pelo outro lado do bloco.
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        2,
        PlayerInput {
            drive: 1.0,
            jump: true,
            ..PlayerInput::default()
        },
    );
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        180,
        PlayerInput::default(),
    );
    let (x, y) = pose(&sim);
    assert!(
        (y - (LIP_Y + FLOAT_HEIGHT)).abs() < 0.1,
        "de pe' no patamar seria y = {:.4}; ele esta' em {y:.4}",
        LIP_Y + FLOAT_HEIGHT
    );
    assert!(
        x - HALF_W > WALL_FACE - 1e-3,
        "e o corpo INTEIRO tem de estar do lado de dentro: borda em {:.4} contra a face {WALL_FACE:.4}",
        x - HALF_W
    );
}

/// **O alcance é o que o `grab` diz** — e é ele que decide o que está alto
/// demais.
///
/// ⚠️ **A régua é a VERTICAL, e a primeira versão deste gate usava a
/// horizontal — e ficou verde nas duas colunas.** O motivo é que a distância
/// lateral **não é uma constante da fixture**: o dedo que empurra leva o corpo
/// até encostar na face, então qualquer `grab` acaba por alcançar. O que é de
/// facto uma propriedade do corpo é *quão alto acima da cabeça* o lábio está —
/// e é isso que o número promete.
///
/// ⚠️ **Duas colunas com a MESMA cena e `grab` diferente**, e não duas cenas com
/// o mesmo `grab`: o que está sob teste é o número, então tudo o resto tem de
/// ser igual ao bit.
#[test]
fn the_reach_is_what_the_grab_says_it_is() {
    // O lábio 0,30 m acima da cabeça: alcançável com 0,4 e alto demais com 0,2.
    for (grab, catches) in [(0.4_f32, true), (0.2, false)] {
        let (mut sim, mut bridge, player) = scene(LIP_Y - 0.30 - HALF_H, 0.30, grab);
        let mut tick = 0;
        run(
            &mut sim,
            &mut bridge,
            player,
            &mut tick,
            90,
            PlayerInput {
                drive: 1.0,
                ..PlayerInput::default()
            },
        );
        let top = pose(&sim).1 + HALF_H;
        assert_eq!(
            (top - LIP_Y).abs() < 0.02,
            catches,
            "grab {grab}: topo em {top:.4}, labio em {LIP_Y:.4}"
        );
    }
}

/// **Um degrau NÃO é uma beirada** — no chão a lei não pergunta nada.
///
/// ⚠️ Sem isto, um personagem a andar contra um degrau da altura do `grab`
/// pendurar-se-ia nele em vez de o subir a pé — e o degrau já tem dono (a perna
/// e o `cling_distance`).
#[test]
fn a_step_you_can_walk_up_is_not_a_ledge() {
    // O corpo assenta NO CHÃO, ao lado de um bloco cujo topo está ao alcance.
    let (mut sim, mut bridge, player) = scene(0.5 + FLOAT_HEIGHT, 0.30, 0.4);
    let mut tick = 0;
    run(
        &mut sim,
        &mut bridge,
        player,
        &mut tick,
        120,
        PlayerInput {
            drive: 1.0,
            ..PlayerInput::default()
        },
    );
    let (x, y) = pose(&sim);
    assert!(
        (y - (0.5 + FLOAT_HEIGHT)).abs() < 0.15,
        "ele continua no chao, a andar: y = {y:.4}"
    );
    assert!(x > WALL_FACE - HALF_W - 0.30, "e andou para a frente");
}
