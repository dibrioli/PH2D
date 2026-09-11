//! **O TERCEIRO MODO — o mundo físico é CENÁRIO** (W-KinPure).
//!
//! O plano define o *puro sangue* numa frase: *nada de reação, nada de
//! empurrão*. Estes gates afirmam as duas metades daquilo que ele **não** faz e
//! — o que importa mais — as quatro coisas que ele **continua** a fazer, porque
//! é o mesmo controlador com dois canais calados.
//!
//! ⚠️ **Todo gate daqui tem CONTROLE**, e não é cerimônia: um zero medido num
//! personagem que não empurra é indistinguível de um zero medido numa cena que
//! não tem o que empurrar. A coluna `Kinematic` é a que dá sentido à coluna
//! `Pure`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, LockRotation, PhysicsBridge,
    PlatformPlayer, PlayerMode, RigidBody,
};
use ph2d_platformer::PlayerInput;

const FLOAT_HEIGHT: f32 = 0.9;

fn spawn_player(sim: &mut SimWorld, mode: PlayerMode, at: Vec2) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Player".to_string()),
            RigidBody {
                kind: BodyKind::Kinematic,
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
                ..PlatformPlayer::default()
            },
            mode,
            Transform::from_translation(at),
        ))
        .id()
}

fn floor(sim: &mut SimWorld) {
    sim.world_mut().spawn((
        Name::new("Floor".to_string()),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 60.0,
                half_y: 0.5,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, -0.5)),
    ));
}

fn xy(sim: &SimWorld, e: Entity) -> (f32, f32) {
    let t = sim.world().get::<Transform>(e).expect("pose").translation;
    (t.x, t.y)
}

fn drive(bridge: &mut PhysicsBridge, e: Entity, d: f32, jump: bool) {
    bridge.set_player_input(
        e,
        PlayerInput {
            drive: d,
            jump,
            ..PlayerInput::default()
        },
    );
}

