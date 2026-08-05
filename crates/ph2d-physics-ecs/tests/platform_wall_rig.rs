//! **A cena da parede** (W13) — a fixture que o gate e a sonda partilham.
//!
//! ⚠️ **Incluída por `#[path]` nos dois arquivos, e não copiada**, pelo motivo
//! que o `platform_scene.rs` já escreve: cada arquivo em `tests/` é um crate
//! próprio, e uma segunda cópia da cena seria uma segunda resposta a *"como é
//! um personagem a cair ao lado de uma parede"* — as duas divergiriam no dia em
//! que a cápsula ou o alcance do sensor mudassem, com a que não foi atualizada a
//! seguir VERDE sobre outra premissa.
//!
//! ⚠️ O `allow(dead_code)` é do formato, não do descuido: cada consumidor usa o
//! subconjunto de que precisa (a sonda mexe na config, o gate não).
#![allow(dead_code)]

use ph2d_core::Vec2;
use ph2d_ecs::{Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, PlatformPlayer, PlayerInput,
    RigidBody,
};

/// A altura de flutuação desta fixture — a mesma razão geométrica do
/// `platform_scene`.
pub const FLOAT_HEIGHT: f32 = 0.9;
/// A face ESQUERDA da parede: é para ela que o personagem é empurrado.
pub const WALL_FACE: f32 = 0.75;
/// Onde o personagem começa a cair.
pub const START_Y: f32 = 5.0;

pub struct Rig {
    pub sim: SimWorld,
    pub bridge: PhysicsBridge,
    pub player: ph2d_ecs::Entity,
}

impl Rig {
    /// Afina a config autorada deste player — usado pelas sondas que varrem um
    /// knob.
    pub fn player_cfg(&mut self, f: impl FnOnce(&mut PlatformPlayer)) {
        let mut c = self
            .sim
            .world_mut()
            .get_mut::<PlatformPlayer>(self.player)
            .expect("o player tem PlatformPlayer");
        f(&mut c);
    }
}

pub fn pose(sim: &SimWorld) -> (f32, f32) {
    let mut found = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == "Player" {
            found = Some((t.translation.x, t.translation.y));
        }
    }
    found.expect("o player tem de existir")
}

/// Empurrar contra a parede.
pub fn into_wall() -> PlayerInput {
    PlayerInput {
        drive: 1.0,
        jump: false,
        down: false,
    }
}

/// **Uma parede alta à direita, e um personagem a cair ao lado dela.**
///
/// `slide` e `jump_height` são as duas metades da capacidade; passar `0.0` nas
/// duas dá o CONTROLE — a mesma cena, o mesmo número de tiques, com a parede a
/// ser só geometria.
pub fn rig(slide: f32, jump_height: f32) -> Rig {
    let mut sim = SimWorld::new();
    // Um chão bem longe: a cena é sobre a queda, e o chão existe só para o
    // personagem não cair para sempre.
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
        Transform::from_translation(Vec2::new(0.0, -20.0)),
    ));
    // A parede: alta e fina, com a face esquerda em `WALL_FACE`.
    sim.world_mut().spawn((
        Name::new("Wall"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 0.5,
                half_y: 12.0,
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(WALL_FACE + 0.5, 0.0)),
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
                // ⚠️ A quina e a memória do chão fora do caminho: esta cena não
                // tem teto nem plataforma móvel, e desligá-las explicitamente é
                // o que mantém a fixture a medir UMA wave.
                corner_reach: 0.0,
                lift_momentum: 0.0,
                wall_slide_speed: slide,
                wall_jump_height: jump_height,
                ..PlatformPlayer::default()
            },
            Transform::from_translation(Vec2::new(0.35, START_Y)),
        ))
        .id();

    Rig {
        sim,
        bridge: PhysicsBridge::new(),
        player,
    }
}
