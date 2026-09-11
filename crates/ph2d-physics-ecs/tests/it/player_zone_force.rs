//! **A CORRENTEZA LEVA UM PERSONAGEM CINEMÁTICO** (W-ZoneForce) — a metade ECS.
//!
//! Antes desta wave o `effector::apply` recusava corpo não-dinâmico antes de o tocar
//! (e a recusa está CERTA: um corpo cinemático tem massa infinita e o solver ignoraria
//! o impulso) e a lei cinemática integrava um `Fluid` **sem força nenhuma**. As duas
//! metades corretas somavam um personagem que a água empurra num modo e não empurra nos
//! outros dois — medido, `0,0000 m` em qualquer força, contra os 21,83 m de um caixote.
//!
//! O oráculo destes gates é o **modo DINÂMICO**: seja o que for que o freio da caminhada
//! deixe passar, os três modos têm de deixar passar o mesmo. Um literal aqui seria uma
//! segunda opinião sobre uma lei que o outro modo já responde.
//!
//! ⚠️ **Sem gravidade de propósito:** a zona passa a ser a única coisa a agir, então
//! *onde ele parou* É *o que ela fez*. E isso só é perguntável porque a consulta deixou
//! de sair por um early-out de gravidade — ver `fluid_at`.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    AreaEffector, AreaFalloff, AreaForceWorldAxes, BodyKind, Collider, ColliderShape, LockRotation,
    PhysicsBridge, PhysicsSettings, PlatformPlayer, PlayerInput, PlayerMode, RigidBody,
};

const HALF_H: f32 = 0.3;
const RADIUS: f32 = 0.2;

/// Sem gravidade: a zona é a única coisa a agir.
fn zero_gravity() -> PhysicsSettings {
    PhysicsSettings {
        gravity_y: 0.0,
        ..Default::default()
    }
}

/// Uma correnteza larga que empurra em `+X` do próprio frame, girada por `rotation`.
fn current(sim: &mut SimWorld, force: f32, rotation: f32, world_axes: bool, falloff: f32) {
    let mut e = sim.world_mut().spawn((
        Name::new("Current"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            is_sensor: true,
            shape: ColliderShape::Cuboid {
                half_x: 30.0,
                half_y: 30.0,
            },
            ..Collider::default()
        },
        AreaEffector {
            force: [force, 0.0],
        },
        Transform {
            rotation,
            ..Transform::from_translation(Vec2::new(0.0, 0.0))
        },
    ));
    if world_axes {
        e.insert(AreaForceWorldAxes);
    }
    if falloff > 0.0 {
        e.insert(AreaFalloff(falloff));
    }
}

/// Um personagem no modo pedido, ou um caixote nu quando `mode` é `None`.
///
/// `brake` é o knob do artista que decide quanto a caminhada resiste à correnteza — a
/// ablação que separa *"a zona não chega"* de *"a zona chega e ele anda contra"*.
fn subject(sim: &mut SimWorld, mode: Option<PlayerMode>, brake: f32, at: Vec2) -> Entity {
    let mut e = sim.world_mut().spawn((
        Name::new("Subject"),
        RigidBody {
            kind: if mode.is_some_and(PlayerMode::drives_itself) {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            },
        },
        Collider {
            shape: ColliderShape::Capsule {
                half_height: HALF_H,
                radius: RADIUS,
            },
            density: 1.0,
            ..Collider::default()
        },
        LockRotation,
        Transform::from_translation(at),
    ));
    if let Some(m) = mode {
        e.insert(PlatformPlayer {
            acceleration: brake,
            air_acceleration: brake,
            ..PlatformPlayer::default()
        });
        e.insert(m);
    }
    e.id()
}

fn pose_of(sim: &SimWorld) -> Vec2 {
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Subject" {
            return t.translation;
        }
    }
    panic!("o sujeito tem de existir");
}

/// Dois segundos de correnteza. Devolve para onde o sujeito foi levado.
fn carried(mode: Option<PlayerMode>, force: f32, brake: f32) -> Vec2 {
    carried_zone(mode, force, brake, 0.0, false, 0.0, Vec2::new(0.0, 0.0))
}

#[allow(clippy::too_many_arguments)]
fn carried_zone(
    mode: Option<PlayerMode>,
    force: f32,
    brake: f32,
    rotation: f32,
    world_axes: bool,
    falloff: f32,
    at: Vec2,
) -> Vec2 {
    let mut sim = SimWorld::new();
    current(&mut sim, force, rotation, world_axes, falloff);
    let who = subject(&mut sim, mode, brake, at);
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    pose_of(&sim)
}

/// O freio de fábrica satura contra uma correnteza fraca — os dois modos mal andam, e
/// é o mesmo mal-andar. Este é o valor onde a correnteza de facto VENCE o freio, que é
/// o regime em que a pergunta da wave tem resposta observável.
const LOOSE_BRAKE: f32 = 1.0;

