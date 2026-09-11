//! **Os povoadores partilhados das cenas de física** — o chão, o personagem e a
//! laje estática.
//!
//! ⚠️ **Porque é que estes três vieram para cá e não ficaram onde estavam:** eles
//! viviam em ficheiros que **não** podem sair da shell (o roteador, que precisa da
//! `App`; e o `physics_smoke_player`, que autora uma track de timeline), mas eles
//! próprios são **leis puras sobre um `World`** — não conhecem janela, nem `gfx`,
//! nem painel. Deixá-los lá prenderia 18 cenas à shell por causa de uma função de
//! vinte linhas: *o que decide onde uma função mora é o que ela TOCA, nunca o
//! ficheiro em que foi escrita*.

use bevy_ecs::world::World;
use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PlatformPlayer, RigidBody,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// A altura de flutuação das cenas de player — ver o aviso do módulo delas.
const FLOAT: f32 = 0.9;

/// Chão estático, centrado em `y = -1` (topo em `y = -0.8`). O quad da sprite
/// (tamanho cheio) casa com o collider (meias-extensões).
pub fn spawn_floor(world: &mut World) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, -1.0)),
        Sprite::atlas(WHITE_TILE_KEY, [8.0, 0.4], [0.40, 0.42, 0.48, 1.0]),
        Name::new("Floor"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 4.0,
                half_y: 0.2,
            },
            density: 1.0,
            ..Collider::default()
        },
    ));
}

/// O personagem: cápsula dinâmica, rotação travada (D4), com a config do plano.
pub fn spawn_player(world: &mut World, at: Vec2) -> Entity {
    world
        .spawn((
            Name::new("Player"),
            Transform::from_translation(at),
            Sprite::atlas(WHITE_TILE_KEY, [0.4, 1.0], [0.25, 0.85, 1.0, 1.0]),
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
        ))
        .id()
}

/// Um bloco estático, opcionalmente inclinado.
pub fn slab(
    world: &mut World,
    name: &str,
    at: Vec2,
    half: [f32; 2],
    rot: f32,
    tint: [f32; 4],
) -> Entity {
    world
        .spawn((
            Name::new(name.to_string()),
            Transform {
                rotation: rot,
                ..Transform::from_translation(at)
            },
            Sprite::atlas(WHITE_TILE_KEY, [half[0] * 2.0, half[1] * 2.0], tint),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: half[0],
                    half_y: half[1],
                },
                ..Collider::default()
            },
        ))
        .id()
}