/// Uma jangada sem peso próprio: todo milímetro que ela desce é do personagem.
fn raft_sink(mode: PlayerMode) -> f32 {
    let mut sim = SimWorld::new();
    let raft = sim
        .world_mut()
        .spawn((
            Name::new("Raft".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 30.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            LockRotation,
            Transform::from_translation(Vec2::new(0.0, -0.25)),
        ))
        .id();
    spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    for t in 1..=60u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let before = xy(&sim, raft).1;
    for t in 61..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    xy(&sim, raft).1 - before
}

/// **O PESO não chega ao chão** — e o Snap prova que a jangada afunda mesmo.
#[test]
fn the_pure_player_does_not_weigh_on_what_holds_him() {
    let snap = raft_sink(PlayerMode::Kinematic);
    let pure = raft_sink(PlayerMode::Pure);
    assert!(
        snap < -0.05,
        "o CONTROLE tem de afundar a jangada: {snap:.4} m"
    );
    assert!(
        pure.abs() < 1.0e-4,
        "e o puro sangue nao a toca: {pure:.4} m (contra {snap:.4})"
    );
}

/// **O EMPURRÃO lateral não existe** — e o Snap prova que o caixote anda.
#[test]
fn the_pure_player_does_not_shove_what_he_walks_into() {
    fn travel(mode: PlayerMode) -> f32 {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        let krate = sim
            .world_mut()
            .spawn((
                Name::new("Crate".to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
                LockRotation,
                Transform::from_translation(Vec2::new(1.5, 0.3)),
            ))
            .id();
        let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
        let mut bridge = PhysicsBridge::new();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let before = xy(&sim, krate).0;
        drive(&mut bridge, p, 1.0, false);
        for t in 61..=240u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        xy(&sim, krate).0 - before
    }
    let snap = travel(PlayerMode::Kinematic);
    let pure = travel(PlayerMode::Pure);
    assert!(snap > 5.0, "o CONTROLE tem de empurrar: {snap:.4} m");
    assert!(
        pure.abs() < 1.0e-3,
        "e o puro sangue passa sem mover nada: {pure:.4} m (contra {snap:.4})"
    );
}

/// **A LEI DE INTENÇÃO é BIT-IDÊNTICA** — andar e pular são o mesmo caminho.
///
/// ⚠️ É este gate que torna o `Pure` *o mesmo personagem com dois canais
/// calados* em vez de um segundo personagem. Sobre chão ESTÁTICO nada volta ao
/// player por nenhuma das duas metades, então a igualdade é EXATA — e uma
/// tolerância aqui esconderia precisamente a divergência que ela existe para
/// proibir.
#[test]
fn the_pure_player_walks_and_jumps_exactly_like_snap() {
    fn run(mode: PlayerMode) -> (f32, f32) {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
        let mut bridge = PhysicsBridge::new();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        drive(&mut bridge, p, 1.0, false);
        for t in 61..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let walked = xy(&sim, p).0;
        drive(&mut bridge, p, 1.0, true);
        let mut apex = f32::MIN;
        for t in 181..=300u64 {
            bridge.dispatch(&mut sim, true, t);
            apex = apex.max(xy(&sim, p).1);
        }
        (walked, apex)
    }
    let snap = run(PlayerMode::Kinematic);
    let pure = run(PlayerMode::Pure);
    assert_eq!(
        snap, pure,
        "o puro sangue e' o MESMO controlador: andou/apice {snap:?} contra {pure:?}"
    );
    assert!(snap.0 > 5.0, "e a fixture tem de conter o fenomeno");
}

/// **SER LEVADO não é influenciar** (K7) — a plataforma continua a carregá-lo.
///
/// ⚠️ O canal que a wave cala é o que sai DELE; o que entra fica. Se o `Pure`
/// perdesse isto, a wave teria calado o canal errado — e um platformer clássico
/// é exactamente o género que anda em cima de plataformas.
#[test]
fn the_pure_player_is_still_carried_by_a_moving_platform() {
    fn carried(mode: PlayerMode) -> f32 {
        let mut sim = SimWorld::new();
        let plat = sim
            .world_mut()
            .spawn((
                Name::new("Platform".to_string()),
                RigidBody {
                    kind: BodyKind::Kinematic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 8.0,
                        half_y: 0.25,
                    },
                    ..Collider::default()
                },
                Transform::from_translation(Vec2::new(0.0, -0.25)),
            ))
            .id();
        let p = spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
        let mut bridge = PhysicsBridge::new();
        for t in 1..=60u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let before = xy(&sim, p).0;
        for t in 61..=180u64 {
            let x = xy(&sim, plat).0;
            sim.world_mut()
                .get_mut::<Transform>(plat)
                .expect("pose")
                .translation
                .x = x + 2.0 / 60.0;
            bridge.dispatch(&mut sim, true, t);
        }
        xy(&sim, p).0 - before
    }
    let snap = carried(PlayerMode::Kinematic);
    let pure = carried(PlayerMode::Pure);
    assert!(snap > 1.0, "o CONTROLE e' levado: {snap:.4} m");
    assert_eq!(
        snap, pure,
        "e o puro sangue e' levado igual: {snap:.4} contra {pure:.4}"
    );
}

/// **CENÁRIO não quer dizer FANTASMA** — um caixote atirado contra ele PARA.
///
/// ⚠️ Este gate existe para impedir uma leitura tentadora de *"o mundo físico é
/// cenário"*: a de que o mundo deixaria de o ver. Ele é sólido nos dois modos,
/// e em nenhum platformer clássico se atravessa uma caixa.
#[test]
fn the_pure_player_is_still_solid() {
    fn stops_at(mode: PlayerMode) -> f32 {
        let mut sim = SimWorld::new();
        floor(&mut sim);
        spawn_player(&mut sim, mode, Vec2::new(0.0, FLOAT_HEIGHT));
        let krate = sim
            .world_mut()
            .spawn((
                Name::new("Crate".to_string()),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                Collider {
                    shape: ColliderShape::Cuboid {
                        half_x: 0.3,
                        half_y: 0.3,
                    },
                    ..Collider::default()
                },
                LockRotation,
                InitialVelocity {
                    linvel: [-6.0, 0.0],
                    angvel: 0.0,
                },
                Transform::from_translation(Vec2::new(3.0, 0.3)),
            ))
            .id();
        let mut bridge = PhysicsBridge::new();
        for t in 1..=180u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        xy(&sim, krate).0
    }
    for mode in [PlayerMode::Kinematic, PlayerMode::Pure] {
        let x = stops_at(mode);
        assert!(
            x > 0.2,
            "{mode:?}: o caixote tem de ser BARRADO pelo personagem em x=0, e parou em {x:.4}"
        );
    }
}

/// **Os knobs AUTORADOS sobrevivem ao modo** — o `Pure` cala, não apaga.
///
/// ⚠️ A metade que importa é a de VOLTA: se o chip zerasse os escalares, o
/// artista perderia o que escreveu ao experimentar o modo, e a perda seria
/// silenciosa. Aqui a MESMA entidade, com os MESMOS números, afunda a jangada
/// depois de voltar.
#[test]
fn the_authored_reaction_is_silenced_by_the_mode_never_erased() {
    let mut sim = SimWorld::new();
    let raft = sim
        .world_mut()
        .spawn((
            Name::new("Raft".to_string()),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 30.0,
                    half_y: 0.25,
                },
                ..Collider::default()
            },
            GravityScale(0.0),
            LockRotation,
            Transform::from_translation(Vec2::new(0.0, -0.25)),
        ))
        .id();
    let p = spawn_player(&mut sim, PlayerMode::Pure, Vec2::new(0.0, FLOAT_HEIGHT));
    let mut bridge = PhysicsBridge::new();
    let start = xy(&sim, raft).1;
    for t in 1..=180u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let after_pure = xy(&sim, raft).1;

    // O artista volta ao Snap. Nada mais é tocado.
    *sim.world_mut().get_mut::<PlayerMode>(p).expect("modo") = PlayerMode::Kinematic;
    for t in 181..=360u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let after_snap = xy(&sim, raft).1;

    // ⚠️ **As DUAS metades, e a primeira é a que o gate quase não teve:** sem
    // ela, *"depois de voltar afundou mais"* é verdade num mundo em que o modo
    // nunca calou nada — a jangada sem peso próprio afunda para sempre, então a
    // segunda leitura é sempre menor que a primeira.
    assert!(
        (after_pure - start).abs() < 1.0e-4,
        "sob o puro sangue a jangada nao se move: {start:.4} -> {after_pure:.4}"
    );
    assert!(
        after_snap < after_pure - 0.05,
        "voltar ao Snap tem de devolver o peso autorado: {after_pure:.4} -> {after_snap:.4}"
    );
    let cfg = sim.world().get::<PlatformPlayer>(p).expect("player");
    assert!(
        (cfg.reaction_support - 1.0).abs() < 1.0e-6 && (cfg.reaction_push - 1.0).abs() < 1.0e-6,
        "e os escalares nunca foram escritos: {} / {}",
        cfg.reaction_support,
        cfg.reaction_push
    );
}