#[test]
fn a_current_carries_a_kinematic_player() {
    for mode in [PlayerMode::Kinematic, PlayerMode::Pure] {
        let x = carried(Some(mode), 4.0, LOOSE_BRAKE).x;
        assert!(
            x > 5.0,
            "{mode:?} tem de ser levado pela correnteza; andou {x:.4} m (antes da wave: 0,0000 em qualquer forca)"
        );
    }
}

#[test]
fn the_three_modes_answer_the_same_current() {
    // O DINÂMICO é o oráculo: o que o freio da caminhada dele deixa passar é o que os
    // outros dois têm de deixar passar. A banda é a mesma classe de aproximação que o
    // arrasto já carrega (a lei cinemática integra por TIQUE, o solver por SUB-PASSO).
    for force in [4.0f32, 16.0, 64.0] {
        let dyn_x = carried(Some(PlayerMode::Dynamic), force, LOOSE_BRAKE).x;
        for mode in [PlayerMode::Kinematic, PlayerMode::Pure] {
            let kin_x = carried(Some(mode), force, LOOSE_BRAKE).x;
            let ratio = kin_x / dyn_x;
            assert!(
                (0.94..=1.06).contains(&ratio),
                "forca {force}: {mode:?} andou {kin_x:.4} m contra {dyn_x:.4} do Dynamic (razao {ratio:.4})"
            );
        }
    }
}

#[test]
fn a_dry_world_leaves_the_kinematic_player_where_it_was() {
    // O CONTROLE: sem zona nenhuma o personagem não anda de lado. Sem ele os gates
    // acima seriam satisfeitos por qualquer deriva.
    let mut sim = SimWorld::new();
    let who = subject(
        &mut sim,
        Some(PlayerMode::Kinematic),
        LOOSE_BRAKE,
        Vec2::new(0.0, 0.0),
    );
    let mut bridge = PhysicsBridge::new();
    bridge.set_settings(zero_gravity());
    bridge.set_player_input(who, PlayerInput::default());
    for t in 1..=120u64 {
        bridge.dispatch(&mut sim, true, t);
    }
    let x = pose_of(&sim).x;
    assert!(x.abs() < 1e-3, "sem zona ele nao anda de lado; x = {x:.6}");
}

#[test]
fn the_kinematic_player_is_pushed_in_the_zones_own_frame() {
    // ⚠️ Este é o gate que prova que a consulta PERGUNTA à porta do solver em vez de
    // re-derivar: o frame (W-AreaFrame) chega ao personagem cinemático sem uma linha
    // sobre rotação existir do lado da lei.
    let turned = carried_zone(
        Some(PlayerMode::Kinematic),
        16.0,
        LOOSE_BRAKE,
        std::f32::consts::FRAC_PI_2,
        false,
        0.0,
        Vec2::new(0.0, 0.0),
    );
    assert!(
        turned.y > 3.0 && turned.x.abs() < turned.y * 0.25,
        "uma zona girada 90 graus tem de empurrar para +Y; foi para ({:.3}, {:.3})",
        turned.x,
        turned.y
    );
    // E a escapatória do W-AreaFrame continua a valer do outro lado da fronteira.
    let pinned = carried_zone(
        Some(PlayerMode::Kinematic),
        16.0,
        LOOSE_BRAKE,
        std::f32::consts::FRAC_PI_2,
        true,
        0.0,
        Vec2::new(0.0, 0.0),
    );
    assert!(
        pinned.x > 3.0 && pinned.y.abs() < pinned.x * 0.25,
        "marcada 'world axes', a MESMA zona girada tem de empurrar para +X; foi para ({:.3}, {:.3})",
        pinned.x,
        pinned.y
    );
}

#[test]
fn the_falloff_reaches_the_kinematic_player() {
    // A terceira porta que a consulta herda de graça (W-AreaFalloff): quem está longe
    // do olho da rajada é levado menos. O corpo da margem nasce a 24 dos 30 de
    // meia-extensão, então continua DENTRO durante a corrida — "andou menos" não pode
    // ser "saiu".
    let eye = carried_zone(
        Some(PlayerMode::Kinematic),
        16.0,
        LOOSE_BRAKE,
        0.0,
        false,
        1.0,
        Vec2::new(0.0, 0.0),
    )
    .x;
    let edge = carried_zone(
        Some(PlayerMode::Kinematic),
        16.0,
        LOOSE_BRAKE,
        0.0,
        false,
        1.0,
        Vec2::new(0.0, 24.0),
    )
    .x;
    assert!(
        edge < eye * 0.6,
        "na margem o empurrao tem de ser bem menor: olho {eye:.4} m, margem {edge:.4} m"
    );
    assert!(
        edge > 0.0,
        "a margem ainda recebe alguma coisa: {edge:.4} m"
    );
}
